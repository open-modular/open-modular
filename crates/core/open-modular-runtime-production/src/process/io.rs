pub mod audio;

use std::fmt::Debug;

use crossbeam::channel::Receiver;
use fancy_constructor::new;
use open_modular_core::FRAME_DURATION;
// #[cfg(feature = "perf")]
// use open_modular_performance::timing::{
//     TimingAggregator,
//     TimingCollector,
// };
use open_modular_runtime::process::{
    Process,
    ProcessControl,
};
use open_modular_synchronization::{
    barrier::{
        BarrierGroups,
        Barriers,
    },
    control::Exit,
    time::Timer,
};

use crate::{
    process::io::audio::{
        AudioController,
        AudioProtocol,
    },
    runtime_old::Runtime,
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
    #[new(default)]
    iteration: usize,
    // #[cfg(feature = "perf")]
    // #[new(val = &runtime.timing_aggregator)]
    // timing_aggregator: &'rt TimingAggregator,

    // #[cfg(feature = "perf")]
    // #[new(val = runtime.timing_aggregator.collector("io/configure", 1500))]
    // timing_collector: TimingCollector,
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
    fn phase_0(&mut self) -> ProcessControl {
        self.audio_timer.reset();

        // #[cfg(feature = "perf")]
        // self.timing_collector.enter();

        if self.iteration >= 325 {
            if self.exit.triggered() {
                return ProcessControl::Exit;
            }

            self.audio_controller.collect();

            if let Ok(protocol) = self.audio_receiver.try_recv() {
                match protocol {
                    AudioProtocol::GetOutputBuffer(id, output_buffer_value) => {
                        let barriers = self.barrier_groups.barriers();

                        // #[cfg(feature = "perf")]
                        // let timing = {
                        //     let name = format!("io[{id}]/io");
                        //     let collector = self.timing_aggregator.collector(&name, 1500);

                        //     Some(collector)
                        // };

                        // #[cfg(not(feature = "perf"))]
                        // let timing = None;

                        let output_buffer = self.audio_controller.output_buffer(barriers, id);

                        output_buffer_value.set(output_buffer);
                    }
                    AudioProtocol::GetOutputs(outputs_value) => {
                        let outputs = self.audio_controller.outputs();

                        outputs_value.set(outputs);
                    }
                }
            }

            self.iteration = 0;
        } else {
            self.iteration += 1;
        }

        // #[cfg(feature = "perf")]
        // self.timing_collector.exit();

        ProcessControl::Continue
    }

    fn phase_2(&mut self) {
        if !self.audio_controller.is_active() {
            self.audio_timer.wait();
        }
    }
}
