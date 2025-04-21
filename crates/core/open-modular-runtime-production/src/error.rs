use std::result;

use snafu::Snafu;

// =================================================================================================
// Error
// =================================================================================================

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)), visibility(pub(crate)))]
pub enum Error {
    #[snafu(transparent)]
    Audio {
        source: open_modular_io_audio::Error,
    },
    #[snafu(display("internal general error: {message}"))]
    General { message: String },
}

// -------------------------------------------------------------------------------------------------

// Result

pub(crate) type Result<T> = result::Result<T, Error>;
