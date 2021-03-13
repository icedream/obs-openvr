use std::{
    time::{
        Instant,
        Duration,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    start: Instant,
    checkpoint: Option<Instant>
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            start: Instant::now(),
            checkpoint: None,
        }
    }

    #[inline(always)]
    pub fn start_time(&self) -> Instant {
        self.start
    }

    pub fn last_checkpoint(&self) -> Instant {
        self.checkpoint.unwrap_or(self.start)
    }

    pub fn elapsed_checkpoint(&self) -> Duration {
        Instant::now().duration_since(self.last_checkpoint())
    }

    pub fn checkpoint(&mut self) -> Duration {
        let ret = self.elapsed_checkpoint();
        self.checkpoint = Some(Instant::now());
        ret
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now().duration_since(self.start)
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.elapsed().as_millis()
    }
}

pub trait TimerExt {
    fn log_checkpoint_ms<Name: AsRef<str>>(&mut self, checkpoint_name: Name);
}

impl TimerExt for Timer {
    fn log_checkpoint_ms<Name: AsRef<str>>(&mut self, checkpoint_name: Name) {
        info!("{} took {}ms", checkpoint_name.as_ref(), self.checkpoint().as_millis());
    }
}
