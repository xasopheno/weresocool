use crate::{IdError, IndexError, ParseError};
use scop::ScopError;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorInner {
    #[error("{0}")]
    Msg(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[cfg(feature = "app")]
    #[error("PortAudio error: {0}")]
    PortAudio(#[from] weresocool_portaudio::error::Error),

    #[error("SerdeJson error: {0}")]
    SerdeJson(#[from] serde_json::error::Error),

    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("Id error: {0}")]
    ScopeError(#[from] ScopError),

    #[error("Id error: {0}")]
    IdError(#[from] IdError),

    #[error("Index error: {0}")]
    IndexError(#[from] IndexError),

    #[error("Hound error: {0}")]
    HoundError(#[from] hound::Error),
    #[cfg(all(feature = "app", not(target_os = "windows")))]
    #[error("Lame error: {0}")]
    LameError(#[from] weresocool_lame::Error),

    #[cfg(all(feature = "app", not(target_os = "windows")))]
    #[error("LameEncode error: {0}")]
    LameEncodeError(#[from] weresocool_lame::EncodeError),
}
