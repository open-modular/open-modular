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
    BUFFER_FRAMES_F64,
    BUFFER_FRAMES_U32,
    SAMPLE_RATE_F64,
    Vector,
};
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
        GetPortOutputVector as _,
        GetPortOutputs as _,
        Port,
        PortOutputs,
        Ports,
    },
};
use tracing::instrument;

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

    outputs: PortOutputs,
    ports: Ports,
    #[debug(skip)]
    #[new(default)]
    _r: PhantomData<R>,
}

impl<R> Define for Sine<R>
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

impl<R> Instantiate for Sine<R>
where
    R: Debug,
{
    type Context = R;

    #[instrument(level = "debug", skip(ports, _context))]
    fn instantiate(ports: Ports, _context: Self::Context) -> Self {
        let factor = 440. * TAU;
        let increment = 1. / SAMPLE_RATE_F64;

        let time = Vector::from_slice(
            &(0..BUFFER_FRAMES_U32)
                .map(|i| f64::from(i) * increment)
                .collect::<Vec<_>>()[..],
        );

        let factor = Vector::splat(factor);
        let increment = Vector::splat(increment * BUFFER_FRAMES_F64);
        let scale = Vector::splat(0.15);
        let output = Vector::default();

        let outputs = ports.outputs();

        Self::new(factor, increment, output, scale, time, outputs, ports)
    }
}

impl<R> Process for Sine<R>
where
    R: Debug,
{
    fn process(&mut self, args: &ProcessArgs) {
        if let Some(Port::Connected(output)) = self.outputs.vector(0, &args.token) {
            self.time += self.increment;

            self.output = self.time;
            self.output *= self.factor;
            self.output = unsafe { simd::simd_fsin(self.output) };
            self.output *= self.scale;

            *output = self.output;
        }
    }
}
