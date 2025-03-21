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
        ModuleInstanceReference,
    },
    port::{
        PortConnect as _,
        PortDisconnect as _,
        PortInputGet as _,
        PortInputReference,
        PortOutputGet as _,
        PortOutputReference,
        PortReference,
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
    instances: IndexMap<Uuid, SyncUnsafeCell<M>>,
}

impl<M> Processor<M>
where
    M: Debug + Module,
{
    #[instrument(level = "debug", skip(self))]
    pub fn add(&mut self, key: Uuid, instance: M) -> ModuleInstanceReference {
        trace!(?instance, "adding module");

        let instance = SyncUnsafeCell::new(instance);

        self.instances.insert(key, instance);

        ModuleInstanceReference::new(key)
    }

    pub fn remove(&mut self, instance_ref: &ModuleInstanceReference) {
        self.instances.swap_remove(&instance_ref.instance);
    }
}

impl<M> Processor<M>
where
    M: Module,
{
    pub fn connect(&mut self, output_ref: &PortOutputReference, input_ref: &PortInputReference) {
        let outputs = self
            .instances
            .get(&output_ref.instance)
            .map(|instance| unsafe { (*instance.get()).as_mut() })
            .expect("output instance to exist");

        let inputs = self
            .instances
            .get(&input_ref.instance)
            .map(|instance| unsafe { (*instance.get()).as_ref() })
            .expect("input instance to exist");

        let output = outputs.port(output_ref.port).expect("output port to exist");
        let input = inputs.port(input_ref.port).expect("input port to exist");

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

impl<M> Processor<M>
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

impl<M> Default for Processor<M>
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
