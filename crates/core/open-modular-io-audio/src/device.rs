use std::{
    ffi::CStr,
    slice,
};

use fancy_constructor::new;
use open_modular_core::{
    MIN_CHANNELS,
    SAMPLE_RATE,
};
use rtaudio_sys::{
    rtaudio_device_info,
    rtaudio_format_t,
};

use crate::{
    audio::Audio,
    error::{
        Error,
        GeneralError,
        Result,
    },
};

// =================================================================================================
// Device
// =================================================================================================

// Info

#[derive(new, Debug)]
pub struct Device {
    pub id: u32,
    pub name: DeviceName,
    pub channels: DeviceChannels,
    pub defaults: DeviceDefaults,
    pub formats: DeviceFormats,
    pub rates: DeviceRates,
}

impl Device {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn available(audio: &Audio) -> Result<impl Iterator<Item = Result<Self>>> {
        audio
            .run(|audio| unsafe { rtaudio_sys::rtaudio_device_count(audio.raw()) })
            .map(|count| (0..count).map(|i| Device::from_index(audio, i)))
    }
}

impl Device {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn from_raw(audio: &Audio, id: u32) -> Result<Self> {
        audio
            .run(|audio| unsafe { rtaudio_sys::rtaudio_get_device_info(audio.raw(), id) })
            .and_then(TryInto::try_into)
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn from_index(audio: &Audio, index: i32) -> Result<Self> {
        audio
            .run(|audio| unsafe { rtaudio_sys::rtaudio_get_device_id(audio.raw(), index) })
            .and_then(|id| Self::from_raw(audio, id))
    }
}

impl TryFrom<rtaudio_device_info> for Device {
    type Error = Error;

    #[allow(clippy::map_unwrap_or)]
    fn try_from(device_info: rtaudio_device_info) -> Result<Self> {
        let id = device_info.id;
        let name = device_info.try_into()?;
        let channels = device_info.into();
        let defaults = device_info.into();
        let formats = device_info.into();
        let rates = device_info.into();

        Ok(Self::new(id, name, channels, defaults, formats, rates))
    }
}

// Channels

#[derive(new, Debug)]
pub struct DeviceChannels {
    pub duplex: u32,
    pub input: u32,
    pub output: u32,
}

impl From<rtaudio_device_info> for DeviceChannels {
    fn from(info: rtaudio_device_info) -> Self {
        Self::new(
            info.duplex_channels,
            info.input_channels,
            info.output_channels,
        )
    }
}

// Defaults

#[derive(new, Debug)]
pub struct DeviceDefaults {
    pub input: bool,
    pub output: bool,
}

impl From<rtaudio_device_info> for DeviceDefaults {
    fn from(info: rtaudio_device_info) -> Self {
        Self::new(info.is_default_input != 0, info.is_default_output != 0)
    }
}

// Formats

bitflags::bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct DeviceFormats: rtaudio_format_t {
        const I8 = rtaudio_sys::RTAUDIO_FORMAT_SINT8;
        const I16 = rtaudio_sys::RTAUDIO_FORMAT_SINT16;
        const I24 = rtaudio_sys::RTAUDIO_FORMAT_SINT24;
        const I32 = rtaudio_sys::RTAUDIO_FORMAT_SINT32;
        const F32 = rtaudio_sys::RTAUDIO_FORMAT_FLOAT32;
        const F64 = rtaudio_sys::RTAUDIO_FORMAT_FLOAT64;
    }
}

impl From<rtaudio_device_info> for DeviceFormats {
    fn from(info: rtaudio_device_info) -> Self {
        Self::from_bits_truncate(info.native_formats)
    }
}

// Name

#[derive(new, Debug)]
pub struct DeviceName {
    #[new(into)]
    pub name: String,
}

impl TryFrom<rtaudio_device_info> for DeviceName {
    type Error = Error;

    fn try_from(info: rtaudio_device_info) -> Result<Self> {
        let name_ptr = info.name.as_ptr().cast::<u8>();
        let name_len = info.name.len();
        let name_bytes = unsafe { slice::from_raw_parts(name_ptr, name_len) };
        let name = CStr::from_bytes_until_nul(name_bytes)
            .map(|name| name.to_string_lossy().to_string())
            .map_err(|_| GeneralError::create("invalid device name"))?;

        Ok(Self::new(name))
    }
}

// Rates

#[derive(new, Debug)]
pub struct DeviceRates {
    pub available: Vec<u32>,
    pub preferred: u32,
}

impl From<rtaudio_device_info> for DeviceRates {
    fn from(info: rtaudio_device_info) -> Self {
        Self::new(
            info.sample_rates
                .into_iter()
                .take_while(|rate| *rate != 0)
                .collect(),
            info.preferred_sample_rate,
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Filters

pub trait DeviceOutputFilter {
    fn output(self) -> impl Iterator<Item = Device>;
}

impl<T> DeviceOutputFilter for T
where
    T: Iterator<Item = Device>,
{
    fn output(self) -> impl Iterator<Item = Device> {
        self.filter(|device_info| {
            device_info.channels.output
                >= u32::try_from(MIN_CHANNELS).expect("invalid channel minimum")
        })
        .filter(|device_info| device_info.formats.contains(DeviceFormats::F32))
        .filter(|device_info| {
            device_info
                .rates
                .available
                .contains(&u32::try_from(SAMPLE_RATE).expect("invalid sample rate"))
        })
    }
}
