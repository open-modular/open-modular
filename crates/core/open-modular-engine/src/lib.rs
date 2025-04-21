#![feature(sync_unsafe_cell)]

pub mod bus;
pub mod context;
pub mod module;
pub mod port;
pub mod processor;
pub mod protocol;

// =================================================================================================
// Compute
// =================================================================================================

// Re-Export

#[doc(hidden)]
pub mod _dependencies {
    pub use uuid;
}
