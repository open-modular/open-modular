use std::fmt::Debug;

use open_modular_engine::module::{
    Module,
    ModuleSource,
};

// =================================================================================================
// Runtime
// =================================================================================================

pub trait Runtime {
    type Context;

    fn run<M>(&self)
    where
        M: Debug + Module + ModuleSource<Context = Self::Context>;
}
