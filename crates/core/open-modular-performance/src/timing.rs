use std::time::{
    Duration,
    Instant,
};

use crossbeam::channel::{
    self,
    Receiver,
    Sender,
};
use derive_more::with_trait::Debug;
use fancy_constructor::new;
use tracing::{
    Level,
    info,
    instrument,
    warn,
};

// =================================================================================================
// Timing
// =================================================================================================

// Data

#[derive(new, Debug)]
pub struct Timing {
    duration: Duration,
    iterations: u32,
    #[new(into)]
    name: String,
}

// -------------------------------------------------------------------------------------------------

// Collector

#[derive(new, Debug)]
pub struct TimingCollector {
    #[new(default)]
    accumulator: Duration,
    #[new(val = Instant::now())]
    initiator: Instant,
    #[new(into)]
    name: String,
    reporting_interval: u32,
    #[new(default)]
    reporting_iteration: u32,
    sample_interval: u32,
    #[new(default)]
    sample_iteration: u32,
    #[debug(skip)]
    sender: Sender<Timing>,
}

impl TimingCollector {
    #[instrument(level = Level::TRACE)]
    pub fn enter(&mut self) {
        if self.sample_iteration == self.sample_interval - 1 {
            self.sample_iteration = 0;
            self.initiator = Instant::now();
        } else {
            self.sample_iteration += 1;
        }
    }

    #[instrument(level = Level::TRACE)]
    pub fn exit(&mut self) {
        if self.sample_iteration == 0 {
            self.accumulator += self.initiator.elapsed();

            if self.reporting_iteration == self.reporting_interval - 1 {
                let timing = Timing::new(self.accumulator, self.reporting_interval, &self.name);

                if let Err(err) = self.sender.try_send(timing) {
                    warn!(?err, name = self.name, "timing data send error");
                }

                self.accumulator = Duration::default();
                self.reporting_iteration = 0;
            } else {
                self.reporting_iteration += 1;
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Aggregator

#[derive(new, Debug)]
pub struct TimingAggregator {
    #[debug(skip)]
    #[new(val = channel::unbounded())]
    channels: (Sender<Timing>, Receiver<Timing>),
    #[new(val = Duration::from_secs(1))]
    timeout: Duration,
}

impl TimingAggregator {
    #[instrument(level = Level::DEBUG, skip(self, name), ret)]
    pub fn collector(
        &self,
        name: impl Into<String>,
        reporting_interval: u32,
        sampling_interval: u32,
    ) -> TimingCollector {
        TimingCollector::new(
            name,
            reporting_interval,
            sampling_interval,
            self.channels.0.clone(),
        )
    }
}

impl TimingAggregator {
    #[instrument(level = Level::TRACE, skip(self))]
    pub fn process(&self) {
        if let Ok(timing) = self.channels.1.recv_timeout(self.timeout) {
            let average = timing.duration / timing.iterations;

            info!(?average, name = timing.name, "calculated timing");
        }
    }
}

impl Default for TimingAggregator {
    fn default() -> Self {
        Self::new()
    }
}
