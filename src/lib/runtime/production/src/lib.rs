#![feature(portable_simd)]
#![feature(sync_unsafe_cell)]
#![feature(trait_alias)]

mod context;
mod process;
mod runtime;

// =================================================================================================
// Production
// =================================================================================================

// Configuration

static COMPUTE_CAPACITY: usize = 512;

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::runtime::Runtime;
