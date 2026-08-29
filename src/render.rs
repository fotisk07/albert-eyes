use crate::{BackupStatus, CpStatus, StatusSnapshot, animation::PupilState};

const CARD_WIDTH: usize = 55;
const INNER_WIDTH: usize = CARD_WIDTH - 2;
const BAR_WIDTH: usize = 10;
const EYE_SEPARATION: usize = 13;
const EYE_WIDTH: usize = 9;
const EYE_WALL_DISTANCE: usize = (INNER_WIDTH - 2 * EYE_WIDTH - EYE_SEPARATION) / 2;

fn row(content: &str) -> String {
    let content: String = content.chars().take(INNER_WIDTH).collect();
    format!("│{content:<53}│")
}

fn bar(percent: f64) -> String {
    let percent = percent.clamp(0.0, 100.0);
    let filled = ((percent / 100.0) * BAR_WIDTH as f64).round() as usize;

    format!("{}{}", "█".repeat(filled), "░".repeat(BAR_WIDTH - filled))
}

fn head(pupils: PupilState) -> String {
    let wall = " ".repeat(EYE_WALL_DISTANCE);
    let sep = " ".repeat(EYE_SEPARATION);

    let pupil: String = (0..7)
        .map(|i| {
            if pupils.visible && i == pupils.position {
                '●'
            } else {
                ' '
            }
        })
        .collect();

    [
        row(""),
        row(&format!("{wall}╭───────╮{sep}╭───────╮")),
        row(&format!("{wall}│{pupil}│{sep}│{pupil}│")),
        row(&format!("{wall}╰───────╯{sep}╰───────╯")),
        row(&format!("{}ᴗ", " ".repeat(INNER_WIDTH / 2))),
        row(&format!("{}╰───╯", " ".repeat((INNER_WIDTH - 5) / 2))),
        row(""),
    ]
    .join("\n")
}

pub fn render(snapshot: &StatusSnapshot, pupils: PupilState) -> String {
    let cheap = &snapshot.cheap_telemetry;
    let activity = &snapshot.activity_telemetry;

    let temperature = match cheap.temperature {
        Some(v) if v < 70 => format!("{v}°C · comfy"),
        Some(v) if v < 80 => format!("{v}°C · warm"),
        Some(v) => format!("{v}°C · alarmed"),
        None => "-- · puzzled".into(),
    };

    let cpu = match activity.cpu_usage {
        Some(v) if v < 20.0 => format!("{v:.0}% · relaxed"),
        Some(v) if v <= 70.0 => format!("{v:.0}% · awake"),
        Some(v) => format!("{v:.0}% · energetic"),
        None => "-- · unknown".into(),
    };

    let ram = match &cheap.memory {
        Some(v) if v.total > 0 => {
            let percent = v.used as f64 / v.total as f64 * 100.0;
            format!("{percent:.0}% {}", bar(percent))
        }
        _ => "-- ----------".into(),
    };

    let storage = match snapshot.storage {
        Some(v) => format!("{v}% {}", bar(v as f64)),
        None => "-- ----------".into(),
    };

    let disk = match &activity.disk_activity {
        Some(v) => format!("DISK R {:.1} · W {:.1} MiB/s", v.read_mib_s, v.write_mib_s),
        None => "DISK R -- · W -- MiB/s".into(),
    };

    let uptime = match &cheap.uptime {
        Some(v) if v.days > 0 => {
            format!("{}d{}h{}m", v.days, v.hours, v.minutes)
        }
        Some(v) if v.hours > 0 => {
            format!("{}h{}m", v.hours, v.minutes)
        }
        Some(v) => format!("{}m", v.minutes),
        None => "--".into(),
    };

    let copyparty = match snapshot.copyparty {
        CpStatus::Running => "COPY OK",
        CpStatus::Stopped => "COPY STOP",
        CpStatus::Failed => "COPY FAIL",
        CpStatus::Unknown => "COPY ?",
    };

    let backup = match snapshot.backup {
        BackupStatus::Current { age_hours } => {
            format!("BACKUP {age_hours}h ago")
        }
        BackupStatus::Stale { age_hours } => {
            format!("BACKUP STALE {age_hours}h")
        }
        BackupStatus::Unavailable => "BACKUP N/A".into(),
        BackupStatus::Checking => "BACKUP Checking...".into(),
    };

    [
        format!("┌{}┐", "─".repeat(INNER_WIDTH)),
        head(pupils),
        row(&format!("  TEMP {temperature:<20} CPU {cpu}")),
        row(&format!("  RAM  {ram:<19} STORE {storage}")),
        row(&format!("  {disk}")),
        row(&format!("  UP {uptime:<12} {copyparty:<14} {backup}")),
        format!("└{}┘", "─".repeat(INNER_WIDTH)),
    ]
    .join("\n")
}
