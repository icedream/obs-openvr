use std::{
    time::{
        Instant,
        Duration,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct Timer(Instant);

impl Timer {
    pub fn new() -> Self {
        Timer(Instant::now())
    }

    #[inline(always)]
    pub fn start_time(&self) -> Instant {
        self.0
    }

    pub fn reset(&mut self) {
        self.0 = Instant::now();
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now().duration_since(self.0)
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.elapsed().as_millis()
    }
}
