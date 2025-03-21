use std::marker::PhantomData;

use fancy_constructor::new;
use open_modular_engine::{
    module::{
        Define,
        Instantiate,
        ModuleDefinition,
        ModuleDefinitionBuilder,
        Process,
        ProcessArgs,
        module,
    },
    port::{
        GetPortInput as _,
        GetPortInputs as _,
        GetPortOutput as _,
        GetPortOutputs as _,
        Port,
        Ports,
    },
};

// =================================================================================================
// Utility
// =================================================================================================

#[module(id = "54d93000-7dd2-45ce-a3f1-ad53b0a04fac")]
#[derive(new, Debug)]
#[new(vis())]
pub struct Multiple<R> {
    ports: Ports,
    #[new(default)]
    _r: PhantomData<R>,
}

impl<R> Define for Multiple<R> {
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

impl<R> Instantiate for Multiple<R> {
    type Context = R;

    fn instantiate(ports: Ports, _context: Self::Context) -> Self {
        Self::new(ports)
    }
}

impl<R> Process for Multiple<R> {
    fn process(&mut self, args: &ProcessArgs) {
        let inputs = self.inputs();
        let mut outputs = self.outputs();

        if let Some(Port::Connected(input)) = inputs.port(0, &args.token) {
            for i in 0..4 {
                if let Some(Port::Connected((output, _))) = outputs.port(i, &args.token) {
                    *output = *input;
                }
            }
        }
    }
}
