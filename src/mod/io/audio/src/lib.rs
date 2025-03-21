#![feature(sync_unsafe_cell)]

use derive_more::with_trait::Debug;
use fancy_constructor::new;
use open_modular_core::MAX_CHANNELS_U32;
use open_modular_engine::{
    self,
    module::{
        ModuleDefine,
        ModuleDefinition,
        ModuleDefinitionBuilder,
        ModuleInstantiate,
        module,
    },
    port::{
        Port,
        PortInputVectorGet,
        PortInputs,
        PortOutputs,
    },
    processor::{
        Process,
        ProcessArgs,
    },
};
use open_modular_runtime::io::audio::{
    AudioOutput,
    AudioOutputBuffer,
    GetAudio,
    GetAudioOutputBuffer as _,
    GetAudioOutputs as _,
};
use open_modular_utils::sync::Pending;
use tracing::{
    debug,
    info,
    instrument,
};

// =================================================================================================
// Audio
// =================================================================================================

#[module(id = "47d0fca2-cb58-4011-8a55-31ecd4b184c1")]
#[derive(new, Debug)]
#[new(vis())]
pub struct Output<R>
where
    R: Debug + GetAudio,
{
    port_inputs: PortInputs,
    port_outputs: PortOutputs,
    runtime: R,
    state: OutputState,
}

impl<R> ModuleDefine for Output<R>
where
    R: Debug + GetAudio,
{
    fn define(module: ModuleDefinitionBuilder) -> impl Into<ModuleDefinition> {
        (0..MAX_CHANNELS_U32).fold(
            module
                .name("audio/out")
                .description("Multi-Channel Audio Output"),
            |module, i| module.with_input(|input| input.name(format!("Channel {i}"))),
        )
    }
}

impl<R> ModuleInstantiate for Output<R>
where
    R: Debug + GetAudio,
{
    type Context = R;

    #[instrument(level = "debug", skip(context, port_inputs, port_outputs))]
    fn instantiate(
        context: Self::Context,
        port_inputs: PortInputs,
        port_outputs: PortOutputs,
    ) -> Self {
        info!("instantiating audio output module, and setting state to await pending outputs");

        let state = OutputState::AwaitingOutputs(context.audio().outputs());

        Self::new(port_inputs, port_outputs, context, state)
    }
}

impl<R> Process for Output<R>
where
    R: Debug + GetAudio,
{
    #[instrument(level = "debug", skip(self))]
    fn process(&mut self, args: &ProcessArgs) {
        match &mut self.state {
            OutputState::Active(output_buffer) => {
                let output_buffer = unsafe { &mut *output_buffer.0.get() };

                output_buffer
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, output_vector)| {
                        if let Some(Port::Connected(input)) =
                            self.port_inputs.vector(i, &args.token)
                        {
                            output_vector.clone_from(input);
                        }
                    });
            }
            OutputState::AwaitingOutputBuffer(output_buffer_pending) => {
                if let Some(buffer) = output_buffer_pending.value() {
                    debug!(
                        action = "receive",
                        correlation = output_buffer_pending.correlation,
                        protocol = "audio",
                        variant = "get_output_buffer",
                        ?buffer
                    );

                    self.state = OutputState::Active(buffer);
                }
            }
            OutputState::AwaitingOutputs(outputs_pending) => {
                if let Some(outputs) = outputs_pending.value() {
                    debug!(
                        action = "receive",
                        correlation = outputs_pending.correlation,
                        protocol = "audio",
                        variant = "get_outputs",
                        ?outputs,
                    );

                    let pending_buffer = self.runtime.audio().output_buffer(131);

                    self.state = OutputState::AwaitingOutputBuffer(pending_buffer);
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

// State

#[derive(Debug)]
enum OutputState {
    Active(AudioOutputBuffer),
    AwaitingOutputBuffer(Pending<AudioOutputBuffer>),
    AwaitingOutputs(Pending<Vec<AudioOutput>>),
}
