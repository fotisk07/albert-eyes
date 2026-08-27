use crate::{BackupStatus, CpStatus, DiskActivity, MemoryUsage, StatusSnapshot};
use std::fmt::Write;

const CARD_WIDTH: usize = 55;
const INNER_WIDTH: usize = CARD_WIDTH - 2;
const BAR_WIDTH: usize = 10;
const EYE_SEPARATION: usize = 13;
const EYE_WIDTH: usize = 9;
const EYE_WALL_DISTANCE: usize = (INNER_WIDTH - 2 * EYE_WIDTH - EYE_SEPARATION) / 2;

struct Telemetry {
    temperature: String,
    cpu: String,
    storage: String,
    ram: String,
    disk: String,
    uptime: String,
    copyparty: String,
    backup: String,
}

fn bounded(text: &str, max_width: usize) -> String {
    text.chars().take(max_width).collect()
}

fn row(content: &str) -> String {
    let content = bounded(content, INNER_WIDTH);
    format!("│{:<53}│", content)
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
        Some(v) if v < 70 => format!("{:.0}°C · comfy", v),
        Some(v) if v < 80 => format!("{:.0}°C · warm", v),
        Some(v) => format!("{:.0}°C · alarmed", v),
        None => String::from("-- · puzzled"),
    }
}

fn cpu_text(cpu_usage: Option<f64>) -> String {
    match cpu_usage {
        Some(v) if v < 20.0 => format!("{:.0}% · relaxed", v),
        Some(v) if v <= 70.0 => format!("{:.0}% · awake", v),
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

fn copyparty_text(status: &CpStatus) -> String {
    match status {
        CpStatus::Running => format!("COPY OK"),
        CpStatus::Stopped => format!("COPY STOP"),
        CpStatus::Failed => format!("COPY FAIL"),
        CpStatus::Unknown => format!("COPY ?"),
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
        BackupStatus::Unavailable => String::from("BACKUP N/A"),
        BackupStatus::Checking => String::from("BACKUP Checking..."),
    }
}

fn ram_text(memory: &Option<MemoryUsage>) -> String {
    match memory {
        Some(v) if v.total > 0 => {
            let percent = (v.used as f64 / v.total as f64) * 100.0;
            let percent = percent.clamp(0.0, 100.0);

            format!("{:.0}% {}", percent, bar(percent))
        }
        _ => String::from("-- ----------"),
    }
}

fn storage_text(storage: &Option<u8>) -> String {
    match storage {
        Some(v) => {
            let percent = (*v as f64).clamp(0.0, 100.0);
            format!("{:.0}% {}", percent, bar(percent))
        }
        None => String::from("-- ----------"),
    }
}
fn disk_text(disk: &Option<DiskActivity>) -> String {
    match disk {
        Some(v) => format!("DISK R {:.1} · W {:.1} MiB/s", v.read_mib_s, v.write_mib_s),
        None => String::from("DISK R -- · W -- MiB/s"),
    }
}

fn render_pupils(pupil_pos: usize) -> (String, String) {
    let left: String = (0..7)
        .map(|position| if pupil_pos == position { '●' } else { ' ' })
        .collect();
    let right: String = (0..7)
        .map(|position| if pupil_pos == position { '●' } else { ' ' })
        .collect();

    (left, right)
}

fn create_head_str(pupil_pos: usize) -> String {
    let wall = " ".repeat(EYE_WALL_DISTANCE);
    let sep = " ".repeat(EYE_SEPARATION);
    let (lefti, right) = render_pupils(pupil_pos);

    [
        row(""),
        row(&format!("{wall}╭───────╮{sep}╭───────╮")),
        row(&format!("{wall}│{right}│{sep}│{lefti}│")),
        row(&format!("{wall}╰───────╯{sep}╰───────╯")),
        row(&format!("{}ᴗ", " ".repeat(INNER_WIDTH / 2))),
        row(&format!("{}╰───╯", " ".repeat((INNER_WIDTH - 5) / 2))),
        row(""),
    ]
    .join("\n")
}

fn create_tm_text(snapshot: &Telemetry) -> String {
    [
        row(&format!(
            "  TEMP {:<20} CPU {}",
            snapshot.temperature, snapshot.cpu
        )),
        row(&format!(
            "  RAM  {:<19} STORE {}",
            snapshot.ram, snapshot.storage
        )),
        row(&format!("  {}", snapshot.disk)),
        row(&format!(
            "  UP {:<12} {:<14} {}",
            snapshot.uptime, snapshot.copyparty, snapshot.backup
        )),
    ]
    .join("\n")
}

pub fn render(snapshot: &StatusSnapshot, pupil_pos: usize) -> String {
    let telemetry_text = Telemetry {
        temperature: temperature_text(snapshot.temperature),
        cpu: cpu_text(snapshot.cpu_usage),
        storage: storage_text(&snapshot.storage),
        ram: ram_text(&snapshot.memory),
        disk: disk_text(&snapshot.disk_activity),
        uptime: uptime_text(snapshot),
        copyparty: copyparty_text(&snapshot.copyparty),
        backup: backup_text(&snapshot.backup),
    };

    let mut report = String::new();
    let _ = writeln!(&mut report, "┌{}┐", "─".repeat(INNER_WIDTH));
    let _ = writeln!(&mut report, "{}", create_head_str(pupil_pos));
    let _ = writeln!(&mut report, "{}", create_tm_text(&telemetry_text));
    let _ = writeln!(&mut report, "└{}┘", "─".repeat(INNER_WIDTH));

    return report;
}
