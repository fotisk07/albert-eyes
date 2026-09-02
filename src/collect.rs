use crate::status::{DiskAvailability, DiskHealth, DiskStatus, PiStatus};
use chrono::{DateTime, Utc};
use std::path::Path;
use std::{
    fs,
    process::Command,
    time::{Duration, Instant},
};

// Albert configuration
const DISK_DEVICE: &str = "sda1";
const STORAGE_PATH: &str = "/srv/storage";
const RESTIC_REPO: &str = "/srv/storage/backups/fotis-xps/";
const RESTIC_PASSWORD_FILE: &str = "/home/fotis/.config/albert-eyes/restic/restic-password";
const BACKUP_STALE_HOURS: i64 = 48;

// Linux interfaces
const TEMP_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";
const UPTIME_PATH: &str = "/proc/uptime";
const MEMINFO_PATH: &str = "/proc/meminfo";
const CPU_STAT_PATH: &str = "/proc/stat";
const DISK_STATS_PATH: &str = "/proc/diskstats";

// Disk conversion
const BYTES_PER_SECTOR: f64 = 512.0;
const BYTES_PER_MIB: f64 = 1_048_576.0;

pub fn collect_pi_status() -> PiStatus {
    PiStatus {
        temperature_c: Some(10),
        cpu_percent: Some(10),
        ram_percent: Some(10),
    }
}

pub fn collect_disk_status(uid_path: &str, mount_path: &str) -> DiskStatus {
    let availability = match Path::new(uid_path).exists() {
        true => is_mounted(
            mount_path,
            uid_path.strip_prefix("/dev/disk/by-uuid/").unwrap(),
        ),
        false => DiskAvailability::Missing,
    };
    let (total_gib, available_gib) = match availability {
        DiskAvailability::Mounted => get_total_and_available_gib(mount_path),
        _ => (None, None),
    };
    DiskStatus {
        availability,
        temperature_c: None,
        available_gib,
        total_gib,
        health: None,
    }
}

fn is_mounted(mount_path: &str, uuid: &str) -> DiskAvailability {
    let output = Command::new("findmnt")
        .args([
            "--noheadings",
            "--raw",
            "--mountpoint",
            mount_path,
            "--output",
            "UUID",
        ])
        .output();

    let Ok(output) = output else {
        return DiskAvailability::Unknown;
    };

    let found_uuid = String::from_utf8_lossy(&output.stdout);

    if found_uuid.trim() == uuid {
        DiskAvailability::Mounted
    } else {
        DiskAvailability::Unmounted
    }
}

fn get_total_and_available_gib(mount_path: &str) -> (Option<u16>, Option<u16>) {
    let Ok(output) = Command::new("df").args(["-BG", mount_path]).output() else {
        return (None, None);
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(line) = stdout.lines().nth(1) else {
        return (None, None);
    };

    let fields: Vec<_> = line.split_whitespace().collect();

    (
        fields[1].trim_end_matches('G').parse().ok(),
        fields[3].trim_end_matches('G').parse().ok(),
    )
}

pub struct Uptime {
    pub days: u64,
    pub hours: u64,
    pub minutes: u64,
}

pub struct MemoryUsage {
    pub used: u64,
    pub total: u64,
}

pub struct DiskActivity {
    pub read_mib_s: f64,
    pub write_mib_s: f64,
}

pub struct CheapTelemetry {
    pub temperature: Option<i32>,
    pub uptime: Option<Uptime>,
    pub memory: Option<MemoryUsage>,
}

pub struct ActivityTelemetry {
    pub cpu_usage: Option<f64>,
    pub disk_activity: Option<DiskActivity>,
}

pub enum CpStatus {
    Running,
    Stopped,
    Failed,
    Unknown,
}

pub enum BackupStatus {
    Checking,
    Current { age_hours: i64 },
    Stale { age_hours: i64 },
    Unavailable,
}

struct CpuSample([u64; 8]);

struct DiskSample {
    read: u64,
    written: u64,
}

pub struct ActivityCollector {
    previous_cpu: Option<CpuSample>,
    previous_disk: Option<(DiskSample, Instant)>,
}

impl CheapTelemetry {
    pub fn collect() -> Self {
        Self {
            temperature: temperature(),
            uptime: uptime(),
            memory: memory(),
        }
    }
}

impl ActivityCollector {
    pub fn new() -> Self {
        Self {
            previous_cpu: None,
            previous_disk: None,
        }
    }

    pub fn collect(&mut self) -> ActivityTelemetry {
        let now = Instant::now();

        let cpu = cpu_sample();

        let cpu_usage = self
            .previous_cpu
            .as_ref()
            .zip(cpu.as_ref())
            .and_then(|(previous, current)| calculate_util(previous, current));

        self.previous_cpu = cpu;

        let disk = disk_sample();

        let disk_activity = self.previous_disk.as_ref().zip(disk.as_ref()).and_then(
            |((previous, previous_time), current)| {
                calculate_disk_activity(previous, current, now.duration_since(*previous_time))
            },
        );

        self.previous_disk = disk.map(|sample| (sample, now));

        ActivityTelemetry {
            cpu_usage,
            disk_activity,
        }
    }
}

impl Default for ActivityCollector {
    fn default() -> Self {
        Self::new()
    }
}

pub fn temperature() -> Option<i32> {
    fs::read_to_string(TEMP_PATH)
        .ok()?
        .trim()
        .parse::<i32>()
        .ok()
        .map(|value| value / 1000)
}

pub fn uptime() -> Option<Uptime> {
    let seconds = fs::read_to_string(UPTIME_PATH)
        .ok()?
        .split_whitespace()
        .next()?
        .parse::<f64>()
        .ok()? as u64;

    Some(Uptime {
        days: seconds / 86_400,
        hours: (seconds % 86_400) / 3_600,
        minutes: (seconds % 3_600) / 60,
    })
}

pub fn memory() -> Option<MemoryUsage> {
    let meminfo = fs::read_to_string(MEMINFO_PATH).ok()?;

    let value = |name: &str| {
        meminfo
            .lines()
            .find(|line| line.starts_with(name))?
            .split_whitespace()
            .nth(1)?
            .parse::<u64>()
            .ok()
    };

    let total = value("MemTotal:")?;
    let available = value("MemAvailable:")?;

    Some(MemoryUsage {
        used: (total - available) / 1024,
        total: total / 1024,
    })
}

pub fn copyparty() -> CpStatus {
    let Ok(output) = Command::new("systemctl")
        .args(["--user", "is-active", "copyparty"])
        .output()
    else {
        return CpStatus::Unknown;
    };

    match String::from_utf8_lossy(&output.stdout).trim() {
        "active" => CpStatus::Running,
        "inactive" => CpStatus::Stopped,
        "failed" => CpStatus::Failed,
        _ => CpStatus::Unknown,
    }
}

pub fn backup() -> BackupStatus {
    let Ok(output) = Command::new("restic")
        .args([
            "--password-file",
            RESTIC_PASSWORD_FILE,
            "-r",
            RESTIC_REPO,
            "snapshots",
            "--latest",
            "1",
            "--json",
        ])
        .output()
    else {
        return BackupStatus::Unavailable;
    };

    if !output.status.success() {
        return BackupStatus::Unavailable;
    }

    let parsed: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(value) => value,
        Err(_) => return BackupStatus::Unavailable,
    };

    let Some(time) = parsed
        .get(0)
        .and_then(|snapshot| snapshot.get("time"))
        .and_then(|time| time.as_str())
    else {
        return BackupStatus::Unavailable;
    };

    let Ok(snapshot_time) = DateTime::parse_from_rfc3339(time) else {
        return BackupStatus::Unavailable;
    };

    let age_hours = (Utc::now() - snapshot_time.with_timezone(&Utc)).num_hours();

    if age_hours > BACKUP_STALE_HOURS {
        BackupStatus::Stale { age_hours }
    } else {
        BackupStatus::Current { age_hours }
    }
}

fn cpu_sample() -> Option<CpuSample> {
    let stat = fs::read_to_string(CPU_STAT_PATH).ok()?;

    let values = stat
        .lines()
        .find(|line| line.starts_with("cpu "))?
        .split_whitespace()
        .skip(1)
        .take(8)
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;

    Some(CpuSample(values.try_into().ok()?))
}

fn calculate_util(previous: &CpuSample, current: &CpuSample) -> Option<f64> {
    let delta: Vec<u64> = current
        .0
        .iter()
        .zip(&previous.0)
        .map(|(current, previous)| current.checked_sub(*previous))
        .collect::<Option<_>>()?;

    let total: u64 = delta.iter().sum();

    if total == 0 {
        return None;
    }

    let idle = delta[3].checked_add(delta[4])?;
    let busy = total.checked_sub(idle)?;

    Some(busy as f64 / total as f64 * 100.0)
}

fn disk_sample() -> Option<DiskSample> {
    let diskstats = fs::read_to_string(DISK_STATS_PATH).ok()?;

    let fields = diskstats
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>())
        .find(|fields| fields.get(2) == Some(&DISK_DEVICE))?;

    Some(DiskSample {
        read: fields.get(5)?.parse().ok()?,
        written: fields.get(9)?.parse().ok()?,
    })
}

fn calculate_disk_activity(
    previous: &DiskSample,
    current: &DiskSample,
    elapsed: Duration,
) -> Option<DiskActivity> {
    let seconds = elapsed.as_secs_f64();

    if seconds == 0.0 {
        return None;
    }

    let rate = |current: u64, previous: u64| {
        current
            .checked_sub(previous)
            .map(|delta| delta as f64 * BYTES_PER_SECTOR / BYTES_PER_MIB / seconds)
    };

    Some(DiskActivity {
        read_mib_s: rate(current.read, previous.read)?,
        write_mib_s: rate(current.written, previous.written)?,
    })
}
