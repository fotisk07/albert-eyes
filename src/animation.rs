use std::time::Instant;

#[derive(Clone, Copy)]
pub struct PupilState {
    pub position: usize,
    pub visible: bool,
}

pub struct IdleAnimation {
    pupils: PupilState,
    target: usize,
    last_move: Instant,
}

impl IdleAnimation {
    pub fn new() -> Self {
        Self {
            pupils: PupilState {
                position: 1,
                visible: true,
            },
            target: 5,
            last_move: Instant::now(),
        }
    }

    pub fn pupils(&self) -> PupilState {
        self.pupils
    }

    pub fn update(&mut self) {
        if self.pupils.position == self.target {
            self.target = match self.target {
                5 => 1,
                1 => 5,
                _ => unreachable!(),
            };
        }

        self.pupils.position = match self.pupils.position {
            n if n < self.target => n + 1,
            n if n > self.target => n - 1,
            n => n,
        };

        self.last_move = Instant::now();
    }
}
