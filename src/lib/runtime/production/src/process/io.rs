pub mod audio;

use std::fmt::Debug;

use crossbeam::channel::Receiver;
use fancy_constructor::new;
use open_modular_core::FRAME_DURATION;
#[cfg(feature = "perf")]
use open_modular_perf::timing::{
    TimingAggregator,
    TimingCollector,
};
use open_modular_runtime::process::{
    Process,
    ProcessControl,
};
use open_modular_sync::{
    barrier::{
        BarrierGroups,
        Barriers,
    },
    control::Exit,
    time::Timer,
};
use tracing::{
    Level,
    debug,
    instrument,
};

use crate::{
    process::io::audio::{
        AudioController,
        AudioProtocol,
    },
    runtime::Runtime,
};

// =================================================================================================
// IO
// =================================================================================================

#[derive(new, Debug)]
#[new(args(runtime: &'rt Runtime), vis())]
pub struct Io<'rt> {
    #[new(default)]
    audio_controller: AudioController,
    #[new(val = &runtime.audio_receiver)]
    audio_receiver: &'rt Receiver<AudioProtocol>,
    #[new(val = Timer::new(FRAME_DURATION))]
    audio_timer: Timer,
    #[new(val = &runtime.barrier_groups)]
    barrier_groups: &'rt BarrierGroups,
    barriers: Barriers,
    #[new(val = runtime.exit.clone())]
    exit: Exit,

    #[cfg(feature = "perf")]
    #[new(val = &runtime.timing_aggregator)]
    timing_aggregator: &'rt TimingAggregator,

    #[cfg(feature = "perf")]
    #[new(val = runtime.timing_aggregator.collector("io/configure", 50, 50))]
    timing_collector: TimingCollector,
}

impl<'rt> Io<'rt> {
    pub fn spawn(runtime: &'rt Runtime, barriers: Barriers) {
        Self::new(runtime, barriers).process();
    }
}

impl AsMut<Barriers> for Io<'_> {
    fn as_mut(&mut self) -> &mut Barriers {
        &mut self.barriers
    }
}

impl Process for Io<'_> {
    #[instrument(level = Level::TRACE, skip(self))]
    fn configure(&mut self) -> ProcessControl {
        self.audio_timer.reset();

        #[cfg(feature = "perf")]
        self.timing_collector.enter();

        if self.exit.triggered() {
            debug!(action = "break", sync = "exit");

            return ProcessControl::Exit;
        }

        self.audio_controller.collect();

        if let Ok(protocol) = self.audio_receiver.try_recv() {
            match protocol {
                AudioProtocol::GetOutputBuffer(id, output_buffer_value) => {
                    debug!(
                        action = "handle",
                        correlation = output_buffer_value.correlation,
                        protocol = "audio",
                        variant = "get_output_buffer",
                        id,
                    );

                    let barriers = self.barrier_groups.barriers();

                    #[cfg(feature = "perf")]
                    let timing = {
                        let name = format!("io[{id}]/io");
                        let collector = self.timing_aggregator.collector(&name, 50, 50);

                        Some(collector)
                    };

                    #[cfg(not(feature = "perf"))]
                    let timing = None;

                    let output_buffer = self.audio_controller.output_buffer(barriers, id, timing);

                    output_buffer_value.set(output_buffer);
                }
                AudioProtocol::GetOutputs(outputs_value) => {
                    debug!(
                        action = "handle",
                        correlation = outputs_value.correlation,
                        protocol = "audio",
                        variant = "get_outputs",
                    );

                    let outputs = self.audio_controller.outputs();

                    outputs_value.set(outputs);
                }
            }
        }

        #[cfg(feature = "perf")]
        self.timing_collector.exit();

        ProcessControl::Continue
    }

    #[instrument(level = Level::TRACE, skip(self))]
    fn io(&mut self) {
        if !self.audio_controller.is_active() {
            self.audio_timer.wait();
        }
    }
}
