use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Time {
    last_frame: Instant,
    delta: Duration,
}

impl Time {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            delta: Duration::ZERO,
        }
    }

    pub fn update(&mut self) {
        let new_instant = Instant::now();
        let delta = self.last_frame.elapsed();

        self.last_frame = new_instant;
        self.delta = delta;
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }
}
