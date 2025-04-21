use std::fmt::Debug;

use open_modular_engine::module::{
    Module,
    ModuleSource,
};
use open_modular_io_audio::{
    Api,
    Audio,
    Configuration as AudioConfiguration,
    Device,
    Stream,
    StreamInactive,
    StreamInfo,
    StreamOutput,
    StreamParameters,
};
use open_modular_runtime::runtime;

use crate::{
    configuration::Configuration,
    context::Context,
    error::{
        Error,
        Result,
    },
};

// =================================================================================================
// Runtime
// =================================================================================================

#[derive(Debug)]
pub struct Runtime {
    configuration: Configuration,
}

impl Runtime {
    #[must_use]
    pub fn new(configuration: Configuration) -> Self {
        Self { configuration }
    }
}

impl runtime::Runtime for Runtime {
    type Context = Context;
    type Error = Error;

    fn run<M>(&self) -> Result<()>
    where
        M: Debug + Module + ModuleSource<Context = Self::Context>,
    {
        let audio = RuntimeAudio::new(&self.configuration.audio)?;

        audio.activate(|_data, _info| {})?;

        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------

// Audio

#[derive(Debug)]
struct RuntimeAudio {
    stream: Stream<StreamOutput, StreamInactive>,
}

impl RuntimeAudio {
    fn new(configuration: &AudioConfiguration) -> Result<Self> {
        let api = Api::try_from(configuration.api.as_str())?;
        let audio = Audio::new(api)?;

        let output = StreamParameters::for_device(configuration.output.device)
            .channels(configuration.output.channels)
            .build();

        let stream = Stream::output(audio, output)?;

        Ok(Self { stream })
    }
}

impl RuntimeAudio {
    fn activate<F>(self, callback: F) -> Result<()>
    where
        F: FnMut(&mut [f32], &StreamInfo) + Send + 'static,
    {
        self.stream.activate(callback)?;

        Ok(())
    }
}
