use std::fmt::Debug;

use open_modular_engine::module::module_enum;
use open_modular_module_gen::Sine as OscillatorSine;
use open_modular_module_io::audio::Output as AudioOutput;
use open_modular_module_util::Multiple as UtilityMultiple;
use open_modular_runtime::io::audio::GetAudio;

// =================================================================================================
// Module
// =================================================================================================

#[rustfmt::skip]
#[module_enum(id = "2d845926-8ef1-43ec-9be2-8471cb55a2e4")]
#[derive(Debug)]
pub enum Module<R>
where
    R: Debug + GetAudio,
{
    // IO

    AudioOutput,

    // Oscillator
    
    OscillatorSine,

    // Utility
    
    UtilityMultiple,
}

#[rustfmt::skip]
#[module_enum(id = "68f9841f-983d-4eb0-a99d-444a615436d6")]
#[derive(Debug)]
pub enum ModulePerf<R>
where
    R: Debug,
{

    // Oscillator

    OscillatorSine,

    // Utility

    UtilityMultiple,
}
