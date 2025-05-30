#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(portable_simd)]

use std::{
    f64::consts::TAU,
    intrinsics::simd,
    marker::PhantomData,
};

use derive_more::with_trait::Debug;
use fancy_constructor::new;
use open_modular_core::{
    BUFFER_FRAMES,
    SAMPLE_RATE,
    Vector,
};
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
// Oscillator
// =================================================================================================

#[module(id = "f75487a4-7847-43f9-ab47-71bd6acfb78d")]
#[derive(new, Debug)]
#[new(vis())]
pub struct Sine<R>
where
    R: Debug,
{
    factor: Vector,
    increment: Vector,
    output: Vector,
    scale: Vector,
    time: Vector,

    port_inputs: PortInputs,
    port_outputs: PortOutputs,
    #[debug(skip)]
    #[new(default)]
    _r: PhantomData<R>,
}

impl<R> ModuleDefine for Sine<R>
where
    R: Debug,
{
    fn define(module: ModuleDefinitionBuilder) -> impl Into<ModuleDefinition> {
        module
            .name("oscillator/sine")
            .description("Sinusoidal Oscillator")
            .with_output(|output| output.name("Output"))
    }
}

impl<R> ModuleInstantiate for Sine<R>
where
    R: Debug,
{
    type Context = R;

    #[allow(clippy::cast_precision_loss)]
    fn instantiate(
        _context: Self::Context,
        port_inputs: PortInputs,
        port_outputs: PortOutputs,
    ) -> Self {
        let factor = 440. * TAU;
        let increment = 1. / SAMPLE_RATE as f64;

        let time = Vector::from_slice(
            &(0..u32::try_from(BUFFER_FRAMES).expect("invalid buffer size"))
                .map(|i| f64::from(i) * increment)
                .collect::<Vec<_>>()[..],
        );

        let factor = Vector::splat(factor);
        let increment = Vector::splat(increment * BUFFER_FRAMES as f64);
        let scale = Vector::splat(0.15);
        let output = Vector::default();

        Self::new(
            factor,
            increment,
            output,
            scale,
            time,
            port_inputs,
            port_outputs,
        )
    }
}

impl<R> Process for Sine<R>
where
    R: Debug,
{
    fn process(&mut self, args: &ProcessArgs) {
        if let Some(Port::Connected(output)) = self.port_outputs.vector(0, &args.token) {
            self.time += self.increment;

            self.output = self.time;
            self.output *= self.factor;
            self.output = unsafe { simd::simd_fsin(self.output) };
            self.output *= self.scale;

            *output = self.output;
        }
    }
}
