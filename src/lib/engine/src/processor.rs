use std::{
    cell::SyncUnsafeCell,
    fmt::Debug,
};

use open_modular_utils::collections::IndexMap;
use tracing::{
    instrument,
    trace,
};

use crate::{
    module::{
        Module,
        ProcessArgs,
    },
    node::{
        GetInputMut as _,
        GetOutputMut as _,
    },
    port::{
        Connect as _,
        Disconnect as _,
        InputRef,
        OutputRef,
        PortRef,
    },
};

// =================================================================================================
// Processor
// =================================================================================================

#[derive(Debug)]
pub struct Processor<const C: usize, M>
where
    M: Module,
{
    args: ProcessArgs,
    instances: IndexMap<C, SyncUnsafeCell<M>>,
}

impl<const C: usize, M> Processor<C, M>
where
    M: Debug + Module,
{
    #[instrument(level = "debug", skip(self))]
    pub fn add(&mut self, module: M) -> InstanceRef {
        trace!(?module, "adding module");

        let instance = SyncUnsafeCell::new(module);
        let index = self.instances.add(instance);

        InstanceRef(index)
    }

    pub unsafe fn remove(&mut self, instance_ref: &InstanceRef) {
        unsafe {
            // TODO: Disconnection, etc.

            self.instances.remove_unchecked(instance_ref.0);
        }
    }
}

impl<const C: usize, M> Processor<C, M>
where
    M: Module,
{
    pub unsafe fn connect(&mut self, output_ref: &OutputRef, input_ref: &InputRef) {
        unsafe {
            (*self.instances.get_unchecked(output_ref.0.0).get())
                .output_mut(output_ref.0.1)
                .connect(
                    (*self.instances.get_unchecked(input_ref.0.0).get()).input_mut(input_ref.0.1),
                );
        }
    }

    pub unsafe fn disconnect(&mut self, port_ref: impl Into<PortRef>) {
        unsafe {
            match port_ref.into() {
                PortRef::Input(input) => (*self.instances.get_unchecked(input.0.0).get())
                    .input_mut(input.0.1)
                    .disconnect(),
                PortRef::Output(output) => (*self.instances.get_unchecked(output.0.0).get())
                    .output_mut(output.0.1)
                    .disconnect(),
            }
        }
    }
}

impl<const C: usize, M> Processor<C, M>
where
    M: Module,
{
    pub unsafe fn process(&mut self, iteration: u64) {
        self.args.token.0 = (iteration % 2) as usize;

        self.instances.values().for_each(|module| unsafe {
            (*module.get()).process(&self.args);
        });
    }
}

impl<const C: usize, M> Default for Processor<C, M>
where
    M: Module,
{
    fn default() -> Self {
        Self {
            args: ProcessArgs::default(),
            instances: IndexMap::default(),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Instance Ref

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstanceRef(pub usize);

impl InstanceRef {
    #[must_use]
    pub fn input_ref(&self, input: usize) -> InputRef {
        InputRef((self.0, input))
    }

    #[must_use]
    pub fn output_ref(&self, output: usize) -> OutputRef {
        OutputRef((self.0, output))
    }
}
