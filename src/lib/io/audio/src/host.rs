use rtaudio_sys::rtaudio_t;

use crate::{
    device::{
        DeviceInfo,
        DeviceInfoIterator,
    },
    system::{
        self,
        Api,
    },
};

// =================================================================================================
// Host
// =================================================================================================

#[derive(Debug)]
pub struct Host {
    audio: rtaudio_t,
}

impl Host {
    #[must_use]
    pub fn new(api: Option<Api>) -> Self {
        let api = api.unwrap_or_default();
        let handle = system::create_audio(api);

        unsafe {
            rtaudio_sys::rtaudio_show_warnings(handle, 1);
        }

        Self { audio: handle }
    }
}

impl Host {
    #[must_use]
    pub fn device(&self, device: u32) -> DeviceInfo {
        DeviceInfo::from_id(&self.audio, device)
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn devices(&self) -> impl Iterator<Item = DeviceInfo> {
        DeviceInfoIterator::new(&self.audio, unsafe {
            rtaudio_sys::rtaudio_device_count(self.audio).max(0)
        })
    }
}

impl AsRef<rtaudio_t> for Host {
    fn as_ref(&self) -> &rtaudio_t {
        &self.audio
    }
}

impl Default for Host {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        unsafe { rtaudio_sys::rtaudio_destroy(self.audio) }
    }
}
