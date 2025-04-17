use std::{
    ffi::CStr,
    slice,
};

use fancy_constructor::new;
use open_modular_core::{
    MAX_CHANNELS,
    MIN_CHANNELS,
    SAMPLE_RATE,
};
use rtaudio_sys::{
    RTAUDIO_FORMAT_FLOAT32,
    RTAUDIO_FORMAT_FLOAT64,
    RTAUDIO_FORMAT_SINT8,
    RTAUDIO_FORMAT_SINT16,
    RTAUDIO_FORMAT_SINT24,
    RTAUDIO_FORMAT_SINT32,
    rtaudio_device_info,
    rtaudio_format_t,
    rtaudio_t,
};

use crate::system::{
    self,
};

// =================================================================================================
// Device
// =================================================================================================

// Info

#[derive(new, Debug)]
pub struct DeviceInfo {
    pub id: u32,
    pub name: DeviceNameInfo,
    pub channels: DeviceChannelsInfo,
    pub defaults: DeviceDefaultsInfo,
    pub formats: DeviceFormatsInfo,
    pub sample_rates: DeviceSampleRatesInfo,
}

impl DeviceInfo {
    pub(crate) fn from_id(audio: &rtaudio_t, id: u32) -> Self {
        let info = unsafe { rtaudio_sys::rtaudio_get_device_info(*audio, id) };

        system::validate_audio(*audio);

        info.into()
    }

    pub(crate) fn from_index(audio: &rtaudio_t, index: i32) -> Self {
        let id = unsafe { rtaudio_sys::rtaudio_get_device_id(*audio, index) };

        system::validate_audio(*audio);

        Self::from_id(audio, id)
    }
}

impl From<rtaudio_device_info> for DeviceInfo {
    #[allow(clippy::map_unwrap_or)]
    fn from(device_info: rtaudio_device_info) -> Self {
        let id = device_info.id;

        let name = unsafe {
            CStr::from_bytes_until_nul(slice::from_raw_parts(
                device_info.name.as_ptr().cast::<u8>(),
                device_info.name.len(),
            ))
            .map(|c_str| c_str.to_string_lossy().to_string())
            .map(|name| {
                let name = name.trim();

                name.split_once(':')
                    .map(|(company, device)| {
                        let company = company.trim();
                        let device = device.trim();

                        DeviceNameInfo::new(name, Some((company.into(), device.into())))
                    })
                    .unwrap_or_else(|| DeviceNameInfo::new(name, None))
            })
            .expect("valid device name")
        };

        // NOTE: There is a current maximum of 16 (MAX_CHANNELS) channels for any
        // duplex, input, or output device. For devices which support more than this,
        // the channel counts are artificially reported as the maximum allowable channel
        // count, which is then used to determine buffer sizes, etc.

        let channels = DeviceChannelsInfo::new(
            device_info
                .duplex_channels
                .min(u32::try_from(MAX_CHANNELS).expect("invalid channel maximum")),
            device_info
                .input_channels
                .min(u32::try_from(MAX_CHANNELS).expect("invalid channel maximum")),
            device_info
                .output_channels
                .min(u32::try_from(MAX_CHANNELS).expect("invalid channel maximum")),
        );

        let defaults = DeviceDefaultsInfo::new(
            device_info.is_default_input != 0,
            device_info.is_default_output != 0,
        );

        let formats = DeviceFormatsInfo::from_bits_truncate(device_info.native_formats);

        let sample_rates = DeviceSampleRatesInfo::new(
            device_info
                .sample_rates
                .into_iter()
                .take_while(|sample_rate| *sample_rate != 0)
                .collect(),
            device_info.preferred_sample_rate,
        );

        Self::new(id, name, channels, defaults, formats, sample_rates)
    }
}

#[derive(new, Debug)]
pub struct DeviceChannelsInfo {
    pub duplex: u32,
    pub input: u32,
    pub output: u32,
}

#[derive(new, Debug)]
pub struct DeviceDefaultsInfo {
    pub input: bool,
    pub output: bool,
}

bitflags::bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct DeviceFormatsInfo: rtaudio_format_t {
        const I8 = RTAUDIO_FORMAT_SINT8;
        const I16 = RTAUDIO_FORMAT_SINT16;
        const I24 = RTAUDIO_FORMAT_SINT24;
        const I32 = RTAUDIO_FORMAT_SINT32;
        const F32 = RTAUDIO_FORMAT_FLOAT32;
        const F64 = RTAUDIO_FORMAT_FLOAT64;
    }
}

#[derive(new, Debug)]
pub struct DeviceNameInfo {
    #[new(into)]
    pub name: String,
    pub components: Option<(String, String)>,
}

#[derive(new, Debug)]
pub struct DeviceSampleRatesInfo {
    pub available: Vec<u32>,
    pub preferred: u32,
}

// -------------------------------------------------------------------------------------------------

// Iterator

#[derive(new, Debug)]
pub(crate) struct DeviceInfoIterator<'a> {
    audio: &'a rtaudio_t,
    count: i32,
    #[new(default)]
    index: i32,
}

impl Iterator for DeviceInfoIterator<'_> {
    type Item = DeviceInfo;

    fn next(&mut self) -> Option<Self::Item> {
        match self.index {
            index if index < self.count => {
                self.index += 1;

                Some(DeviceInfo::from_index(self.audio, index))
            }
            _ => None,
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Filters

pub trait DeviceOutputFilter {
    fn output(self) -> impl Iterator<Item = DeviceInfo>;
}

impl<T> DeviceOutputFilter for T
where
    T: Iterator<Item = DeviceInfo>,
{
    fn output(self) -> impl Iterator<Item = DeviceInfo> {
        self.filter(|device_info| {
            device_info.channels.output
                >= u32::try_from(MIN_CHANNELS).expect("invalid channel minimum")
        })
        .filter(|device_info| device_info.formats.contains(DeviceFormatsInfo::F32))
        .filter(|device_info| {
            device_info
                .sample_rates
                .available
                .contains(&u32::try_from(SAMPLE_RATE).expect("invalid sample rate"))
        })
    }
}
