use std::fmt::Debug;

use fancy_constructor::new;
use uuid::Uuid;

use crate::{
    module::{
        Module,
        ModuleSource,
    },
    processor::Processor,
};

// =================================================================================================
// Protocol
// =================================================================================================

#[derive(Clone, Debug)]
pub enum Protocol {
    Add(ProtocolAdd),
    Connect(ProtocolConnect),
}

impl Protocol {
    pub fn apply<C, M>(self, context: &C, processor: &mut Processor<M>)
    where
        C: Clone,
        M: Debug + Module + ModuleSource<Context = C>,
    {
        match self {
            Self::Add(add) => add.apply(context.clone(), processor),
            Self::Connect(connect) => connect.apply(processor),
        }
    }
}

impl From<ProtocolAdd> for Protocol {
    fn from(add: ProtocolAdd) -> Self {
        Self::Add(add)
    }
}

impl From<ProtocolConnect> for Protocol {
    fn from(connect: ProtocolConnect) -> Self {
        Self::Connect(connect)
    }
}

// Add

#[derive(new, Clone, Debug)]
pub struct ProtocolAdd {
    instance: Uuid,
    module: Uuid,
}

impl ProtocolAdd {
    pub fn apply<C, M>(self, context: C, processor: &mut Processor<M>)
    where
        M: Debug + Module + ModuleSource<Context = C>,
    {
        let module = M::get(&self.module, context);

        processor.add(self.instance, module);
    }
}

// Connect

#[derive(new, Clone, Debug)]
pub struct ProtocolConnect {
    input_instance: Uuid,
    input_port: usize,
    output_instance: Uuid,
    output_port: usize,
}

impl ProtocolConnect {
    pub fn apply<C, M>(self, processor: &mut Processor<M>)
    where
        C: Clone,
        M: Debug + Module + ModuleSource<Context = C>,
    {
        unsafe {
            processor.connect(
                self.input_instance,
                self.input_port,
                self.output_instance,
                self.output_port,
            );
        }
    }
}
