use crate::status::{
    AlbertStatus, BackupStatus, BackupStatuses, DiskAvailability, DiskHealth, DiskStatus, PiStatus,
};
use std::path::Path;
use std::time::{Duration, SystemTime};
use std::{fs, process::Command};

const AL_DEVICE: &str = "/dev/disk/by-uuid/ba307f60-44e2-42d0-b7af-589753384ebd";
const BERT_DEVICE: &str = "/dev/disk/by-uuid/f71b55fd-9bd2-427e-bc54-d1a00cc5d6ce";

const XPS_TO_AL_REPO: &str = "/srv/storage/backups/fotis-xps";
const XPS_TO_BERT_REPO: &str = "/srv/recovery/computer-backups/fotis-xps";
const AL_TO_BERT_REPO: &str = "/srv/recovery/shared";

const DAILY_BACKUP_STALE_HOURS: u16 = 48;
const WEEKDAY_BACKUP_STALE_HOURS: u16 = 96;

// Linux interfaces
const TEMP_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";
const MEMINFO_PATH: &str = "/proc/meminfo";
const SMART_CACHE_PATH: &str = "/run/albert-eyes/smart.json";

pub fn collect_status() -> AlbertStatus {
    let mut al = collect_disk_status(AL_DEVICE, "/srv/storage");
    let mut bert = collect_disk_status(BERT_DEVICE, "/srv/recovery");
    apply_smart_cache(&mut al, &mut bert);

    AlbertStatus {
        al,
        bert,
        pi: collect_pi_status(),
        backups: collect_backup_statuses(),
    }
}

fn apply_smart_cache(al: &mut DiskStatus, bert: &mut DiskStatus) {
    let Ok(contents) = fs::read_to_string(SMART_CACHE_PATH) else {
        return;
    };
    let Ok(cache) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return;
    };

    apply_disk_smart(al, cache.get("al"));
    apply_disk_smart(bert, cache.get("bert"));
}

fn apply_disk_smart(disk: &mut DiskStatus, smart: Option<&serde_json::Value>) {
    if !matches!(&disk.availability, DiskAvailability::Mounted) {
        return;
    }

    disk.temperature_c = smart
        .and_then(|value| value.get("temperature_c"))
        .and_then(serde_json::Value::as_u64)
        .and_then(|temperature| u8::try_from(temperature).ok());

    disk.health = smart
        .and_then(|value| value.get("healthy"))
        .and_then(serde_json::Value::as_bool)
        .map(|healthy| {
            if healthy {
                DiskHealth::Healthy
            } else {
                DiskHealth::Sick
            }
        });
}

fn collect_pi_status() -> PiStatus {
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

fn collect_ram_percent() -> Option<u8> {
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

fn collect_backup_statuses() -> BackupStatuses {
    let mut statuses = BackupStatuses {
        xps_to_al: collect_backup_status(XPS_TO_AL_REPO, DAILY_BACKUP_STALE_HOURS),
        xps_to_bert: collect_backup_status(XPS_TO_BERT_REPO, WEEKDAY_BACKUP_STALE_HOURS),
        al_to_bert: collect_backup_status(AL_TO_BERT_REPO, DAILY_BACKUP_STALE_HOURS),
    };

    match std::env::var("ALBERT_EYES_BACKUP").as_deref() {
        Ok("xps-to-al") => statuses.xps_to_al = BackupStatus::Running,
        Ok("xps-to-bert") => statuses.xps_to_bert = BackupStatus::Running,
        Ok("al-to-bert") => statuses.al_to_bert = BackupStatus::Running,
        _ => {}
    }

    statuses
}

fn collect_backup_status(repository: &str, stale_hours: u16) -> BackupStatus {
    let repository = Path::new(repository);

    let Ok(locks) = fs::read_dir(repository.join("locks")) else {
        return BackupStatus::Unavailable;
    };

    if locks.into_iter().next().is_some() {
        return BackupStatus::Running;
    }

    let Ok(entries) = fs::read_dir(repository.join("snapshots")) else {
        return BackupStatus::Unavailable;
    };

    let latest = entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.metadata().ok()?.modified().ok())
        .max();

    let Some(latest) = latest else {
        return BackupStatus::Unavailable;
    };

    let age = SystemTime::now()
        .duration_since(latest)
        .unwrap_or(Duration::ZERO);
    let age_minutes = u32::try_from(age.as_secs() / 60).unwrap_or(u32::MAX);
    let stale_after_minutes = u32::from(stale_hours) * 60;

    if age_minutes > stale_after_minutes {
        BackupStatus::Stale { age_minutes }
    } else {
        BackupStatus::Current { age_minutes }
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
