use std::{
    cell::SyncUnsafeCell,
    fmt::Debug,
};

use fancy_constructor::new;
use indexmap::IndexMap;
use open_modular_core::Vector;
use uuid::Uuid;

use crate::{
    bus::BusReceiver,
    module::{
        Module,
        ModuleSource,
    },
    port::{
        PortConnect as _,
        PortDisconnect as _,
        PortInputGet as _,
        PortOutputGet as _,
    },
};

// =================================================================================================
// Processor
// =================================================================================================

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

// -------------------------------------------------------------------------------------------------

// Processor

#[derive(new, Debug)]
pub struct Processor<M>
where
    M: Module,
{
    #[new(default)]
    args: ProcessArgs,
    #[new(default)]
    modules: IndexMap<Uuid, SyncUnsafeCell<M>>,
    receiver: BusReceiver,
}

impl<M> Processor<M>
where
    M: Debug + Module,
{
    pub fn add(&mut self, instance: Uuid, module: M) {
        let module = SyncUnsafeCell::new(module);

        self.modules.insert(instance, module);
    }

    pub fn remove(&mut self, instance: &Uuid) {
        self.modules.swap_remove(instance);
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
        input_instance: Uuid,
        input_port: usize,
        output_instance: Uuid,
        output_port: usize,
    ) {
        let outputs = self
            .modules
            .get(&output_instance)
            .map(|instance| unsafe { (*instance.get()).as_mut() })
            .expect("output instance to exist");

        let inputs = self
            .modules
            .get(&input_instance)
            .map(|instance| unsafe { (*instance.get()).as_ref() })
            .expect("input instance to exist");

        let output = outputs.port(output_port).expect("output port to exist");
        let input = inputs.port(input_port).expect("input port to exist");

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
    pub unsafe fn disconnect(&mut self, input_instance: Uuid, input_port: usize) {
        let inputs = self
            .modules
            .get(&input_instance)
            .map(|instance| unsafe { (*instance.get()).as_ref() })
            .expect("input instance to exist");

        let input = inputs.port(input_port).expect("input port to exist");

        unsafe {
            input.disconnect();
        }
    }
}

impl<M> Processor<M>
where
    M: Module,
{
    pub fn process<C>(&mut self, context: &C, iteration: u64, _output: &mut [Vector])
    where
        C: Clone,
        M: Debug + ModuleSource<Context = C>,
    {
        self.args.token.0 = (iteration % 2) as usize;

        if let Some(protocol) = self.receiver.receive() {
            protocol.apply(context, self);
        }

        self.modules.values().for_each(|module| unsafe {
            (*module.get()).process(&self.args);
        });
    }
}
