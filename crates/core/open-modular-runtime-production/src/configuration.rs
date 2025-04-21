use fancy_constructor::new;
use serde::{
    Deserialize,
    Serialize,
};

// =================================================================================================
// Configuration
// =================================================================================================

#[derive(new, Debug, Deserialize, Serialize)]
pub struct Configuration {
    pub audio: open_modular_io_audio::Configuration,
}
