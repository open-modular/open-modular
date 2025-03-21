use std::marker::PhantomData;

use fancy_constructor::new;
use open_modular_engine::{
    module::{
        ModuleDefine,
        ModuleDefinition,
        ModuleDefinitionBuilder,
        ModuleInstantiate,
        module,
    },
    port::{
        Port,
        PortInputVectorGet as _,
        PortInputs,
        PortOutputVectorGet as _,
        PortOutputs,
    },
    processor::{
        Process,
        ProcessArgs,
    },
};

// =================================================================================================
// Utility
// =================================================================================================

#[module(id = "54d93000-7dd2-45ce-a3f1-ad53b0a04fac")]
#[derive(new, Debug)]
#[new(vis())]
pub struct Multiple<R> {
    port_inputs: PortInputs,
    port_outputs: PortOutputs,
    #[new(default)]
    _r: PhantomData<R>,
}

impl<R> ModuleDefine for Multiple<R> {
    fn define(module: ModuleDefinitionBuilder) -> impl Into<ModuleDefinition> {
        module
            .name("util/mult")
            .description("Multiple (4-Way)")
            .with_input(|input| input.name("Input"))
            .with_output(|output| output.name("Output 0"))
            .with_output(|output| output.name("Output 1"))
            .with_output(|output| output.name("Output 2"))
            .with_output(|output| output.name("Output 3"))
    }
}

impl<R> ModuleInstantiate for Multiple<R> {
    type Context = R;

    fn instantiate(
        _context: Self::Context,
        port_inputs: PortInputs,
        port_outputs: PortOutputs,
    ) -> Self {
        Self::new(port_inputs, port_outputs)
    }
}

impl<R> Process for Multiple<R> {
    fn process(&mut self, args: &ProcessArgs) {
        if let Some(Port::Connected(input)) = self.port_inputs.vector(0, &args.token) {
            for i in 0..4 {
                if let Some(Port::Connected(output)) = self.port_outputs.vector(i, &args.token) {
                    *output = *input;
                }
            }
        }
    }
}
