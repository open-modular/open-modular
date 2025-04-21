use std::result;

use snafu::Snafu;

// =================================================================================================
// Error
// =================================================================================================

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)), visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("internal general error: {message}"))]
    General { message: String },
    #[snafu(display("internal parameterisation error: {message}"))]
    Parameterisation { message: String },
    #[snafu(display("internal rtaudio error: {code} ({message})"))]
    RtAudio { code: i32, message: String },
}

impl<T> GeneralError<T>
where
    T: Into<String>,
{
    pub fn create(message: T) -> Error {
        Self { message }.build()
    }
}

impl<T> ParameterisationError<T>
where
    T: Into<String>,
{
    pub fn create(message: T) -> Error {
        Self { message }.build()
    }
}

impl<T, U> RtAudioError<T, U>
where
    T: Into<i32>,
    U: Into<String>,
{
    pub fn create(code: T, message: U) -> Error {
        Self { code, message }.build()
    }
}

// -------------------------------------------------------------------------------------------------

// Result

pub(crate) type Result<T> = result::Result<T, Error>;
