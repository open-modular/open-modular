use std::{
    thread,
    time::{
        Duration,
        Instant,
    },
};

use fancy_constructor::new;

// =================================================================================================
// Time
// =================================================================================================

#[derive(new, Clone, Debug)]
pub struct Timer {
    #[new(val = Instant::now() + interval)]
    instant: Instant,
    interval: Duration,
}

impl Timer {
    pub fn reset(&mut self) {
        self.instant = Instant::now() + self.interval;
    }

    pub fn wait(&self) {
        thread::sleep(self.instant - Instant::now());
    }
}
