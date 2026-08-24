use chrono::{DateTime, Utc};
use std::fs;
use std::io::Write;
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{thread, time};

const BYTES_PER_GIB: f64 = 1_073_741_824.0;
const COPYPARTY_UPDATE: u64 = 30;
const STORAGE_UPDATE: u64 = 60;
const BACKUP_UPDATE_SECS: u64 = 600;

struct Uptime {
    days: u64,
    hours: u64,
    minutes: u64,
}

struct MemoryUsage {
    used: u64,
    total: u64,
}

struct StorageUsage {
    used_gib: f64,
    total_gib: f64,
    percent_used: u8,
}
enum ServiceState {
    Running,
    Stopped,
    Failed,
    Unknown,
}

enum BackupStatus {
    Checking,
    Current { age_hours: i64 },
    Stale { age_hours: i64 },
    Unavailable,
}

struct StatusSnapshot {
    temperature: Option<i32>,
    uptime: Option<Uptime>,
    memory: Option<MemoryUsage>,
    storage: Option<StorageUsage>,
    cpu_usage: Option<f64>,
    copyparty: ServiceState,
    backup: BackupStatus,
}

enum OverallHealth {
    Healthy,
    Attention,
    Warning,
    Unknown,
}
struct CpuSample {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}
fn render(snapshot: &StatusSnapshot, health: &OverallHealth) {
    let temperature = match snapshot.temperature {
        Some(v) => format!("{} °C", v),
        None => String::from("--"),
    };

    let cpu_usage = match snapshot.cpu_usage {
        Some(v) => format!("{:.1}%", v),
        None => String::from("--"),
    };

    let uptime = match &snapshot.uptime {
        Some(v) => format!("{}d {}h {}m", v.days, v.hours, v.minutes),
        None => String::from("--"),
    };

    let memory = match &snapshot.memory {
        Some(v) => format!("{} MiB / {} MiB", v.used, v.total),
        None => String::from("--"),
    };

    let storage = match &snapshot.storage {
        Some(v) => format!(
            "{}% used ({:.1} GiB / {:.1} GiB)",
            v.percent_used, v.used_gib, v.total_gib
        ),
        None => String::from("--"),
    };

    let copyparty = match snapshot.copyparty {
        ServiceState::Running => "Running",
        ServiceState::Stopped => "Stopped",
        ServiceState::Failed => "Failed",
        ServiceState::Unknown => "Unknown",
    };

    let backup = match &snapshot.backup {
        BackupStatus::Current { age_hours } => {
            format!("Current ({} hours ago)", age_hours)
        }
        BackupStatus::Stale { age_hours } => {
            format!("Stale ({} hours ago)", age_hours)
        }
        BackupStatus::Unavailable => String::from("Unavailable"),
        BackupStatus::Checking => String::from("Checking..."),
    };

    let health_text = match health {
        OverallHealth::Healthy => "All good",
        OverallHealth::Attention => "Something is not great",
        OverallHealth::Warning => "AHHHHHH",
        OverallHealth::Unknown => "x x",
    };

    println!("##### {} #####\n", health_text);

    println!("CPU Usage        : {}", cpu_usage);
    println!("CPU/HDD Temp     : {} / --", temperature);
    println!("Uptime           : {}", uptime);
    println!("Storage          : {}", storage);
    println!("Memory           : {}", memory);

    println!();

    println!("Copyparty status : {}", copyparty);
    println!("Last Backup      : {}", backup);
}

fn collect_temperature() -> Option<i32> {
    let temperature: String = match fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        Ok(v) => v,
        Err(_e) => return None,
    };

    temperature
        .trim()
        .parse::<i32>()
        .ok()
        .map(|value| value / 1000)
}

fn collect_uptime() -> Option<Uptime> {
    let uptime: String = fs::read_to_string("/proc/uptime").ok()?;
    let uptime = uptime.split_whitespace().next()?;
    let uptime: f64 = uptime.parse::<f64>().ok()?;
    let total_seconds = time::Duration::try_from_secs_f64(uptime).ok()?.as_secs();

    Some(Uptime {
        days: total_seconds / 86_400,
        hours: (total_seconds % 86_400) / 3_600,
        minutes: (total_seconds % 3_600) / 60,
    })
}

fn collect_memory() -> Option<MemoryUsage> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;

    let mem_total = meminfo
        .lines()
        .find(|line| line.starts_with("MemTotal:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|n| n.parse::<u64>().ok())?;

    let mem_available = meminfo
        .lines()
        .find(|line| line.starts_with("MemAvailable:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|n| n.parse::<u64>().ok())?;

    Some(MemoryUsage {
        used: (mem_total - mem_available) / 1024,
        total: mem_total / 1024,
    })
}

fn collect_storage() -> Option<StorageUsage> {
    let output = Command::new("df")
        .args([
            "--output=size,used,avail,pcent,target",
            "-B1",
            "/srv/storage",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().nth(1)?;
    let parts: Vec<&str> = line.split_whitespace().collect();

    let total_bytes = parts.first()?.parse::<u64>().ok()?;
    let used_bytes = parts.get(1)?.parse::<u64>().ok()?;
    let percent_used = parts.get(3)?.trim_end_matches('%').parse::<u8>().ok()?;

    Some(StorageUsage {
        used_gib: used_bytes as f64 / BYTES_PER_GIB,
        total_gib: total_bytes as f64 / BYTES_PER_GIB,
        percent_used,
    })
}

fn collect_copyparty() -> ServiceState {
    let output = Command::new("systemctl")
        .args(["--user", "is-active", "copyparty"])
        .output();

    match output {
        Ok(v) => {
            let text = String::from_utf8_lossy(&v.stdout);
            let status = text.trim();

            match status {
                "active" => ServiceState::Running,
                "inactive" => ServiceState::Stopped,
                "failed" => ServiceState::Failed,
                _ => ServiceState::Unknown,
            }
        }
        Err(_) => ServiceState::Unknown,
    }
}

fn collect_backup() -> BackupStatus {
    let restic = Command::new("restic")
        .args([
            "--password-file",
            "/home/fotis/.config/albert-eyes/restic/restic-password",
            "-r",
            "/srv/storage/backups/fotis-xps/",
            "snapshots",
            "--latest",
            "1",
            "--json",
        ])
        .output();

    match restic {
        Ok(v) => {
            if v.status.success() {
                let restic_json = String::from_utf8_lossy(&v.stdout).to_string();

                let parsed = serde_json::from_str::<serde_json::Value>(&restic_json);

                let parsed = match parsed {
                    Ok(v) => v,
                    Err(_) => return BackupStatus::Unavailable,
                };
                let time_str = parsed[0]["time"].as_str().unwrap_or("Unknown").to_string();
                let snapshot_time = DateTime::parse_from_rfc3339(&time_str);
                match snapshot_time {
                    Ok(v) => {
                        let now = Utc::now();
                        let age = now - v.with_timezone(&Utc);

                        if age.num_hours() > 48 {
                            BackupStatus::Stale {
                                age_hours: age.num_hours(),
                            }
                        } else {
                            BackupStatus::Current {
                                age_hours: age.num_hours(),
                            }
                        }
                    }
                    Err(_) => BackupStatus::Unavailable,
                }
            } else {
                BackupStatus::Unavailable
            }
        }
        Err(_) => BackupStatus::Unavailable,
    }
}

fn parse_cpu_sample(stat: &str) -> Option<CpuSample> {
    let line = stat
        .lines()
        .find(|line| line.split_whitespace().next() == Some("cpu"))?;

    let mut fields = line.split_whitespace();

    // Consume exactly "cpu"
    if fields.next()? != "cpu" {
        return None;
    }

    Some(CpuSample {
        user: fields.next()?.parse().ok()?,
        nice: fields.next()?.parse().ok()?,
        system: fields.next()?.parse().ok()?,
        idle: fields.next()?.parse().ok()?,
        iowait: fields.next()?.parse().ok()?,
        irq: fields.next()?.parse().ok()?,
        softirq: fields.next()?.parse().ok()?,
        steal: fields.next()?.parse().ok()?,
    })
}

fn calculate_util(previous: &CpuSample, current: &CpuSample) -> Option<f64> {
    let user = current.user.checked_sub(previous.user)?;
    let nice = current.nice.checked_sub(previous.nice)?;
    let system = current.system.checked_sub(previous.system)?;
    let idle = current.idle.checked_sub(previous.idle)?;
    let iowait = current.iowait.checked_sub(previous.iowait)?;
    let irq = current.irq.checked_sub(previous.irq)?;
    let softirq = current.softirq.checked_sub(previous.softirq)?;
    let steal = current.steal.checked_sub(previous.steal)?;

    let total_delta = user
        .checked_add(nice)?
        .checked_add(system)?
        .checked_add(idle)?
        .checked_add(iowait)?
        .checked_add(irq)?
        .checked_add(softirq)?
        .checked_add(steal)?;

    if total_delta == 0 {
        return None;
    }

    let idle_delta = idle.checked_add(iowait)?;
    let busy_delta = total_delta.checked_sub(idle_delta)?;

    Some((busy_delta as f64 / total_delta as f64) * 100.0)
}

fn collect_cpu_sample() -> Option<CpuSample> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    parse_cpu_sample(&stat)
}

fn derive_health(snapshot: &StatusSnapshot) -> OverallHealth {
    match snapshot.temperature {
        None => OverallHealth::Unknown,
        Some(v) => {
            if v < 70 {
                OverallHealth::Healthy
            } else if v < 80 {
                OverallHealth::Attention
            } else {
                OverallHealth::Warning
            }
        }
    }
}

fn main() {
    let mut status = StatusSnapshot {
        temperature: collect_temperature(),
        uptime: collect_uptime(),
        memory: collect_memory(),
        storage: collect_storage(),
        cpu_usage: None,
        copyparty: collect_copyparty(),
        backup: BackupStatus::Checking,
    };
    let mut copyparty_times = Instant::now();
    let mut storage_times = Instant::now();

    let (tx, rx) = mpsc::channel();

    let worker_sender = tx.clone();
    thread::spawn(move || {
        let val = collect_backup();
        let _ = worker_sender.send(val);
    });
    let mut backup_command_running = true;
    let mut backup_times = Instant::now();

    let mut previous_cpu = None;

    loop {
        status.temperature = collect_temperature();
        status.uptime = collect_uptime();
        status.memory = collect_memory();

        let current_cpu = collect_cpu_sample();

        status.cpu_usage = match (&previous_cpu, &current_cpu) {
            (Some(previous), Some(current)) => calculate_util(previous, current),
            _ => None,
        };

        previous_cpu = current_cpu;
        let received = rx.try_recv();
        match received {
            Ok(v) => {
                status.backup = v;
                backup_command_running = false;
            }
            Err(_) => {
                if backup_times.elapsed() >= Duration::from_secs(BACKUP_UPDATE_SECS)
                    && !backup_command_running
                {
                    let worker_sender = tx.clone();
                    thread::spawn(move || {
                        let val = collect_backup();
                        let _ = worker_sender.send(val);
                    });
                    backup_command_running = true;
                    backup_times = Instant::now();
                }
            }
        }

        if copyparty_times.elapsed() >= Duration::from_secs(COPYPARTY_UPDATE) {
            status.copyparty = collect_copyparty();
            copyparty_times = Instant::now();
        }
        if storage_times.elapsed() >= Duration::from_secs(STORAGE_UPDATE) {
            status.storage = collect_storage();
            storage_times = Instant::now();
        }

        let health = derive_health(&status);
        print!("\x1B[2J");
        print!("\x1B[H");

        render(&status, &health);
        let _ = std::io::stdout().flush();

        let sleep_duration = time::Duration::from_secs(2);
        thread::sleep(sleep_duration);
    }
}
