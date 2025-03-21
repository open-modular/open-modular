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
        ProcessArgs,
    },
    port::{
        // Connect as _,
        Connect as _,
        Disconnect as _,
        PortInputReference,
        PortOutputReference,
        PortReference,
    },
    port::{
        GetPortInputs,
        // GetInputMut as _,
        // GetOutputMut as _,
        GetPortOutputs,
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
    instances: IndexMap<Uuid, SyncUnsafeCell<M>>,
}

impl<const C: usize, M> Processor<C, M>
where
    M: Debug + Module,
{
    #[instrument(level = "debug", skip(self))]
    pub fn add(&mut self, key: Uuid, instance: M) -> InstanceRef {
        trace!(?instance, "adding module");

        let instance = SyncUnsafeCell::new(instance);

        self.instances.insert(key, instance);

        InstanceRef(key)
    }

    pub fn remove(&mut self, instance_ref: &InstanceRef) {
        self.instances.swap_remove(&instance_ref.0);
    }
}

impl<const C: usize, M> Processor<C, M>
where
    M: Module,
{
    pub fn connect(&mut self, output_ref: &PortOutputReference, input_ref: &PortInputReference) {
        let output_instance = self
            .instances
            .get(&output_ref.0.0)
            .expect("instance to exist");

        let outputs = unsafe { (*output_instance.get()).outputs() };

        let output = outputs
            .outputs
            .get(output_ref.0.1)
            .expect("output to exist");

        // let output = unsafe {
        //     (*self
        //         .instances
        //         .get(&output_ref.0.0)
        //         .expect("output instance to exist")
        //         .get())
        //     .output_mut(output_ref.0.1)
        //     .expect("output port to exist")
        // };

        let input_instance = self
            .instances
            .get(&input_ref.0.0)
            .expect("instance to exist");

        let inputs = unsafe { (*input_instance.get()).inputs() };

        let input = inputs
            .inputs
            .get(input_ref.0.1)
            .expect("input port to exist");

        output.connect(input);
    }

    pub fn disconnect(&mut self, port_ref: impl Into<PortReference>) {
        // match port_ref.into() {
        //     PortRef::Input(input_ref) => {
        //         let input = unsafe {
        //             (*self
        //                 .instances
        //                 .get(&input_ref.0.0)
        //                 .expect("output instance to exist")
        //                 .get())
        //             .input_mut(input_ref.0.1)
        //             .expect("input port to exist")
        //         };

        //         input.disconnect();
        //     }
        //     PortRef::Output(output_ref) => {
        //         let output = unsafe {
        //             (*self
        //                 .instances
        //                 .get(&output_ref.0.0)
        //                 .expect("output instance to exist")
        //                 .get())
        //             .output_mut(output_ref.0.1)
        //             .expect("output port to exist")
        //         };

        //         output.disconnect();
        //     }
        // }
    }
}

impl<const C: usize, M> Processor<C, M>
where
    M: Module,
{
    pub fn process(&mut self, iteration: u64) {
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
pub struct InstanceRef(pub Uuid);

impl InstanceRef {
    #[must_use]
    pub fn input_ref(&self, input: usize) -> PortInputReference {
        PortInputReference((self.0, input))
    }

    #[must_use]
    pub fn output_ref(&self, output: usize) -> PortOutputReference {
        PortOutputReference((self.0, output))
    }
}
