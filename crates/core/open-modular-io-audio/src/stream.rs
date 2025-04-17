use std::{
    ffi::{
        c_char,
        c_int,
        c_uint,
        c_void,
    },
    pin::Pin,
    ptr,
    slice,
};

use bon::Builder;
use derive_more::Debug;
use fancy_constructor::new;
use open_modular_core::{
    BUFFER_FRAMES,
    SAMPLE_RATE,
};
use rtaudio_sys::{
    MAX_NAME_LENGTH,
    RTAUDIO_FLAGS_ALSA_USE_DEFAULT,
    RTAUDIO_FLAGS_HOG_DEVICE,
    RTAUDIO_FLAGS_JACK_DONT_CONNECT,
    RTAUDIO_FLAGS_MINIMIZE_LATENCY,
    RTAUDIO_FLAGS_NONINTERLEAVED,
    RTAUDIO_FLAGS_SCHEDULE_REALTIME,
    RTAUDIO_FORMAT_FLOAT32,
    RTAUDIO_STATUS_INPUT_OVERFLOW,
    RTAUDIO_STATUS_OUTPUT_UNDERFLOW,
    rtaudio_error_t,
    rtaudio_stream_flags_t,
    rtaudio_stream_options,
    rtaudio_stream_parameters,
    rtaudio_stream_status_t,
    rtaudio_t,
};

use crate::system::{
    self,
    Api,
};

// =================================================================================================
// Stream
// =================================================================================================

// Direction

pub trait StreamDirection {
    type Context;
}

#[derive(Debug)]
pub struct StreamOutput;

impl StreamDirection for StreamOutput {
    type Context = StreamOutputContext;
}

#[derive(Debug)]
pub struct StreamUnspecified;

impl StreamDirection for StreamUnspecified {
    type Context = ();
}

// -------------------------------------------------------------------------------------------------

// Stream

#[derive(new, Debug)]
#[new(vis(pub(crate)))]
pub struct Stream<D, S>
where
    D: StreamDirection,
    S: StreamState,
{
    state: Option<(rtaudio_t, Pin<Box<D::Context>>)>,
    _specialisation: (D, S),
}

impl Stream<StreamUnspecified, StreamAny> {
    pub fn output(
        api: impl Into<Api>,
        parameters: impl Into<StreamParameters>,
    ) -> Stream<StreamOutput, StreamInactive> {
        let audio = system::create_audio(api.into());
        let parameters = parameters.into();

        let info = StreamInfo::new(
            StreamBuffersInfo::new(),
            StreamChannelsInfo::new(0, parameters.channels),
            StreamSamplesInfo::new(),
        );

        let output = &mut parameters.into();

        let mut context = Box::pin(StreamOutputContext::new(info));
        let context_ptr: *mut StreamOutputContext = &mut *context;

        let mut buffer_frames = u32::try_from(BUFFER_FRAMES).expect("invalid buffer size");

        let userdata = context_ptr.cast::<c_void>();

        let mut options = StreamOptions {
            flags: StreamFlags::NONINTERLEAVED | StreamFlags::SCHEDULE_REALTIME,
            ..StreamOptions::default()
        }
        .into();

        unsafe {
            rtaudio_sys::rtaudio_open_stream(
                audio,
                output,
                ptr::null_mut(),
                RTAUDIO_FORMAT_FLOAT32,
                u32::try_from(SAMPLE_RATE).expect("invalid sample rate"),
                &mut buffer_frames,
                Some(stream_output_callback),
                userdata,
                &mut options,
                Some(stream_error_callback),
            );
        }

        system::validate_audio(audio);

        context.info.buffers.frames = buffer_frames;

        let latency = unsafe { rtaudio_sys::rtaudio_get_stream_latency(audio) };

        if latency > 0 {
            context.info.latency = Some(usize::try_from(latency).expect("valid latency value"));
        }

        system::validate_audio(audio);

        let sample_rate = unsafe { rtaudio_sys::rtaudio_get_stream_sample_rate(audio) };

        if sample_rate > 0 {
            context.info.samples.rate = sample_rate;
        }

        system::validate_audio(audio);

        Stream::new(Some((audio, context)), (StreamOutput, StreamInactive))
    }
}

impl Stream<StreamOutput, StreamInactive> {
    pub fn activate<F>(mut self, callback: F) -> Stream<StreamOutput, StreamActive>
    where
        F: FnMut(&mut [f32], &StreamInfo) + Send + 'static,
    {
        match self.state.take() {
            Some((audio, mut context)) => {
                context.callback = Box::new(callback);

                unsafe {
                    rtaudio_sys::rtaudio_start_stream(audio);
                }

                system::validate_audio(audio);

                Stream::new(Some((audio, context)), (StreamOutput, StreamActive))
            }
            _ => panic!("state not found"),
        }
    }
}

impl<D, S> AsRef<rtaudio_t> for Stream<D, S>
where
    D: StreamDirection,
    S: StreamState,
{
    fn as_ref(&self) -> &rtaudio_t {
        &self.state.as_ref().expect("data to be present").0
    }
}

impl<D, S> Drop for Stream<D, S>
where
    D: StreamDirection,
    S: StreamState,
{
    fn drop(&mut self) {
        match self.state.take() {
            Some((audio, _)) if !audio.is_null() => unsafe {
                rtaudio_sys::rtaudio_close_stream(audio);
                rtaudio_sys::rtaudio_destroy(audio);
            },
            _ => {}
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Status

pub trait StreamState {}

#[derive(Debug)]
pub struct StreamAny;

impl StreamState for StreamAny {}

#[derive(Debug)]
pub struct StreamInactive;

impl StreamState for StreamInactive {}

#[derive(Debug)]
pub struct StreamActive;

impl StreamState for StreamActive {}

// -------------------------------------------------------------------------------------------------

// Context

#[allow(clippy::type_complexity)]
#[derive(new, Debug)]
pub struct StreamOutputContext {
    info: StreamInfo,
    #[debug(skip)]
    #[new(val(Box::new(|_, _| {})))]
    callback: Box<dyn FnMut(&mut [f32], &StreamInfo) + Send + 'static>,
}

// -------------------------------------------------------------------------------------------------

// Flags

bitflags::bitflags! {
    /// Stream option flags.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct StreamFlags: rtaudio_stream_flags_t {
        const NONINTERLEAVED = RTAUDIO_FLAGS_NONINTERLEAVED;
        const MINIMIZE_LATENCY = RTAUDIO_FLAGS_MINIMIZE_LATENCY;
        const HOG_DEVICE = RTAUDIO_FLAGS_HOG_DEVICE;
        const SCHEDULE_REALTIME = RTAUDIO_FLAGS_SCHEDULE_REALTIME;
        const ALSA_USE_DEFAULT = RTAUDIO_FLAGS_ALSA_USE_DEFAULT;
        const JACK_DONT_CONNECT = RTAUDIO_FLAGS_JACK_DONT_CONNECT;
    }
}

// -------------------------------------------------------------------------------------------------

// Info

#[derive(new, Clone, Debug)]
pub struct StreamInfo {
    pub buffers: StreamBuffersInfo,
    pub channels: StreamChannelsInfo,
    pub samples: StreamSamplesInfo,
    #[new(default)]
    pub latency: Option<usize>,
    #[new(val = StreamStatus::empty())]
    pub status: StreamStatus,
    #[new(val(0.0))]
    pub time: f64,
}

#[derive(new, Clone, Debug)]
pub struct StreamBuffersInfo {
    #[new(val = u32::try_from(BUFFER_FRAMES).expect("invalid buffer size"))]
    pub frames: u32,
    #[new(default)]
    pub interleaved: bool,
}

#[derive(new, Clone, Debug)]
pub struct StreamChannelsInfo {
    pub input: u32,
    pub output: u32,
}

#[derive(new, Clone, Debug)]
pub struct StreamSamplesInfo {
    #[new(val = RTAUDIO_FORMAT_FLOAT32)]
    pub format: u64,
    #[allow(clippy::cast_possible_truncation)]
    #[new(val = u32::try_from(SAMPLE_RATE).expect("invalid sample rate"))]
    pub rate: u32,
}

// -------------------------------------------------------------------------------------------------

// Options

#[derive(Debug)]
pub struct StreamOptions {
    pub buffers: u32,
    pub flags: StreamFlags,
    pub name: String,
    pub priority: i32,
}

impl Default for StreamOptions {
    fn default() -> Self {
        Self {
            buffers: 4,
            flags: StreamFlags::empty(),
            name: String::default(),
            priority: -1,
        }
    }
}

impl From<StreamOptions> for rtaudio_stream_options {
    fn from(value: StreamOptions) -> Self {
        rtaudio_stream_options {
            flags: value.flags.bits(),
            name: [0i8; MAX_NAME_LENGTH],
            num_buffers: value.buffers,
            priority: value.priority,
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Parameters

#[derive(new, Builder, Debug)]
#[builder(derive(Debug), start_fn = for_device)]
pub struct StreamParameters {
    #[builder(start_fn)]
    pub device: u32,
    #[builder(default = 2)]
    pub channels: u32,
    #[builder(default = 0)]
    pub channels_offset: u32,
}

impl From<StreamParameters> for rtaudio_stream_parameters {
    fn from(stream_parameters: StreamParameters) -> Self {
        rtaudio_stream_parameters {
            device_id: stream_parameters.device,
            num_channels: stream_parameters.channels,
            first_channel: stream_parameters.channels_offset,
        }
    }
}

impl<S> From<StreamParametersBuilder<S>> for StreamParameters
where
    S: stream_parameters_builder::IsComplete,
{
    fn from(builder: StreamParametersBuilder<S>) -> Self {
        builder.build()
    }
}

// -------------------------------------------------------------------------------------------------

// Status

bitflags::bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
    pub struct StreamStatus: rtaudio_stream_status_t {
        const Ok = 0;
        const Overflow = RTAUDIO_STATUS_INPUT_OVERFLOW;
        const Underflow = RTAUDIO_STATUS_OUTPUT_UNDERFLOW;
    }
}

// -------------------------------------------------------------------------------------------------

// Callbacks

#[unsafe(no_mangle)]
pub(crate) unsafe extern "C" fn stream_output_callback(
    output: *mut c_void,
    _input: *mut c_void,
    frames: c_uint,
    stream_time: f64,
    status: rtaudio_stream_status_t,
    userdata: *mut c_void,
) -> c_int {
    let context_ptr = userdata.cast::<StreamOutputContext>();
    let context = unsafe { &mut *context_ptr };

    context.info.status = StreamStatus::from_bits_truncate(status);
    context.info.time = stream_time;

    let output_channels = context.info.channels.output;
    let output_ptr = output.cast::<f32>();

    let output = if output_ptr.is_null() || output_channels == 0 {
        &mut [] as &mut [f32]
    } else {
        unsafe { slice::from_raw_parts_mut(output_ptr, (output_channels * frames) as usize) }
    };

    (context.callback)(output, &context.info);

    0
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "C" fn stream_error_callback(
    raw_err: rtaudio_error_t,
    _raw_msg: *const c_char,
) {
    println!("error callback invoked: {raw_err}");
}
