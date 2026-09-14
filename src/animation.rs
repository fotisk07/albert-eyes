use crate::status::{AlbertStatus, BackupStatus};
use chrono::{Local, Timelike};
use std::env;
use std::time::{Duration, Instant};

const PUPIL_CENTER: usize = 3;
const PUPIL_LEFT: usize = 1;
const PUPIL_RIGHT: usize = 5;

#[derive(Clone, Copy)]
pub enum Mood {
    Normal,
    Excited,
    Mad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayPhase {
    Morning,
    Day,
    Evening,
    Night,
}

impl DayPhase {
    fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "morning" => Some(Self::Morning),
            "day" => Some(Self::Day),
            "evening" => Some(Self::Evening),
            "night" => Some(Self::Night),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EyeState {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mouth {
    Smile,
    Happy,
    Relaxed,
    SmallO,
    Yawn,
    Sleeping,
    SleepingOpen,
    Hidden,
}

#[derive(Clone, Copy)]
pub struct FacePose {
    pub phase: DayPhase,
    pub pupil_position: usize,
    pub eyes: EyeState,
    pub mouth: Mouth,
    pub vertical_offset: usize,
    pub scene_frame: u8,
    pub mood: Mood,
}

pub struct Animator {
    phase: DayPhase,
    pupil_position: usize,
    excited_scene_frame: u8,
    excited_vertical_offset: usize,
    action: Action,
    phase_override: Option<DayPhase>,
    backup_running: bool,
}

#[derive(Clone, Copy)]
enum Action {
    Moving {
        target: usize,
    },
    Dwelling {
        until: Instant,
    },
    Blinking {
        until: Instant,
    },
    Sequence {
        kind: SequenceKind,
        frame: u8,
        until: Instant,
    },
    Mad {
        until: Instant,
    },
}

#[derive(Clone, Copy)]
enum SequenceKind {
    Sip,
    Yawn,
    Nod,
    Sleep,
}
impl SequenceKind {
    fn frame_count(self) -> u8 {
        match self {
            Self::Sip => 5,
            Self::Yawn | Self::Nod | Self::Sleep => 4,
        }
    }

    fn frame_duration(self, frame: u8) -> Duration {
        let milliseconds = match (self, frame) {
            (Self::Sip, 0) => 600,
            (Self::Sip, 1 | 3) => 300,
            (Self::Sip, 2) => 900,
            (Self::Sip, _) => 400,
            (Self::Yawn, 0 | 3) => 350,
            (Self::Yawn, 1) => 500,
            (Self::Yawn, _) => 900,
            (Self::Nod, 0 | 3) => 500,
            (Self::Nod, _) => 1_000,
            (Self::Sleep, 0 | 3) => 1_200,
            (Self::Sleep, _) => 1_500,
        };

        Duration::from_millis(milliseconds)
    }

    fn apply_frame(self, frame: u8, pose: &mut FacePose) {
        match (self, frame) {
            (Self::Sip, 0) => {
                pose.pupil_position = PUPIL_RIGHT;
                pose.eyes = EyeState::Open;
                pose.mouth = Mouth::Relaxed;
                pose.scene_frame = 0;
            }
            (Self::Sip, 1) => pose.scene_frame = 1,
            (Self::Sip, 2) => {
                pose.eyes = EyeState::Closed;
                pose.mouth = Mouth::Hidden;
                pose.scene_frame = 2;
            }
            (Self::Sip, 3) => {
                pose.eyes = EyeState::Open;
                pose.mouth = Mouth::Relaxed;
                pose.scene_frame = 1;
            }
            (Self::Sip, 4) => pose.scene_frame = 0,
            (Self::Yawn, 0) => pose.mouth = Mouth::SmallO,
            (Self::Yawn, 1 | 2) => {
                pose.eyes = EyeState::Closed;
                pose.mouth = Mouth::Yawn;
            }
            (Self::Yawn, 3) => {
                pose.eyes = EyeState::Open;
                pose.mouth = Mouth::SmallO;
            }
            (Self::Nod, 0) => pose.eyes = EyeState::Closed,
            (Self::Nod, 1 | 2) => {
                pose.eyes = EyeState::Closed;
                pose.vertical_offset = 1;
            }
            (Self::Nod, 3) => {
                pose.eyes = EyeState::Open;
                pose.vertical_offset = 0;
            }
            (Self::Sleep, 0) => {
                pose.eyes = EyeState::Closed;
                pose.mouth = Mouth::Sleeping;
                pose.vertical_offset = 0;
                pose.scene_frame = 0;
            }
            (Self::Sleep, 1) => {
                pose.mouth = Mouth::SleepingOpen;
                pose.scene_frame = 1;
            }
            (Self::Sleep, 2) => {
                pose.mouth = Mouth::SleepingOpen;
                pose.scene_frame = 2;
            }
            (Self::Sleep, 3) => {
                pose.mouth = Mouth::Sleeping;
                pose.scene_frame = 0;
            }
            _ => {}
        }
    }
}

impl Animator {
    pub fn new() -> Self {
        let phase_override = env::var("ALBERT_EYES_PHASE")
            .ok()
            .and_then(|name| DayPhase::from_name(&name));
        let phase = phase_override.unwrap_or_else(current_phase);
        let now = Instant::now();

        Self {
            phase,
            pupil_position: PUPIL_CENTER,
            excited_scene_frame: 0,
            excited_vertical_offset: 0,
            action: Action::Dwelling {
                until: now + random_dwell(phase),
            },
            phase_override,
            backup_running: false,
        }
    }

    pub fn pose(&self) -> FacePose {
        let mut pose = default_pose(self.phase);
        pose.pupil_position = self.pupil_position;

        if self.backup_running {
            pose.eyes = EyeState::Open;
            pose.mouth = Mouth::Happy;
            pose.mood = Mood::Excited;
            pose.scene_frame = self.excited_scene_frame;
            pose.vertical_offset = self.excited_vertical_offset;
        }

        match self.action {
            Action::Blinking { .. } => pose.eyes = EyeState::Closed,
            Action::Sequence { kind, frame, .. } => {
                for current_frame in 0..=frame {
                    kind.apply_frame(current_frame, &mut pose);
                }
            }
            Action::Mad { .. } => pose.mood = Mood::Mad,
            Action::Moving { .. } | Action::Dwelling { .. } => {}
        }
        pose
    }

    pub fn touched(&mut self) {
        self.action = Action::Mad {
            until: Instant::now() + Duration::from_secs(1),
        };
    }

    pub fn update(&mut self, status: &AlbertStatus) {
        let now = Instant::now();
        let phase = self.phase_override.unwrap_or_else(current_phase);
        let backup_running = has_running_backup(status);

        if phase != self.phase || backup_running != self.backup_running {
            self.phase = phase;
            self.backup_running = backup_running;
            self.pupil_position = PUPIL_CENTER;
            self.excited_scene_frame = 0;
            self.excited_vertical_offset = 0;
            self.start_dwelling(now);
            return;
        }

        match self.action {
            Action::Moving { target } => self.update_movement(target, now),
            Action::Dwelling { until } if now >= until => {
                self.action = self.choose_action(now);
            }
            Action::Blinking { until } if now >= until => {
                self.start_dwelling(now);
            }
            Action::Sequence { kind, frame, until } if now >= until => {
                self.advance_sequence(kind, frame, now)
            }
            Action::Dwelling { .. } | Action::Blinking { .. } | Action::Sequence { .. } => {}
            Action::Mad { until } if now >= until => {
                self.start_dwelling(now);
            }
            Action::Mad { .. } => {}
        }
    }

    fn update_movement(&mut self, target: usize, now: Instant) {
        self.pupil_position = match self.pupil_position {
            position if position < target => position + 1,
            position if position > target => position - 1,
            position => position,
        };

        if self.pupil_position == target {
            self.start_dwelling(now);
        }
    }

    fn choose_action(&mut self, now: Instant) -> Action {
        if self.backup_running {
            self.excited_scene_frame = (self.excited_scene_frame + 1) % 2;
            self.excited_vertical_offset = rand::random_range(0..=1);

            return match rand::random_range(0..10) {
                0..3 => Action::Moving { target: PUPIL_LEFT },
                3..6 => Action::Moving {
                    target: PUPIL_RIGHT,
                },
                6..9 => Action::Moving {
                    target: PUPIL_CENTER,
                },
                9 => self.start_blink(now, 150),
                _ => unreachable!(),
            };
        }

        match self.phase {
            DayPhase::Morning => match rand::random_range(0..12) {
                0..4 => Action::Moving {
                    target: PUPIL_CENTER,
                },
                4..6 => Action::Moving { target: PUPIL_LEFT },
                6..8 => Action::Moving {
                    target: PUPIL_RIGHT,
                },
                8..10 => self.start_blink(now, 300),
                10..12 => self.start_sequence(SequenceKind::Sip, now),
                _ => unreachable!(),
            },
            DayPhase::Day => match rand::random_range(0..10) {
                0..5 => Action::Moving {
                    target: PUPIL_CENTER,
                },
                5..7 => Action::Moving { target: PUPIL_LEFT },
                7..9 => Action::Moving {
                    target: PUPIL_RIGHT,
                },
                9 => self.start_blink(now, 200),
                _ => unreachable!(),
            },
            DayPhase::Evening => match rand::random_range(0..14) {
                0..4 => Action::Moving {
                    target: PUPIL_CENTER,
                },
                4 => Action::Moving { target: PUPIL_LEFT },
                5 => Action::Moving {
                    target: PUPIL_RIGHT,
                },
                6..9 => self.start_blink(now, 450),
                9..13 => self.start_sequence(SequenceKind::Yawn, now),
                13 => self.start_sequence(SequenceKind::Nod, now),
                _ => unreachable!(),
            },
            DayPhase::Night => self.start_sequence(SequenceKind::Sleep, now),
        }
    }

    fn start_blink(&mut self, now: Instant, duration_millis: u64) -> Action {
        Action::Blinking {
            until: now + Duration::from_millis(duration_millis),
        }
    }

    fn start_sequence(&mut self, kind: SequenceKind, now: Instant) -> Action {
        Action::Sequence {
            kind,
            frame: 0,
            until: now + kind.frame_duration(0),
        }
    }

    fn advance_sequence(&mut self, kind: SequenceKind, frame: u8, now: Instant) {
        let next_frame = frame + 1;

        if next_frame >= kind.frame_count() {
            self.start_dwelling(now);
            return;
        }

        self.action = Action::Sequence {
            kind,
            frame: next_frame,
            until: now + kind.frame_duration(next_frame),
        };
    }

    fn start_dwelling(&mut self, now: Instant) {
        self.action = Action::Dwelling {
            until: now + self.random_dwell(),
        };
    }

    fn random_dwell(&self) -> Duration {
        if self.backup_running {
            Duration::from_millis(rand::random_range(200..600))
        } else {
            random_dwell(self.phase)
        }
    }
}

fn default_pose(phase: DayPhase) -> FacePose {
    let (eyes, mouth) = match phase {
        DayPhase::Morning => (EyeState::Open, Mouth::Relaxed),
        DayPhase::Day => (EyeState::Open, Mouth::Smile),
        DayPhase::Evening => (EyeState::Open, Mouth::Relaxed),
        DayPhase::Night => (EyeState::Closed, Mouth::Sleeping),
    };

    FacePose {
        phase,
        pupil_position: PUPIL_CENTER,
        eyes,
        mouth,
        vertical_offset: 0,
        scene_frame: 0,
        mood: Mood::Normal,
    }
}

fn has_running_backup(status: &AlbertStatus) -> bool {
    matches!(status.backups.xps_to_al, BackupStatus::Running)
        || matches!(status.backups.xps_to_bert, BackupStatus::Running)
        || matches!(status.backups.al_to_bert, BackupStatus::Running)
}

fn current_phase() -> DayPhase {
    phase_for_hour(Local::now().hour() as u8)
}

fn phase_for_hour(hour: u8) -> DayPhase {
    match hour {
        6..11 => DayPhase::Morning,
        11..18 => DayPhase::Day,
        18..22 => DayPhase::Evening,
        _ => DayPhase::Night,
    }
}

fn random_dwell(phase: DayPhase) -> Duration {
    let milliseconds = match phase {
        DayPhase::Morning => rand::random_range(1_200..3_500),
        DayPhase::Day => rand::random_range(800..2_800),
        DayPhase::Evening => rand::random_range(1_800..4_500),
        DayPhase::Night => rand::random_range(2_000..4_000),
    };

    Duration::from_millis(milliseconds)
}
