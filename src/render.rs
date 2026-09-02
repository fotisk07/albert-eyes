use crate::animation::{DayPhase, EyeState, FacePose, Mouth};
use crate::status::{AlbertStatus, BackupStatus, DiskAvailability, DiskHealth, DiskStatus};

const CARD_WIDTH: usize = 55;
const INNER_WIDTH: usize = CARD_WIDTH - 2;
const BAR_WIDTH: usize = 10;
const STATUS_WIDTH: usize = 37;
const FACE_WIDTH: usize = 45;
const FACE_HEIGHT: usize = 7;
const SMALL_EYE_INNER_WIDTH: usize = 7;
const SMALL_EYE_SEPARATION: usize = 5;
const LARGE_EYE_INNER_WIDTH: usize = 9;
const LARGE_EYE_SEPARATION: usize = 9;

fn row(content: &str) -> String {
    let content: String = content.chars().take(INNER_WIDTH).collect();
    format!("│{content:<INNER_WIDTH$}│")
}

fn centered_row(content: &str) -> String {
    let width = content.chars().count();
    let padding = INNER_WIDTH.saturating_sub(width) / 2;
    row(&format!("{}{content}", " ".repeat(padding)))
}

fn status_row(content: &str) -> String {
    centered_row(&format!("{content:<STATUS_WIDTH$}"))
}

fn eye_rows(pose: FacePose, inner_width: usize, height: usize) -> Vec<String> {
    let eye_width = inner_width + 2;
    let mut rows = vec![" ".repeat(eye_width); height];
    let middle = height / 2;

    match pose.eyes {
        EyeState::Open => {
            rows[0] = format!("╭{}╮", "─".repeat(inner_width));
            rows[height - 1] = format!("╰{}╯", "─".repeat(inner_width));
            for row in &mut rows[1..height - 1] {
                *row = format!("│{}│", " ".repeat(inner_width));
            }

            let scaled_pupil = (pose.pupil_position * (inner_width - 1)
                + (SMALL_EYE_INNER_WIDTH - 1) / 2)
                / (SMALL_EYE_INNER_WIDTH - 1);
            let pupil: String = (0..inner_width)
                .map(|position| if position == scaled_pupil { '●' } else { ' ' })
                .collect();
            rows[middle] = format!("│{pupil}│");
        }
        EyeState::Closed => {
            let eyelid = if pose.phase == DayPhase::Night && pose.scene_frame == 2 {
                format!(" ╰{}╯ ", "─".repeat(inner_width - 2))
            } else {
                format!("╰{}╯", "─".repeat(inner_width))
            };
            rows[middle] = eyelid;
        }
    }

    rows
}

fn put(scene: &mut [Vec<char>], x: usize, y: usize, content: &str) {
    let Some(line) = scene.get_mut(y) else {
        return;
    };

    for (offset, character) in content.chars().enumerate() {
        if let Some(cell) = line.get_mut(x + offset) {
            *cell = character;
        }
    }
}

fn draw_face(
    scene: &mut [Vec<char>],
    x: usize,
    y: usize,
    pose: FacePose,
    eye_inner_width: usize,
    eye_separation: usize,
    eye_height: usize,
) {
    let y = y + pose.vertical_offset;
    let eye_rows = eye_rows(pose, eye_inner_width, eye_height);
    let eye_width = eye_inner_width + 2;
    let right_eye_x = x + eye_width + eye_separation;

    for (offset, eye_row) in eye_rows.iter().enumerate() {
        put(scene, x, y + offset, eye_row);
        put(scene, right_eye_x, y + offset, eye_row);
    }

    let mouth = match pose.mouth {
        Mouth::Smile => "╰───╯",
        Mouth::Relaxed => "╰─╯",
        Mouth::SmallO => "o",
        Mouth::Yawn => "◯",
        Mouth::Sleeping => "~",
        Mouth::SleepingOpen => "o",
        Mouth::Hidden => "",
    };
    let face_width = eye_width * 2 + eye_separation;
    let mouth_width = mouth.chars().count();
    let mouth_x = x + (face_width.saturating_sub(mouth_width)) / 2;

    put(scene, mouth_x, y + eye_height, mouth);
}

fn draw_mug(scene: &mut [Vec<char>], x: usize, y: usize) {
    if y >= 2 {
        put(scene, x + 2, y - 2, "( (");
        put(scene, x + 3, y - 1, ") )");
    }
    put(scene, x, y, "╭────╮");
    put(scene, x, y + 1, "│    ├╮");
    put(scene, x, y + 2, "╰────╯");
}

fn draw_morning(scene: &mut [Vec<char>], pose: FacePose) {
    draw_face(
        scene,
        7,
        1,
        pose,
        SMALL_EYE_INNER_WIDTH,
        SMALL_EYE_SEPARATION,
        3,
    );

    match pose.scene_frame {
        1 => draw_mug(scene, 29, 3),
        2 => draw_mug(scene, 15, 4),
        _ => draw_mug(scene, 34, 2),
    }
}

fn draw_day(scene: &mut [Vec<char>], pose: FacePose) {
    draw_face(
        scene,
        7,
        0,
        pose,
        LARGE_EYE_INNER_WIDTH,
        LARGE_EYE_SEPARATION,
        5,
    );
}

fn draw_evening(scene: &mut [Vec<char>], pose: FacePose) {
    draw_face(
        scene,
        11,
        1,
        pose,
        SMALL_EYE_INNER_WIDTH,
        SMALL_EYE_SEPARATION,
        4,
    );
}

fn draw_night(scene: &mut [Vec<char>], pose: FacePose) {
    draw_face(
        scene,
        11,
        2,
        pose,
        SMALL_EYE_INNER_WIDTH,
        SMALL_EYE_SEPARATION,
        3,
    );
    put(scene, 12, 0, "╭────────────────────╮");
    put(scene, 11, 1, "╱        ✦             ╰───◯");
    put(scene, 10, 2, "╰───────────────────────────╯");

    match pose.scene_frame {
        1 => {
            put(scene, 41, 4, "Z");
            put(scene, 43, 3, "z");
            put(scene, 44, 2, "z");
        }
        2 => {
            put(scene, 39, 3, "Z");
            put(scene, 42, 2, "z");
            put(scene, 44, 1, "z");
        }
        _ => {}
    }
}

fn face(pose: FacePose) -> String {
    let mut scene = vec![vec![' '; FACE_WIDTH]; FACE_HEIGHT];

    match pose.phase {
        DayPhase::Morning => draw_morning(&mut scene, pose),
        DayPhase::Day => draw_day(&mut scene, pose),
        DayPhase::Evening => draw_evening(&mut scene, pose),
        DayPhase::Night => draw_night(&mut scene, pose),
    }

    scene
        .into_iter()
        .map(|line| centered_row(&line.into_iter().collect::<String>()))
        .collect::<Vec<_>>()
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

            status_row(&format!("{name:<5} {temperature:<5} {health}   {capacity}"))
        }
        DiskAvailability::Unmounted => status_row(&format!("{name:<5} UNMOUNTED")),
        DiskAvailability::Missing => status_row(&format!("{name:<5} MISSING")),
        DiskAvailability::Unknown => status_row(&format!("{name:<5} UNKNOWN")),
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

pub fn render(status: &AlbertStatus, pose: FacePose) -> String {
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
        face(pose),
        disk_row("AL", &status.al),
        disk_row("BERT", &status.bert),
        status_row(&format!("PI    {pi_temperature:<5} · RAM {ram}")),
        centered_row(&backups),
        format!("└{}┘", "─".repeat(INNER_WIDTH)),
    ]
    .join("\n")
}
