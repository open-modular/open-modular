// Context

use std::{
    cell::SyncUnsafeCell,
    collections::HashMap,
    simd::num::SimdFloat,
    sync::{
        Arc,
        Weak,
    },
    time::Instant,
};

use crossbeam::channel;
use derive_more::Debug;
use fancy_constructor::new;
use open_modular_core::Vector;
use open_modular_io_audio::{
    device::DeviceOutputFilter as _,
    host::Host,
    stream::{
        Stream,
        StreamActive,
        StreamOutput,
        StreamParameters,
        StreamStatus,
    },
    system::SystemGet as _,
};
// #[cfg(feature = "perf")]
// use open_modular_performance::timing::TimingCollector;
use open_modular_runtime::io::audio::{
    Audio,
    AudioOutput,
    AudioOutputBuffer,
    GetAudioOutputBuffer,
    GetAudioOutputs,
};
use open_modular_synchronization::barrier::Barriers;
use open_modular_utilities::sync::{
    Pending,
    Value,
};

// =================================================================================================
// Audio
// =================================================================================================

// Buffer

#[derive(new, Debug)]
struct AudioBufferWeak(pub Weak<SyncUnsafeCell<Vec<Vector>>>);

impl AudioBufferWeak {
    fn from_buffer(buffer: &AudioOutputBuffer) -> Self {
        Self(Arc::downgrade(&buffer.0))
    }
}

#[derive(Clone, Debug, Default)]
struct AudioBufferWeakMap(pub Arc<SyncUnsafeCell<Vec<AudioBufferWeak>>>);

// -------------------------------------------------------------------------------------------------

// Context

#[derive(new, Clone, Debug)]
pub struct AudioContext {
    sender: channel::Sender<AudioProtocol>,
}

impl Audio for AudioContext {}

impl GetAudioOutputs for AudioContext {
    fn outputs(&self) -> Pending<Vec<AudioOutput>> {
        let (value, pending) = Pending::create();

        self.sender
            .try_send(AudioProtocol::GetOutputs(value))
            .expect("get outputs message to be sent successfully");

        pending
    }
}

impl GetAudioOutputBuffer for AudioContext {
    fn output_buffer(&self, id: u32) -> Pending<AudioOutputBuffer> {
        let (value, pending) = Pending::create();

        self.sender
            .try_send(AudioProtocol::GetOutputBuffer(id, value))
            .expect("get output message to be sent successfully");

        pending
    }
}

// -------------------------------------------------------------------------------------------------

// Controller

#[derive(new, Debug, Default)]
pub struct AudioController {
    #[new(default)]
    host: Host,
    #[new(default)]
    outputs: HashMap<u32, AudioOutputStream>,
}

impl AudioController {
    pub fn is_active(&self) -> bool {
        !self.outputs.is_empty()
    }
}

impl AudioController {
    pub fn collect(&mut self) {
        self.outputs.retain(|_, output| !output.is_empty());
    }
}

impl AudioController {
    #[allow(clippy::manual_let_else, clippy::single_match_else, unused_mut)]
    pub fn output_buffer(
        &mut self,
        mut barriers: Barriers,
        id: u32,
        // mut timing_collector: Option<TimingCollector>,
    ) -> AudioOutputBuffer {
        let device = self.host.device(id);
        let channels = device.channels.output;

        let output = match self.outputs.get_mut(&id) {
            Some(output) => output,
            None => {
                let api = self.host.system();
                let buffers = AudioBufferWeakMap::default();
                let parameters = StreamParameters::for_device(id).channels(channels);

                // Create and activate a new Stream, with a callback function which will utilize
                // the shared buffers that will be handed out to consumers of the audio
                // capability.

                let stream = Stream::output(api, parameters).activate({
                    let buffers = buffers.clone();

                    // Create a default zero-filled vector to use as a multiplier to zero out
                    // re-used vectors when required.

                    let zero = Vector::default();

                    // Create a set of vectors to use as an output mix target for provided buffers.
                    // Currently the output stream is always opened using the maximum number of
                    // channels supported, so this is also the channel capability size.

                    let mut output = (0..channels).map(|_| Vector::default()).collect::<Vec<_>>();

                    move |data, info| {
                        match info.status {
                            StreamStatus::Overflow => {
                                println!("output stream overflow: {:?}", Instant::now());
                            }
                            StreamStatus::Underflow => {
                                println!("output stream underflow: {:?}", Instant::now());
                            }
                            _ => {}
                        }

                        barriers.phase_0.wait();
                        barriers.phase_1.wait();

                        // #[cfg(feature = "perf")]
                        // unsafe {
                        //     timing_collector.as_mut().unwrap_unchecked().enter();
                        // }

                        // Use the shared store of weak buffers which is also stored in the audio
                        // output stream and which is added to when new output buffers are
                        // requested.

                        let buffers = unsafe { &mut (*buffers.0.get()) };

                        // Run retain to clear any buffers where the actual provided buffer is no
                        // longer in use (i.e. has been dropped). Only weak references are held
                        // here, so if the strong reference count is zero, there should be no
                        // external usage.

                        buffers.retain(|buffer| buffer.0.strong_count() > 0);

                        // Only mix buffers and write to the output data if the set of buffers is
                        // not empty. An empty set of buffers indicates that this stream is no
                        // longer in use, and it is expected to be logically collected as part of
                        // the wider process.

                        if !buffers.is_empty() {
                            // Multiply each of the output mix vectors by zero to ensure a clean
                            // starting mix.

                            output.iter_mut().for_each(|output| *output *= zero);

                            // For each provided buffer, zip the output mix with the set of buffer
                            // vectors and add the buffer vector to the output mix (just a basic
                            // summing mix for now, this might be improved upon).

                            for buffer in buffers.iter() {
                                output
                                    .iter_mut()
                                    .zip(unsafe { &*buffer.0.upgrade().unwrap_unchecked().get() })
                                    .for_each(|(output, buffer)| *output += buffer);
                            }

                            // For each of the vectors in the output mix, cast it to the target
                            // output format (f32) and copy it to the appropriate place in the
                            // output data slice.

                            // NOTE: This only works because the output data slice (and the
                            // underlying stream) is configured to use non-interleaved channels, so
                            // each channel can be written sequentially in this way.

                            output.iter().enumerate().for_each(|(i, output)| {
                                output
                                    .cast::<f32>()
                                    .copy_to_slice(&mut data[i * 64..(i + 1) * 64]);
                            });

                            // #[cfg(feature = "perf")]
                            // unsafe {
                            //     timing_collector.as_mut().unwrap_unchecked().
                            // exit(); }
                        }

                        barriers.phase_2.wait();
                    }
                });

                // Create a new audio output stream and add it to the map of active streams.
                // Return a mutable reference to the stream that was added.

                let output = AudioOutputStream::new(buffers, stream);

                self.outputs.insert(id, output);

                unsafe { self.outputs.get_mut(&id).unwrap_unchecked() }
            }
        };

        // Create a new external buffer, and add a aeak reference to it to the active
        // stream.

        let buffer = AudioOutputBuffer::new(channels);
        let buffer_weak = AudioBufferWeak::from_buffer(&buffer);

        unsafe {
            (*output.buffers.0.get()).push(buffer_weak);
        }

        // Return the original buffer. When this buffer is dropped, the local weak
        // reference will be collected as part of the general processing pattern.

        buffer
    }

    pub fn outputs(&self) -> Vec<AudioOutput> {
        self.host
            .devices()
            .output()
            .map(|device| AudioOutput::new(device.channels.output, device.id, device.name.name))
            .collect()
    }
}

// -------------------------------------------------------------------------------------------------

// Protocol

#[derive(Debug)]
pub enum AudioProtocol {
    GetOutputBuffer(u32, Value<AudioOutputBuffer>),
    GetOutputs(#[debug(skip)] Value<Vec<AudioOutput>>),
}

// -------------------------------------------------------------------------------------------------

// Stream

#[derive(new, Debug)]
struct AudioOutputStream {
    buffers: AudioBufferWeakMap,
    _stream: Stream<StreamOutput, StreamActive>,
}

impl AudioOutputStream {
    fn is_empty(&self) -> bool {
        unsafe { (*self.buffers.0.get()).is_empty() }
    }
}
