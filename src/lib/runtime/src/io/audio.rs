use std::{
    cell::SyncUnsafeCell,
    fmt::Debug,
    sync::Arc,
};

use fancy_constructor::new;
use open_modular_core::Vector;
use open_modular_utils::sync::Pending;

// =================================================================================================
// Audio
// =================================================================================================

// Access

pub trait GetAudio {
    fn audio(&self) -> &impl Audio;
}

// -------------------------------------------------------------------------------------------------

// Functionality

#[rustfmt::skip]
pub trait Audio:
      Debug 
    + GetAudioOutputs
    + GetAudioOutputBuffer
{
}

pub trait GetAudioOutputs {
    fn outputs(&self) -> Pending<Vec<AudioOutput>>;
}

pub trait GetAudioOutputBuffer {
    fn output_buffer(&self, id: u32) -> Pending<AudioOutputBuffer>;
}

// -------------------------------------------------------------------------------------------------

// Output

#[derive(new, Debug)]
pub struct AudioOutput {
    pub channels: u32,
    pub id: u32,
    #[new(into)]
    pub name: String,
}

#[derive(Debug)]
pub struct AudioOutputBuffer(pub Arc<SyncUnsafeCell<Vec<Vector>>>);

impl AudioOutputBuffer {
    #[must_use]
    pub fn new(channels: u32) -> Self {
        let buffer = Arc::new(SyncUnsafeCell::new(
            (0..channels).map(|_| Vector::default()).collect(),
        ));

        Self(buffer)
    }
}
