use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct PupilState {
    pub position: usize,
    pub blinking: bool,
}

pub struct IdleAnimation {
    pupils: PupilState,
    action: IdleAction,
}

enum IdleAction {
    Moving { target: usize },
    Dwelling { until: Instant },
    Blinking { until: Instant },
}

impl IdleAnimation {
    pub fn new() -> Self {
        Self {
            pupils: PupilState {
                position: 4,
                blinking: false,
            },
            action: IdleAction::Moving { target: 4 },
        }
    }

    pub fn pupils(&self) -> PupilState {
        self.pupils
    }

    fn choose_action(&mut self) -> IdleAction {
        let roll = rand::random_range(0..10);
        let now = Instant::now();

        match roll {
            0..5 => IdleAction::Moving { target: 4 }, // 50%
            5..7 => IdleAction::Moving { target: 1 }, // 20%
            7..9 => IdleAction::Moving { target: 7 }, // 20%
            9 => {
                self.pupils.blinking = true;
                IdleAction::Blinking {
                    until: now + Duration::from_millis(200),
                }
            } // 10%
            _ => unreachable!(),
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();

        match self.action {
            IdleAction::Moving { target } => {
                // keep moving
                self.pupils.position = match self.pupils.position {
                    n if n < target => n + 1,
                    n if n > target => n - 1,
                    n => n,
                };

                // if target reached then dwell
                if self.pupils.position == target {
                    let dwell_duration = Duration::from_millis(rand::random_range(800..2500));
                    self.action = IdleAction::Dwelling {
                        until: now + dwell_duration,
                    }
                }
            }

            IdleAction::Dwelling { until } => {
                // when dwelling expires make random decision
                if now >= until {
                    self.action = self.choose_action();
                }
            }

            IdleAction::Blinking { until } => {
                // when done blinking dwell a bit
                if now >= until {
                    self.pupils.blinking = false;
                    let dwell_duration = Duration::from_millis(rand::random_range(200..1500));
                    self.action = IdleAction::Dwelling {
                        until: now + dwell_duration,
                    }
                }
            }
        }
    }
}
