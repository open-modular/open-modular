use std::{
    fmt::Debug,
    marker::PhantomData,
    thread,
    time::Duration,
};

use fancy_constructor::new;
use open_modular_synchronization::control::Exit;

use crate::runtime::Runtime;

// =================================================================================================
// Control
// =================================================================================================

#[derive(new, Debug)]
#[new(args(runtime: &'rt Runtime), vis())]
pub struct Control<'rt> {
    #[new(val = runtime.exit.clone())]
    exit: Exit,
    #[new(default)]
    _rt: PhantomData<&'rt ()>,
}

impl<'rt> Control<'rt> {
    pub fn spawn(runtime: &'rt Runtime) {
        Self::new(runtime).process();
    }
}

impl Control<'_> {
    fn process(&mut self) {
        thread::sleep(Duration::from_secs(5));

        self.exit.trigger();
    }
}
