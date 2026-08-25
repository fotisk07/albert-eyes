// This file collects
//  - Temperature
//  - uptime
//  - Memusage
//  - storage usage
//  - copyparty
//  - backup

use chrono::{DateTime, Utc};
use std::process::Command;
use std::{fs, time};

use crate::{BackupStatus, CpStatus, CpuSample, DiskSample, MemoryUsage, StorageUsage, Uptime};

const ALBERT_DISK_DEVICE: &str = "sda1";

// TEMP
pub fn temperature() -> Option<i32> {
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

// UPTIME

pub fn uptime() -> Option<Uptime> {
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

// MEMORY

pub fn memory() -> Option<MemoryUsage> {
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

// STORAGE

pub fn storage() -> Option<StorageUsage> {
    const BYTES_PER_GIB: f64 = 1_073_741_824.0;

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

// COPYPARTY

pub fn copyparty() -> CpStatus {
    let output = Command::new("systemctl")
        .args(["--user", "is-active", "copyparty"])
        .output();

    match output {
        Ok(v) => {
            let text = String::from_utf8_lossy(&v.stdout);
            let status = text.trim();

            match status {
                "active" => CpStatus::Running,
                "inactive" => CpStatus::Stopped,
                "failed" => CpStatus::Failed,
                _ => CpStatus::Unknown,
            }
        }
        Err(_) => CpStatus::Unknown,
    }
}

// BACKUPS
pub fn backup() -> BackupStatus {
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

pub fn disk_sample() -> Option<DiskSample> {
    let diskstats = fs::read_to_string("/proc/diskstats").ok()?;

    let fields = diskstats
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>())
        .find(|fields| fields.get(2) == Some(&ALBERT_DISK_DEVICE))?;

    let sectors_read = fields.get(5)?.parse().ok()?;
    let sectors_written = fields.get(9)?.parse().ok()?;

    Some(DiskSample {
        sectors_read,
        sectors_written,
    })
}

pub fn cpu_sample() -> Option<CpuSample> {
    let stat = fs::read_to_string("/proc/stat").ok()?;

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
