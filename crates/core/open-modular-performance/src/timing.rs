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
    #[new(val = Instant::now())]
    initiator: Instant,
    #[new(into)]
    name: String,
    interval: usize,
    #[new(val = Vec::with_capacity(interval))]
    samples: Vec<Duration>,
    #[debug(skip)]
    sender: Sender<Timing>,
}

impl TimingCollector {
    pub fn enter(&mut self) {
        // self.initiator = Instant::now();

        // if self.sample_iteration == self.sample_interval - 1 {
        //     self.sample_iteration = 0;
        //     self.initiator = Instant::now();
        // } else {
        //     self.sample_iteration += 1;
        // }
    }

    pub fn exit(&mut self) {
        // self.samples.push(self.initiator.elapsed());

        // if self.samples.len() >= self.interval {
        //     let min = self.samples.iter().min().unwrap();
        //     let max = self.samples.iter().max().unwrap();

        //     println!("timing for {}. min: {min:?}, max: {max:?}", self.name);

        //     self.samples.clear();
        // }

        // if self.sample_iteration == 0 {
        //     self.accumulator += self.initiator.elapsed();

        //     if self.reporting_iteration == self.reporting_interval - 1 {
        //         let timing = Timing::new(self.accumulator,
        // self.reporting_interval, &self.name);

        //         if let Err(_err) = self.sender.try_send(timing) {
        //             // log::warn!(err:?, name = self.name; "timing data send
        //             // error");
        //         }

        //         self.accumulator = Duration::default();
        //         self.reporting_iteration = 0;
        //     } else {
        //         self.reporting_iteration += 1;
        //     }
        // }
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
    pub fn collector(&self, name: impl Into<String>, interval: usize) -> TimingCollector {
        TimingCollector::new(name, interval, self.channels.0.clone())
    }
}

impl TimingAggregator {
    pub fn process(&self) {
        if let Ok(timing) = self.channels.1.recv_timeout(self.timeout) {
            let average = timing.duration / timing.iterations;

            println!("average {} timing: {average:?}", timing.name);
        }
    }
}

impl Default for TimingAggregator {
    fn default() -> Self {
        Self::new()
    }
}
