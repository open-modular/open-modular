use fancy_constructor::new;
use serde::{
    Deserialize,
    Serialize,
};

// =================================================================================================
// Configuration
// =================================================================================================

#[derive(new, Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    pub api: String,
    pub output: ConfigurationStream,
}

#[derive(new, Clone, Debug, Deserialize, Serialize)]
pub struct ConfigurationStream {
    pub device: u32,
    pub channels: u32,
}
