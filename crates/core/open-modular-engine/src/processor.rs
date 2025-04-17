use std::{
    cell::SyncUnsafeCell,
    fmt::Debug,
};

use fancy_constructor::new;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::{
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

// Protocol

#[derive(Clone, Debug)]
pub enum ProcessorProtocol {
    Add(ProcessorProtocolAdd),
    Connect(ProcessorProtocolConnect),
}

impl ProcessorProtocol {
    pub fn apply<C, M>(self, context: &C, processor: &mut Processor<M>)
    where
        C: Clone,
        M: Debug + Module + ModuleSource<Context = C>,
    {
        match self {
            Self::Add(add) => {
                let module = M::get(&add.module, context.clone());

                processor.add(add.instance, module);
            }
            Self::Connect(connect) => unsafe {
                processor.connect(
                    connect.input_instance,
                    connect.input_port,
                    connect.output_instance,
                    connect.output_port,
                );
            },
        }
    }
}

impl From<ProcessorProtocolAdd> for ProcessorProtocol {
    fn from(add: ProcessorProtocolAdd) -> Self {
        Self::Add(add)
    }
}

impl From<ProcessorProtocolConnect> for ProcessorProtocol {
    fn from(connect: ProcessorProtocolConnect) -> Self {
        Self::Connect(connect)
    }
}

#[derive(new, Clone, Debug)]
pub struct ProcessorProtocolAdd {
    instance: Uuid,
    module: Uuid,
}

#[derive(new, Clone, Debug)]
pub struct ProcessorProtocolConnect {
    input_instance: Uuid,
    input_port: usize,
    output_instance: Uuid,
    output_port: usize,
}
