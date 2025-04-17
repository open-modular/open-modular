use std::fmt::Debug;

use open_modular_engine::module::module_enum;
use open_modular_modules_generators::Sine;
use open_modular_modules_io::audio::Output;
use open_modular_modules_utilities::Multiple;
use open_modular_runtime::{
    io::audio::GetAudio,
    runtime::Runtime as _,
};
use open_modular_runtime_production::Runtime;

// =================================================================================================
// Tone
// =================================================================================================

pub fn main() {
    Runtime::default().run::<Module<_>>();
}

#[module_enum(id = "2d845926-8ef1-43ec-9be2-8471cb55a2e4")]
#[derive(Debug)]
pub enum Module<R>
where
    R: Debug + GetAudio,
{
    Multiple,
    Output,
    Sine,
}
