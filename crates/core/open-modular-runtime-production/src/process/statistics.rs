use std::fmt::Debug;

use fancy_constructor::new;
use open_modular_performance::timing::TimingAggregator;
use open_modular_synchronization::control::Exit;
use tracing::{
    Level,
    debug,
    instrument,
};

use crate::runtime::Runtime;

// =================================================================================================
// Control
// =================================================================================================

#[derive(new, Debug)]
#[new(args(runtime: &'rt Runtime), vis())]
pub struct Statistics<'rt> {
    #[new(val = runtime.exit.clone())]
    exit: Exit,
    #[new(val = &runtime.timing_aggregator)]
    timing: &'rt TimingAggregator,
}

impl<'rt> Statistics<'rt> {
    #[instrument(level = Level::DEBUG, skip(runtime))]
    pub fn spawn(runtime: &'rt Runtime) {
        Self::new(runtime).process();
    }
}

impl Statistics<'_> {
    #[instrument(level = Level::DEBUG, skip(self))]
    fn process(&mut self) {
        loop {
            if self.exit.triggered() {
                debug!(action = "break", sync = "exit");

                break;
            }

            self.timing.process();
        }
    }
}
