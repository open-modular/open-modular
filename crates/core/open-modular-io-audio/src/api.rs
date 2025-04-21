use std::{
    ffi::{
        CStr,
        CString,
    },
    slice,
};

use num_enum::{
    IntoPrimitive,
    TryFromPrimitive,
};
use rtaudio_sys::rtaudio_api_t;

use crate::error::{
    Error,
    GeneralError,
    Result,
};

// =================================================================================================
// System
// =================================================================================================

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
pub enum Api {
    ALSA = rtaudio_sys::RTAUDIO_API_LINUX_ALSA,
    ASIO = rtaudio_sys::RTAUDIO_API_WINDOWS_ASIO,
    CoreAudio = rtaudio_sys::RTAUDIO_API_MACOSX_CORE,
    DirectSound = rtaudio_sys::RTAUDIO_API_WINDOWS_DS,
    Dummy = rtaudio_sys::RTAUDIO_API_DUMMY,
    Jack = rtaudio_sys::RTAUDIO_API_UNIX_JACK,
    OpenSoundSystem = rtaudio_sys::RTAUDIO_API_LINUX_OSS,
    Pulse = rtaudio_sys::RTAUDIO_API_LINUX_PULSE,
    Unknown = rtaudio_sys::RTAUDIO_API_UNSPECIFIED,
    WASAPI = rtaudio_sys::RTAUDIO_API_WINDOWS_WASAPI,
}

impl Api {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn available() -> Result<impl Iterator<Item = Result<Self>>> {
        match unsafe {
            (
                rtaudio_sys::rtaudio_compiled_api(),
                rtaudio_sys::rtaudio_get_num_compiled_apis(),
            )
        } {
            (apis, n) if !apis.is_null() && n > 0 => {
                Ok(unsafe { slice::from_raw_parts(apis, n as usize) }
                    .iter()
                    .map(|api| Api::from_raw(*api)))
            }
            _ => Err(GeneralError::create("no apis available")),
        }
    }
}

impl Api {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn from_raw(api: rtaudio_api_t) -> Result<Self> {
        Api::try_from(api).map_err(|_| GeneralError::create("api not found"))
    }

    #[must_use]
    pub fn raw(self) -> rtaudio_api_t {
        Api::into(self)
    }
}

impl Api {
    /// Returns the display name of this [`Api`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn display_name(&self) -> Result<String> {
        match unsafe { rtaudio_sys::rtaudio_api_display_name(self.raw()) } {
            name if !name.is_null() => Ok(unsafe { CStr::from_ptr(name.cast_mut()) }
                .to_string_lossy()
                .to_string()),
            _ => Err(GeneralError::create("display name not defined")),
        }
    }

    /// Returns the name of this [`Api`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn name(&self) -> Result<String> {
        match unsafe { rtaudio_sys::rtaudio_api_name(self.raw()) } {
            name if !name.is_null() => Ok(unsafe { CStr::from_ptr(name.cast_mut()) }
                .to_string_lossy()
                .to_string()),
            _ => Err(GeneralError::create("name not defined")),
        }
    }
}

impl Default for Api {
    fn default() -> Self {
        Self::Unknown
    }
}

impl TryFrom<&str> for Api {
    type Error = Error;

    fn try_from(name: &str) -> Result<Self> {
        let name = CString::new(name).map_err(|_| GeneralError::create("invalid api name"))?;
        let name_ptr = name.as_ptr();
        let api = unsafe { rtaudio_sys::rtaudio_compiled_api_by_name(name_ptr) };
        let api = Self::from_raw(api)?;

        Ok(api)
    }
}

impl TryFrom<String> for Api {
    type Error = Error;

    fn try_from(name: String) -> Result<Self> {
        Self::try_from(name.as_str())
    }
}
