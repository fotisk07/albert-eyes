use crate::status::{DiskAvailability, DiskHealth, DiskStatus, PiStatus};
use std::path::Path;
use std::{fs, process::Command};

const RESTIC_REPO: &str = "/srv/storage/backups/fotis-xps/";
const RESTIC_PASSWORD_FILE: &str = "/home/fotis/.config/albert-eyes/restic/restic-password";
const BACKUP_STALE_HOURS: i64 = 48;

// Linux interfaces
const TEMP_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";
const MEMINFO_PATH: &str = "/proc/meminfo";

pub fn collect_pi_status() -> PiStatus {
    PiStatus {
        temperature_c: {
            fs::read_to_string(TEMP_PATH)
                .ok()
                .and_then(|s| s.trim().parse::<i32>().ok())
                .map(|value| (value / 1000) as u8)
        },
        ram_percent: collect_ram_percent(),
    }
}

pub fn collect_ram_percent() -> Option<u8> {
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

    Some(((total - available) * 100 / total) as u8)
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

// Stale code
// pub enum CpStatus {
//     Running,
//     Stopped,
//     Failed,
//     Unknown,
// }

// pub enum BackupStatus {
//     Checking,
//     Current { age_hours: i64 },
//     Stale { age_hours: i64 },
//     Unavailable,
// }

// pub fn copyparty() -> CpStatus {
//     let Ok(output) = Command::new("systemctl")
//         .args(["--user", "is-active", "copyparty"])
//         .output()
//     else {
//         return CpStatus::Unknown;
//     };

//     match String::from_utf8_lossy(&output.stdout).trim() {
//         "active" => CpStatus::Running,
//         "inactive" => CpStatus::Stopped,
//         "failed" => CpStatus::Failed,
//         _ => CpStatus::Unknown,
//     }
// }

// pub fn backup() -> BackupStatus {
//     let Ok(output) = Command::new("restic")
//         .args([
//             "--password-file",
//             RESTIC_PASSWORD_FILE,
//             "-r",
//             RESTIC_REPO,
//             "snapshots",
//             "--latest",
//             "1",
//             "--json",
//         ])
//         .output()
//     else {
//         return BackupStatus::Unavailable;
//     };

//     if !output.status.success() {
//         return BackupStatus::Unavailable;
//     }

//     let parsed: serde_json::Value = match serde_json::from_slice(&output.stdout) {
//         Ok(value) => value,
//         Err(_) => return BackupStatus::Unavailable,
//     };

//     let Some(time) = parsed
//         .get(0)
//         .and_then(|snapshot| snapshot.get("time"))
//         .and_then(|time| time.as_str())
//     else {
//         return BackupStatus::Unavailable;
//     };

//     let Ok(snapshot_time) = DateTime::parse_from_rfc3339(time) else {
//         return BackupStatus::Unavailable;
//     };

//     let age_hours = (Utc::now() - snapshot_time.with_timezone(&Utc)).num_hours();

//     if age_hours > BACKUP_STALE_HOURS {
//         BackupStatus::Stale { age_hours }
//     } else {
//         BackupStatus::Current { age_hours }
//     }
// }
