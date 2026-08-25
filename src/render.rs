use crate::{BackupStatus, CpStatus, StatusSnapshot};

const CARD_WIDTH: usize = 55;
const INNER_WIDTH: usize = CARD_WIDTH - 2;
const BAR_WIDTH: usize = 10;

fn bounded(text: &str, max_width: usize) -> String {
    text.chars().take(max_width).collect()
}

fn row(content: &str) {
    let content = bounded(content, INNER_WIDTH);
    println!("│{:<53}│", content);
}

fn bar(percent: f64) -> String {
    let percent = percent.clamp(0.0, 100.0);

    let filled = ((percent / 100.0) * BAR_WIDTH as f64).round() as usize;
    let filled = filled.min(BAR_WIDTH);
    let empty = BAR_WIDTH - filled;

    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn temperature_text(temperature: Option<i32>) -> String {
    match temperature {
        Some(v) if v < 60 => format!("{:.0}°C · comfy", v),
        Some(v) if v < 75 => format!("{:.0}°C · warm", v),
        Some(v) => format!("{:.0}°C · alarmed", v),
        None => String::from("-- · puzzled"),
    }
}

fn cpu_text(cpu_usage: Option<f64>) -> String {
    match cpu_usage {
        Some(v) if v < 30.0 => format!("{:.0}% · relaxed", v),
        Some(v) if v < 70.0 => format!("{:.0}% · awake", v),
        Some(v) => format!("{:.0}% · energetic", v),
        None => String::from("-- · unknown"),
    }
}

fn uptime_text(snapshot: &StatusSnapshot) -> String {
    match &snapshot.uptime {
        Some(v) if v.days > 0 => {
            format!("{}d{}h{}m", v.days, v.hours, v.minutes)
        }
        Some(v) if v.hours > 0 => {
            format!("{}h{}m", v.hours, v.minutes)
        }
        Some(v) => format!("{}m", v.minutes),
        None => String::from("--"),
    }
}

fn copyparty_text(status: &CpStatus) -> &'static str {
    match status {
        CpStatus::Running => "COPY OK",
        CpStatus::Stopped => "COPY STOP",
        CpStatus::Failed => "COPY FAIL",
        CpStatus::Unknown => "COPY ?",
    }
}

fn backup_text(status: &BackupStatus) -> String {
    match status {
        BackupStatus::Current { age_hours } => {
            format!("BACKUP {}h ago", age_hours)
        }
        BackupStatus::Stale { age_hours } => {
            format!("BACKUP STALE {}h", age_hours)
        }
        BackupStatus::Unavailable => String::from("BACKUP --"),
        BackupStatus::Checking => String::from("BACKUP ..."),
    }
}

pub fn render(snapshot: &StatusSnapshot) {
    let temperature = temperature_text(snapshot.temperature);
    let cpu = cpu_text(snapshot.cpu_usage);

    let ram = match &snapshot.memory {
        Some(v) if v.total > 0 => {
            let percent = (v.used as f64 / v.total as f64) * 100.0;
            let percent = percent.clamp(0.0, 100.0);

            format!("{:.0}% {}", percent, bar(percent))
        }
        _ => String::from("-- ----------"),
    };

    let storage = match &snapshot.storage {
        Some(v) => {
            let percent = (v.percent_used as f64).clamp(0.0, 100.0);
            format!("{:.0}% {}", percent, bar(percent))
        }
        None => String::from("-- ----------"),
    };

    let disk = match &snapshot.disk_activity {
        Some(v) => format!("DISK R {:.1} · W {:.1} MiB/s", v.read_mib_s, v.write_mib_s),
        None => String::from("DISK R -- · W -- MiB/s"),
    };

    let uptime = uptime_text(snapshot);
    let copyparty = copyparty_text(&snapshot.copyparty);
    let backup = backup_text(&snapshot.backup);

    println!("┌{}┐", "─".repeat(INNER_WIDTH));

    // Reserved face area.
    row("           ╭───────╮             ╭───────╮           ");
    row("           │   ●   │             │   ●   │           ");
    row("           ╰───────╯             ╰───────╯           ");
    row("                          ᴗ                          ");
    row("                        ╰───╯                        ");
    row("");

    row(&format!(
        "  TEMP {:<20} CPU {}",
        bounded(&temperature, 20),
        bounded(&cpu, 20)
    ));

    row(&format!(
        "  RAM  {:<19} STORE {}",
        bounded(&ram, 19),
        bounded(&storage, 19)
    ));

    row(&format!("  {}", disk));

    row(&format!(
        "  UP {:<12} {:<14} {}",
        bounded(&uptime, 12),
        copyparty,
        bounded(&backup, 20)
    ));

    println!("└{}┘", "─".repeat(INNER_WIDTH));
}
