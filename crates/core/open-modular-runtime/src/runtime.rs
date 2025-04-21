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
    type Error;

    /// # TODO
    ///
    /// # Errors
    fn run<M>(&self) -> Result<(), Self::Error>
    where
        M: Debug + Module + ModuleSource<Context = Self::Context>;
}
