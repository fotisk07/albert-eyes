use crate::status::{AlbertStatus, BackupStatus};
use chrono::{Local, Timelike};
use std::env;
use std::time::{Duration, Instant};

const PUPIL_CENTER: usize = 3;
const PUPIL_LEFT: usize = 1;
const PUPIL_RIGHT: usize = 5;

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

#[derive(Clone, Copy, Debug)]
pub struct FacePose {
    pub phase: DayPhase,
    pub pupil_position: usize,
    pub eyes: EyeState,
    pub mouth: Mouth,
    pub vertical_offset: usize,
    pub scene_frame: u8,
    pub excited: bool,
}

pub struct Animator {
    pose: FacePose,
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
}

#[derive(Clone, Copy)]
enum SequenceKind {
    Sip,
    Yawn,
    Nod,
    Sleep,
}

impl Animator {
    pub fn new() -> Self {
        let phase_override = env::var("ALBERT_EYES_PHASE")
            .ok()
            .and_then(|name| DayPhase::from_name(&name));
        let phase = phase_override.unwrap_or_else(current_phase);
        let now = Instant::now();

        Self {
            pose: default_pose(phase),
            action: Action::Dwelling {
                until: now + random_dwell(phase),
            },
            phase_override,
            backup_running: false,
        }
    }

    pub fn pose(&self) -> FacePose {
        self.pose
    }

    pub fn update(&mut self, status: &AlbertStatus) {
        let now = Instant::now();
        let phase = self.phase_override.unwrap_or_else(current_phase);
        let backup_running = has_running_backup(status);

        if phase != self.pose.phase || backup_running != self.backup_running {
            self.backup_running = backup_running;
            self.pose = default_pose(phase);
            self.restore_default_expression();
            self.action = Action::Dwelling {
                until: now + self.random_dwell(),
            };
            return;
        }

        match self.action {
            Action::Moving { target } => self.update_movement(target, now),
            Action::Dwelling { until } if now >= until => {
                self.action = self.choose_action(now);
            }
            Action::Blinking { until } if now >= until => {
                self.restore_default_expression();
                self.action = Action::Dwelling {
                    until: now + self.random_dwell(),
                };
            }
            Action::Sequence { kind, frame, until } if now >= until => {
                self.advance_sequence(kind, frame, now)
            }
            Action::Dwelling { .. } | Action::Blinking { .. } | Action::Sequence { .. } => {}
        }
    }

    fn update_movement(&mut self, target: usize, now: Instant) {
        self.pose.pupil_position = match self.pose.pupil_position {
            position if position < target => position + 1,
            position if position > target => position - 1,
            position => position,
        };

        if self.pose.pupil_position == target {
            self.action = Action::Dwelling {
                until: now + self.random_dwell(),
            };
        }
    }

    fn choose_action(&mut self, now: Instant) -> Action {
        if self.backup_running {
            self.pose.scene_frame = (self.pose.scene_frame + 1) % 2;
            self.pose.vertical_offset = rand::random_range(0..=1);

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

        match self.pose.phase {
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
        self.pose.eyes = EyeState::Closed;
        Action::Blinking {
            until: now + Duration::from_millis(duration_millis),
        }
    }

    fn start_sequence(&mut self, kind: SequenceKind, now: Instant) -> Action {
        self.apply_sequence_frame(kind, 0);
        Action::Sequence {
            kind,
            frame: 0,
            until: now + sequence_frame_duration(kind, 0),
        }
    }

    fn advance_sequence(&mut self, kind: SequenceKind, frame: u8, now: Instant) {
        let next_frame = frame + 1;

        if next_frame >= sequence_frame_count(kind) {
            self.restore_default_expression();
            self.action = Action::Dwelling {
                until: now + self.random_dwell(),
            };
            return;
        }

        self.apply_sequence_frame(kind, next_frame);
        self.action = Action::Sequence {
            kind,
            frame: next_frame,
            until: now + sequence_frame_duration(kind, next_frame),
        };
    }

    fn apply_sequence_frame(&mut self, kind: SequenceKind, frame: u8) {
        match (kind, frame) {
            (SequenceKind::Sip, 0) => {
                self.pose.pupil_position = PUPIL_RIGHT;
                self.pose.eyes = EyeState::Open;
                self.pose.mouth = Mouth::Relaxed;
                self.pose.scene_frame = 0;
            }
            (SequenceKind::Sip, 1) => self.pose.scene_frame = 1,
            (SequenceKind::Sip, 2) => {
                self.pose.eyes = EyeState::Closed;
                self.pose.mouth = Mouth::Hidden;
                self.pose.scene_frame = 2;
            }
            (SequenceKind::Sip, 3) => {
                self.pose.eyes = EyeState::Open;
                self.pose.mouth = Mouth::Relaxed;
                self.pose.scene_frame = 1;
            }
            (SequenceKind::Sip, 4) => self.pose.scene_frame = 0,
            (SequenceKind::Yawn, 0) => self.pose.mouth = Mouth::SmallO,
            (SequenceKind::Yawn, 1 | 2) => {
                self.pose.eyes = EyeState::Closed;
                self.pose.mouth = Mouth::Yawn;
            }
            (SequenceKind::Yawn, 3) => {
                self.pose.eyes = EyeState::Open;
                self.pose.mouth = Mouth::SmallO;
            }
            (SequenceKind::Nod, 0) => self.pose.eyes = EyeState::Closed,
            (SequenceKind::Nod, 1 | 2) => {
                self.pose.eyes = EyeState::Closed;
                self.pose.vertical_offset = 1;
            }
            (SequenceKind::Nod, 3) => {
                self.pose.eyes = EyeState::Open;
                self.pose.vertical_offset = 0;
            }
            (SequenceKind::Sleep, 0) => {
                self.pose.eyes = EyeState::Closed;
                self.pose.mouth = Mouth::Sleeping;
                self.pose.vertical_offset = 0;
                self.pose.scene_frame = 0;
            }
            (SequenceKind::Sleep, 1) => {
                self.pose.mouth = Mouth::SleepingOpen;
                self.pose.scene_frame = 1;
            }
            (SequenceKind::Sleep, 2) => {
                self.pose.mouth = Mouth::SleepingOpen;
                self.pose.scene_frame = 2;
            }
            (SequenceKind::Sleep, 3) => {
                self.pose.mouth = Mouth::Sleeping;
                self.pose.scene_frame = 0;
            }
            _ => {}
        }
    }

    fn restore_default_expression(&mut self) {
        let pupil_position = self.pose.pupil_position;
        self.pose = default_pose(self.pose.phase);

        if self.backup_running {
            self.pose.eyes = EyeState::Open;
            self.pose.mouth = Mouth::Happy;
            self.pose.pupil_position = pupil_position;
            self.pose.excited = true;
        } else if self.pose.phase != DayPhase::Night {
            self.pose.pupil_position = pupil_position;
        }
    }

    fn random_dwell(&self) -> Duration {
        if self.backup_running {
            Duration::from_millis(rand::random_range(200..600))
        } else {
            random_dwell(self.pose.phase)
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
        excited: false,
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

fn sequence_frame_count(kind: SequenceKind) -> u8 {
    match kind {
        SequenceKind::Sip => 5,
        SequenceKind::Yawn | SequenceKind::Nod | SequenceKind::Sleep => 4,
    }
}

fn sequence_frame_duration(kind: SequenceKind, frame: u8) -> Duration {
    let milliseconds = match (kind, frame) {
        (SequenceKind::Sip, 0) => 600,
        (SequenceKind::Sip, 1 | 3) => 300,
        (SequenceKind::Sip, 2) => 900,
        (SequenceKind::Sip, _) => 400,
        (SequenceKind::Yawn, 0 | 3) => 350,
        (SequenceKind::Yawn, 1) => 500,
        (SequenceKind::Yawn, _) => 900,
        (SequenceKind::Nod, 0 | 3) => 500,
        (SequenceKind::Nod, _) => 1_000,
        (SequenceKind::Sleep, 0 | 3) => 1_200,
        (SequenceKind::Sleep, _) => 1_500,
    };

    Duration::from_millis(milliseconds)
}

#[cfg(test)]
mod tests {
    use super::{DayPhase, phase_for_hour};

    #[test]
    fn maps_hours_to_daily_phases() {
        assert_eq!(phase_for_hour(5), DayPhase::Night);
        assert_eq!(phase_for_hour(6), DayPhase::Morning);
        assert_eq!(phase_for_hour(10), DayPhase::Morning);
        assert_eq!(phase_for_hour(11), DayPhase::Day);
        assert_eq!(phase_for_hour(17), DayPhase::Day);
        assert_eq!(phase_for_hour(18), DayPhase::Evening);
        assert_eq!(phase_for_hour(21), DayPhase::Evening);
        assert_eq!(phase_for_hour(22), DayPhase::Night);
    }
}
