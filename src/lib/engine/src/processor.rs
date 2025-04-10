use std::{
    cell::SyncUnsafeCell,
    fmt::Debug,
};

use indexmap::IndexMap;
use tracing::{
    instrument,
    trace,
};
use uuid::Uuid;

use crate::{
    module::{
        Module,
        ModuleReference,
    },
    port::{
        PortConnect as _,
        PortDisconnect as _,
        PortInputGet as _,
        PortInputReference,
        PortOutputGet as _,
        PortOutputReference,
    },
};

// =================================================================================================
// Processor
// =================================================================================================

#[derive(Debug)]
pub struct Processor<M>
where
    M: Module,
{
    args: ProcessArgs,
    modules: IndexMap<Uuid, SyncUnsafeCell<M>>,
}

impl<M> Processor<M>
where
    M: Debug + Module,
{
    #[instrument(level = "debug", skip(self))]
    pub fn add(&mut self, key: Uuid, module: M) -> ModuleReference {
        trace!(?module, "adding module");

        let module = SyncUnsafeCell::new(module);

        self.modules.insert(key, module);

        ModuleReference::new(key)
    }

    pub fn remove(&mut self, module_ref: &ModuleReference) {
        self.modules.swap_remove(&module_ref.instance);
    }
}

impl<M> Processor<M>
where
    M: Module,
{
    /// Connects two currently disconnected ports, an output to an input.
    ///
    /// # Panics
    ///
    /// Panics if either of the two ports cannot be found (either the instance
    /// or the port index). Panics if either of the two ports is not currently
    /// disconnected.
    ///
    /// # Safety
    ///
    /// .
    pub unsafe fn connect(
        &mut self,
        output_ref: &PortOutputReference,
        input_ref: &PortInputReference,
    ) {
        let outputs = self
            .modules
            .get(&output_ref.instance)
            .map(|instance| unsafe { (*instance.get()).as_mut() })
            .expect("output instance to exist");

        let inputs = self
            .modules
            .get(&input_ref.instance)
            .map(|instance| unsafe { (*instance.get()).as_ref() })
            .expect("input instance to exist");

        let output = outputs.port(output_ref.port).expect("output port to exist");
        let input = inputs.port(input_ref.port).expect("input port to exist");

        unsafe {
            output.connect(input);
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics when...
    ///
    /// # Safety
    ///
    /// .
    pub unsafe fn disconnect(&mut self, input_ref: &PortInputReference) {
        let inputs = self
            .modules
            .get(&input_ref.instance)
            .map(|instance| unsafe { (*instance.get()).as_ref() })
            .expect("input instance to exist");

        let input = inputs.port(input_ref.port).expect("input port to exist");

        unsafe {
            input.disconnect();
        }
    }
}

impl<M> Processor<M>
where
    M: Module,
{
    pub fn process(&mut self, iteration: u64) {
        self.args.token.0 = (iteration % 2) as usize;
        self.modules.values().for_each(|module| unsafe {
            (*module.get()).process(&self.args);
        });
    }
}

impl<M> Default for Processor<M>
where
    M: Module,
{
    fn default() -> Self {
        Self {
            args: ProcessArgs::default(),
            modules: IndexMap::default(),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Process

pub trait Process {
    fn process(&mut self, args: &ProcessArgs);
}

#[derive(Debug, Default)]
pub struct ProcessArgs {
    pub token: ProcessToken,
}

#[derive(Debug, Default)]
pub struct ProcessToken(pub(crate) usize);
