#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![feature(portable_simd)]

use std::{
    simd::Simd,
    time::Duration,
};

// =================================================================================================
// Core
// =================================================================================================

// Internal Constants

static BUFFER_FRAMES: usize = 64;
static SAMPLE_RATE: usize = 48_000;

static NANOSECONDS_PER_SECOND: usize = 1_000_000_000;
static NANOSECONDS_PER_FRAME: usize = NANOSECONDS_PER_SECOND / SAMPLE_RATE;
static NANOSECONDS_PER_BUFFER: usize = NANOSECONDS_PER_FRAME * BUFFER_FRAMES;

// Public Constants

pub static BUFFER_DURATION: Duration = Duration::from_nanos(NANOSECONDS_PER_BUFFER as u64);
pub static BUFFER_FRAMES_U32: u32 = BUFFER_FRAMES as u32;
pub static BUFFER_FRAMES_F64: f64 = BUFFER_FRAMES as f64;
pub static FRAME_DURATION: Duration = Duration::from_nanos(NANOSECONDS_PER_FRAME as u64);
pub static MIN_CHANNELS_U32: u32 = 2;
pub static MAX_CHANNELS_U32: u32 = 16;
pub static SAMPLE_RATE_U32: u32 = SAMPLE_RATE as u32;
pub static SAMPLE_RATE_F64: f64 = SAMPLE_RATE as f64;

// Public Types

pub type Sample = f64;
pub type Vector = Simd<Sample, BUFFER_FRAMES>;
