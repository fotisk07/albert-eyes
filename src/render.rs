use crate::status::{AlbertStatus, BackupStatus, DiskAvailability, DiskHealth, DiskStatus};

const CARD_WIDTH: usize = 55;
const INNER_WIDTH: usize = CARD_WIDTH - 2;
const BAR_WIDTH: usize = 10;

fn row(content: &str) -> String {
    let content: String = content.chars().take(INNER_WIDTH).collect();
    format!("│{content:<INNER_WIDTH$}│")
}

fn centered_row(content: &str) -> String {
    let width = content.chars().count();
    let padding = INNER_WIDTH.saturating_sub(width) / 2;
    row(&format!("{}{content}", " ".repeat(padding)))
}

fn face() -> String {
    [
        row(""),
        centered_row("╭───────╮     ╭───────╮"),
        centered_row("│   ●   │     │   ●   │"),
        centered_row("╰───────╯     ╰───────╯"),
        centered_row("╰───╯"),
        row(""),
    ]
    .join("\n")
}

fn bar(available_gib: u16, total_gib: u16) -> String {
    let available_percent = if total_gib == 0 {
        0
    } else {
        u32::from(available_gib) * 100 / u32::from(total_gib)
    };
    let filled = available_percent as usize * BAR_WIDTH / 100;

    format!("{}{}", "█".repeat(filled), "░".repeat(BAR_WIDTH - filled))
}

fn disk_row(name: &str, disk: &DiskStatus) -> String {
    match disk.availability {
        DiskAvailability::Mounted => {
            let temperature = disk
                .temperature_c
                .map(|value| format!("{value}°C"))
                .unwrap_or_else(|| "--°C".into());

            let health = match disk.health {
                Some(DiskHealth::Healthy) => "✓",
                Some(DiskHealth::Sick) => "!",
                None => "?",
            };

            let capacity = match (disk.available_gib, disk.total_gib) {
                (Some(available), Some(total)) => {
                    format!("{available}G free  {}", bar(available, total))
                }
                _ => "--G free  ░░░░░░░░░░".into(),
            };

            centered_row(&format!("{name:<5} {temperature:<5} {health}   {capacity}"))
        }
        DiskAvailability::Unmounted => centered_row(&format!("{name:<5} UNMOUNTED")),
        DiskAvailability::Missing => centered_row(&format!("{name:<5} MISSING")),
        DiskAvailability::Unknown => centered_row(&format!("{name:<5} UNKNOWN")),
    }
}

fn backup_age(age_minutes: u32) -> String {
    match age_minutes {
        0 => "now".into(),
        minutes if minutes < 60 => format!("{minutes}m"),
        minutes if minutes < 24 * 60 => format!("{}h", minutes / 60),
        minutes => format!("{}d", minutes / (24 * 60)),
    }
}

fn backup_status(status: &BackupStatus) -> String {
    match status {
        BackupStatus::Current { age_minutes } => format!("{}✓", backup_age(*age_minutes)),
        BackupStatus::Running => "RUN!".into(),
        BackupStatus::Stale { age_minutes } => format!("{}!", backup_age(*age_minutes)),
        BackupStatus::Unavailable => "?".into(),
    }
}

pub fn render(status: &AlbertStatus) -> String {
    let pi_temperature = status
        .pi
        .temperature_c
        .map(|value| format!("{value}°C"))
        .unwrap_or_else(|| "--°C".into());
    let ram = status
        .pi
        .ram_percent
        .map(|value| format!("{value}%"))
        .unwrap_or_else(|| "--%".into());

    let backups = format!(
        "BKP XPS→AL {}  XPS→BERT {}  AL→BERT {}",
        backup_status(&status.backups.xps_to_al),
        backup_status(&status.backups.xps_to_bert),
        backup_status(&status.backups.al_to_bert),
    );

    [
        format!("┌{}┐", "─".repeat(INNER_WIDTH)),
        face(),
        disk_row("AL", &status.al),
        disk_row("BERT", &status.bert),
        centered_row(&format!("PI    {pi_temperature:<5} · RAM {ram}")),
        centered_row(&backups),
        format!("└{}┘", "─".repeat(INNER_WIDTH)),
    ]
    .join("\n")
}
