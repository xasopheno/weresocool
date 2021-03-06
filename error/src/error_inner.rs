use crate::{IdError, IndexError, ParseError};
use failure::Fail;
use std::io;

#[derive(Debug, Fail)]
pub enum ErrorInner {
    #[fail(display = "{}", _0)]
    Msg(String),

    #[fail(display = "I/O error: {}", _0)]
    Io(#[cause] io::Error),

    // #[fail(display = "PortAudio error: {}", _0)]
    // PortAudio(#[cause] portaudio::error::Error),
    // #[fail(display = "Cpal error: {}", _0)]
    // CPAL(#[cause] cpal::StreamError),
    #[fail(display = "SerdeJson error: {}", _0)]
    SerdeJson(#[cause] serde_json::error::Error),

    #[fail(display = "CSV error: {}", _0)]
    CSVError(#[cause] csv::Error),

    #[fail(display = "Parse error: {}", _0)]
    ParseError(#[cause] ParseError),

    #[fail(display = "Id error: {}", _0)]
    IdError(#[cause] IdError),

    #[fail(display = "Index error: {}", _0)]
    IndexError(#[cause] IndexError),

    #[fail(display = "Hound error: {}", _0)]
    HoundError(#[cause] hound::Error),
    // #[fail(display = "Lame error: {}", _0)]
    // LameError(#[cause] weresocool_lame::Error),

    // #[fail(display = "LameEncode error: {}", _0)]
    // LameEncodeError(#[cause] weresocool_lame::EncodeError),
}
