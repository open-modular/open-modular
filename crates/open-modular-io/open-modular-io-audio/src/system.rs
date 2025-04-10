use std::{
    ffi::CStr,
    mem,
    slice,
};

use rtaudio_sys::{
    RTAUDIO_API_DUMMY,
    RTAUDIO_API_LINUX_ALSA,
    RTAUDIO_API_LINUX_OSS,
    RTAUDIO_API_LINUX_PULSE,
    RTAUDIO_API_MACOSX_CORE,
    RTAUDIO_API_UNIX_JACK,
    RTAUDIO_API_UNSPECIFIED,
    RTAUDIO_API_WINDOWS_ASIO,
    RTAUDIO_API_WINDOWS_DS,
    RTAUDIO_API_WINDOWS_WASAPI,
    RTAUDIO_ERROR_NONE,
    rtaudio_t,
};

use crate::host::Host;

// =================================================================================================
// System
// =================================================================================================

// API

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Api {
    ALSA = RTAUDIO_API_LINUX_ALSA,
    ASIO = RTAUDIO_API_WINDOWS_ASIO,
    CoreAudio = RTAUDIO_API_MACOSX_CORE,
    DirectSound = RTAUDIO_API_WINDOWS_DS,
    Dummy = RTAUDIO_API_DUMMY,
    Jack = RTAUDIO_API_UNIX_JACK,
    OSS = RTAUDIO_API_LINUX_OSS,
    Pulse = RTAUDIO_API_LINUX_PULSE,
    Unspecified = RTAUDIO_API_UNSPECIFIED,
    WASAPI = RTAUDIO_API_WINDOWS_WASAPI,
}

impl Api {
    fn from_i32(api: i32) -> Self {
        match api {
            api if (0..=9).contains(&api) => unsafe { mem::transmute::<i32, Self>(api) },
            _ => panic!(""),
        }
    }
}

impl Api {
    #[must_use]
    pub fn apis() -> Vec<Self> {
        unsafe {
            match (
                rtaudio_sys::rtaudio_compiled_api(),
                rtaudio_sys::rtaudio_get_num_compiled_apis(),
            ) {
                (apis, n) if !apis.is_null() && n > 0 => slice::from_raw_parts(apis, n as usize)
                    .iter()
                    .map(|api| Api::from_i32(*api))
                    .collect(),
                _ => Vec::new(),
            }
        }
    }
}

impl Default for Api {
    fn default() -> Self {
        Self::Unspecified
    }
}

impl From<Option<Api>> for Api {
    fn from(api: Option<Api>) -> Self {
        api.unwrap_or_default()
    }
}

impl From<&Host> for Api {
    fn from(host: &Host) -> Self {
        host.api()
    }
}

// -------------------------------------------------------------------------------------------------

// Audio

pub(crate) fn create_audio(api: Api) -> rtaudio_t {
    unsafe {
        match rtaudio_sys::rtaudio_create(api as i32) {
            audio if !audio.is_null() => validate_audio(audio),
            _ => panic!("could not create audio"),
        }
    }
}

pub(crate) fn validate_audio(audio: rtaudio_t) -> rtaudio_t {
    match unsafe { rtaudio_sys::rtaudio_error_type(audio) } {
        RTAUDIO_ERROR_NONE => audio,
        err => panic!("audio error: {err}"),
    }
}

// -------------------------------------------------------------------------------------------------

// Get API

pub trait GetApi {
    fn api(&self) -> Api;
}

impl<T> GetApi for T
where
    T: AsRef<rtaudio_t>,
{
    fn api(&self) -> Api {
        unsafe { mem::transmute::<i32, Api>(rtaudio_sys::rtaudio_current_api(*self.as_ref())) }
    }
}

// -------------------------------------------------------------------------------------------------

// Version

#[must_use]
pub fn rtaudio_version() -> String {
    unsafe {
        match rtaudio_sys::rtaudio_version() {
            version if !version.is_null() => CStr::from_ptr(version.cast_mut())
                .to_string_lossy()
                .to_string(),
            _ => "unknown".into(),
        }
    }
}
