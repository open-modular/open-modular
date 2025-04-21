use rtaudio_sys::rtaudio_t;

use crate::{
    api::Api,
    error::{
        Result,
        RtAudioError,
    },
};

// =================================================================================================
// Audio
// =================================================================================================

#[derive(Debug)]
pub struct Audio {
    audio: rtaudio_t,
}

impl Audio {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn new(api: Api) -> Result<Self> {
        match unsafe { rtaudio_sys::rtaudio_create(api.raw()) } {
            audio if !audio.is_null() => {
                let audio = Self { audio };

                audio.run(|audio| unsafe {
                    rtaudio_sys::rtaudio_show_warnings(audio.raw(), 1);
                })?;

                Ok(audio)
            }
            _ => Err(RtAudioError::create(0, "rtaudio creation error")),
        }
    }
}

impl Audio {
    pub fn from_raw(audio: rtaudio_t) -> Self {
        Self { audio }
    }

    #[must_use]
    pub fn raw(&self) -> rtaudio_t {
        self.audio
    }
}

impl Audio {
    pub(crate) fn run<T>(&self, mut f: impl FnMut(&Audio) -> T) -> Result<T> {
        let value = f(self);

        match unsafe { rtaudio_sys::rtaudio_error_type(self.raw()) } {
            rtaudio_sys::RTAUDIO_ERROR_NONE => Ok(value),
            code => Err(RtAudioError::create(code, "rtaudio validation error")),
        }
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        // unsafe { rtaudio_sys::rtaudio_destroy(self.raw()) }
    }
}
