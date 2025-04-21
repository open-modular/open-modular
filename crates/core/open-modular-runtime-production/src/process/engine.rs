use std::fmt::Debug;

use crossbeam::channel::Receiver;
use fancy_constructor::new;
use open_modular_engine::processor::{
    Processor,
    ProcessorProtocol,
};
// #[cfg(feature = "perf")]
// use open_modular_performance::timing::TimingCollector;
use open_modular_runtime::{
    process::{
        Process,
        ProcessControl,
    },
    runtime,
};
use open_modular_synchronization::{
    barrier::Barriers,
    control::Exit,
};

use crate::runtime_old::{
    Runtime,
    RuntimeModule,
};

// =================================================================================================
// Compute
// =================================================================================================

#[derive(new, Debug)]
#[new(args(runtime: &'rt Runtime), vis())]
pub struct Engine<'rt, M>
where
    M: RuntimeModule,
{
    barriers: Barriers,
    #[new(val = &runtime.context)]
    context: &'rt <Runtime as runtime::Runtime>::Context,
    #[new(val = runtime.exit.clone())]
    exit: Exit,
    #[new(default)]
    iteration: u64,
    #[new(default)]
    processor: Processor<M>,
    processor_receiver: Receiver<ProcessorProtocol>,
    // #[cfg(feature = "perf")]
    // #[new(val = runtime.timing_aggregator.collector("compute/compute", 1500))]
    // timing_collector: TimingCollector,
}

impl<'rt, M> Engine<'rt, M>
where
    M: RuntimeModule,
{
    pub fn spawn(
        runtime: &'rt Runtime,
        barriers: Barriers,
        processor_receiver: Receiver<ProcessorProtocol>,
    ) {
        Self::new(runtime, barriers, processor_receiver).process();
    }
}

impl<M> AsMut<Barriers> for Engine<'_, M>
where
    M: RuntimeModule,
{
    fn as_mut(&mut self) -> &mut Barriers {
        &mut self.barriers
    }
}

impl<M> Process for Engine<'_, M>
where
    M: RuntimeModule,
{
    fn phase_0(&mut self) -> ProcessControl {
        if self.iteration % 325 == 0 {
            if self.exit.triggered() {
                return ProcessControl::Exit;
            }

            if let Ok(protocol) = self.processor_receiver.try_recv() {
                protocol.apply(self.context, &mut self.processor);
            }
        }

        ProcessControl::Continue
    }

    fn phase_1(&mut self) {
        // #[cfg(feature = "perf")]
        // self.timing_collector.enter();

        // self.processor.process(self.iteration);
        self.iteration += 1;

        // #[cfg(feature = "perf")]
        // self.timing_collector.exit();
    }
}
