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
    node::{
        GetInput,
        GetOutput as _,
        GetOutputMut as _,
        Node,
    },
    port::{
        GetConnected,
        GetInputVector,
        GetOutputVectorMut as _,
    },
};

// =================================================================================================
// Utility
// =================================================================================================

#[module(id = "54d93000-7dd2-45ce-a3f1-ad53b0a04fac")]
#[derive(new, Debug)]
#[new(vis())]
pub struct Multiple<R> {
    node: Node,
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

    fn instantiate(node: Node, _context: Self::Context) -> Self {
        Self::new(node)
    }
}

impl<R> Process for Multiple<R> {
    fn process(&mut self, args: &ProcessArgs) {
        if self.input(0).connected() {
            (0..4).for_each(|i| {
                if self.output(i).connected() {
                    *self.output_mut(i).output_vector_mut(&args.token) =
                        unsafe { *self.input(0).input_vector_unchecked(&args.token) };
                }
            });
        }
    }
}
