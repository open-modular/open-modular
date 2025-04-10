use std::{
    fmt::Debug,
    str::FromStr as _,
    sync::Once,
};

use fancy_constructor::new;
use open_modular_engine::processor::Processor;
#[cfg(feature = "perf")]
use open_modular_performance::timing::TimingCollector;
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
use tracing::{
    Level,
    debug,
    info,
    instrument,
};
use uuid::Uuid;

use crate::runtime::{
    Runtime,
    RuntimeModule,
};

// =================================================================================================
// Compute
// =================================================================================================

#[derive(new, Debug)]
#[new(args(runtime: &'rt Runtime), vis())]
pub struct Compute<'rt, M>
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

    #[cfg(feature = "perf")]
    #[new(val = runtime.timing_aggregator.collector("compute/compute", 50, 50))]
    timing_collector: TimingCollector,
}

impl<'rt, M> Compute<'rt, M>
where
    M: RuntimeModule,
{
    pub fn spawn(runtime: &'rt Runtime, barriers: Barriers) {
        Self::new(runtime, barriers).process();
    }
}

impl<M> AsMut<Barriers> for Compute<'_, M>
where
    M: RuntimeModule,
{
    fn as_mut(&mut self) -> &mut Barriers {
        &mut self.barriers
    }
}

static INIT: Once = Once::new();

impl<M> Process for Compute<'_, M>
where
    M: RuntimeModule,
{
    #[instrument(level = Level::TRACE, skip(self), ret)]
    fn configure(&mut self) -> ProcessControl {
        INIT.call_once(|| {
            info!("configuring processor on first run");

            let sine_id = Uuid::from_str("f75487a4-7847-43f9-ab47-71bd6acfb78d").unwrap();
            let sine = M::get(&sine_id, self.context.clone());
            let sine_ref = self.processor.add(sine_id, sine);
            let sine_out_ref = sine_ref.output_ref(0);

            let mult_id = Uuid::from_str("54d93000-7dd2-45ce-a3f1-ad53b0a04fac").unwrap();
            let mult = M::get(&mult_id, self.context.clone());
            let mult_ref = self.processor.add(mult_id, mult);
            let mult_in_ref = mult_ref.input_ref(0);
            let mult_out_l_ref = mult_ref.output_ref(0);
            let mult_out_r_ref = mult_ref.output_ref(1);

            let out_id = Uuid::from_str("47d0fca2-cb58-4011-8a55-31ecd4b184c1").unwrap();
            let out = M::get(&out_id, self.context.clone());
            let out_ref = self.processor.add(out_id, out);
            let out_l_ref = out_ref.input_ref(0);
            let out_r_ref = out_ref.input_ref(1);

            unsafe {
                self.processor.connect(&sine_out_ref, &mult_in_ref);
                self.processor.connect(&mult_out_l_ref, &out_l_ref);
                self.processor.connect(&mult_out_r_ref, &out_r_ref);
            }
        });

        if self.exit.triggered() {
            debug!(action = "break", sync = "exit");

            return ProcessControl::Exit;
        }

        ProcessControl::Continue
    }

    #[instrument(level = Level::TRACE, skip(self))]
    fn compute(&mut self) {
        #[cfg(feature = "perf")]
        self.timing_collector.enter();

        self.processor.process(self.iteration);
        self.iteration += 1;

        #[cfg(feature = "perf")]
        self.timing_collector.exit();
    }
}
