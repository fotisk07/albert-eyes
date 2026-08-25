use crate::{BackupStatus, CpStatus, StatusSnapshot};

pub fn render(snapshot: &StatusSnapshot) {
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
    let disk = match &snapshot.disk_activity {
        Some(v) => format!("R {:.1} · W {:.1} MiB/s", v.read_mib_s, v.write_mib_s,),
        None => String::from("--"),
    };

    let copyparty = match snapshot.copyparty {
        CpStatus::Running => "Running",
        CpStatus::Stopped => "Stopped",
        CpStatus::Failed => "Failed",
        CpStatus::Unknown => "Unknown",
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

    println!("CPU Usage        : {}", cpu_usage);
    println!("CPU/HDD Temp     : {} / --", temperature);
    println!("Disk I/O         : {}", disk);
    println!("Uptime           : {}", uptime);
    println!("Storage          : {}", storage);
    println!("Memory           : {}", memory);

    println!();

    println!("Copyparty status : {}", copyparty);
    println!("Last Backup      : {}", backup);
}
