#![feature(portable_simd)]

use std::{
    simd::Simd,
    time::Duration,
};

// =================================================================================================
// Core
// =================================================================================================

// Constants

pub static BUFFER_FRAMES: usize = 64;
pub static MAX_CHANNELS: usize = 16;
pub static MIN_CHANNELS: usize = 2;
pub static NANOSECONDS_PER_SECOND: usize = 1_000_000_000;
pub static NANOSECONDS_PER_FRAME: usize = NANOSECONDS_PER_SECOND / SAMPLE_RATE;
pub static NANOSECONDS_PER_BUFFER: usize = NANOSECONDS_PER_FRAME * BUFFER_FRAMES;
pub static SAMPLE_RATE: usize = 48_000;

// Durations

pub static BUFFER_DURATION: Duration = Duration::from_nanos(NANOSECONDS_PER_BUFFER as u64);
pub static FRAME_DURATION: Duration = Duration::from_nanos(NANOSECONDS_PER_FRAME as u64);

// Types

pub type Sample = f64;
pub type Vector = Simd<Sample, BUFFER_FRAMES>;
