use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct PupilState {
    pub position: usize,
    pub blinking: bool,
}

pub struct IdleAnimation {
    pupils: PupilState,
    target: usize,
    action: IdleAction,
}

enum IdleAction {
    Moving,
    Dwelling { until: Instant },
    Blinking { until: Instant },
}

impl IdleAnimation {
    pub fn new() -> Self {
        Self {
            pupils: PupilState {
                position: 1,
                blinking: false,
            },
            target: 5,
            action: IdleAction::Moving,
        }
    }

    pub fn pupils(&self) -> PupilState {
        self.pupils
    }

    pub fn update(&mut self) {
        let now = Instant::now();

        match self.action {
            IdleAction::Moving => {
                self.pupils.position = match self.pupils.position {
                    n if n < self.target => n + 1,
                    n if n > self.target => n - 1,
                    n => n,
                };

                if self.pupils.position == self.target {
                    self.target = match self.target {
                        5 => 1,
                        1 => 5,
                        _ => unreachable!(),
                    };

                    self.action = IdleAction::Dwelling {
                        until: now + Duration::from_millis(1000),
                    };
                }
            }

            IdleAction::Dwelling { until } => {
                if now >= until {
                    self.pupils.blinking = true;

                    self.action = IdleAction::Blinking {
                        until: now + Duration::from_millis(500),
                    };
                }
            }

            IdleAction::Blinking { until } => {
                if now >= until {
                    self.pupils.blinking = false;
                    self.action = IdleAction::Moving;
                }
            }
        }
    }
}
