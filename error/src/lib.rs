pub mod error;
pub mod error_inner;
pub mod id_error;
pub mod index_error;
pub mod parse_error;

#[cfg(feature = "app")]
pub mod portaudio_error;

pub use error::Error;
pub use error_inner::ErrorInner;
pub use id_error::IdError;
pub use index_error::IndexError;
pub use parse_error::ParseError;

#[cfg(feature = "app")]
use portaudio_error::PortAudioError;
use scop::ScopError;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Serialize, Deserialize)]
pub enum Serializable {
    Msg(String),
    Scope(ScopError),
    IoError(String),
    HoundError(String),
    #[cfg(feature = "app")]
    #[serde(with = "PortAudioError")]
    PortAudio(weresocool_portaudio::error::Error),
    SerdeJsonError(String),
    CsvError(String),
    ParseError(ParseError),
    IdError(IdError),
    IndexError(IndexError),
    #[cfg(all(feature = "app", not(target_os = "windows")))]
    LameError(weresocool_lame::Error),
    #[cfg(all(feature = "app", not(target_os = "windows")))]
    LameEncodeError(weresocool_lame::EncodeError),
}

impl ErrorInner {
    pub fn into_serializeable(self) -> Serializable {
        match self {
            ErrorInner::Msg(e) => Serializable::Msg(e),
            ErrorInner::ScopeError(e) => Serializable::Scope(e),
            ErrorInner::ParseError(e) => Serializable::ParseError(e),
            ErrorInner::IdError(e) => Serializable::IdError(e),
            ErrorInner::IndexError(e) => Serializable::IndexError(e),
            #[cfg(feature = "app")]
            ErrorInner::PortAudio(e) => Serializable::PortAudio(e),
            #[cfg(all(feature = "app", not(target_os = "windows")))]
            ErrorInner::LameError(e) => Serializable::LameError(e),
            #[cfg(all(feature = "app", not(target_os = "windows")))]
            ErrorInner::LameEncodeError(e) => Serializable::LameEncodeError(e),
            ErrorInner::Io(e) => {
                println!("{:#?}", e);
                Serializable::IoError("".to_string())
            }
            ErrorInner::SerdeJson(e) => {
                println!("{:#?}", e);
                Serializable::SerdeJsonError("SerdeJson Error".to_string())
            }
            ErrorInner::CsvError(e) => {
                println!("{:#?}", e);
                Serializable::CsvError("CsvError".to_string())
            }
            ErrorInner::HoundError(e) => {
                println!("{:#?}", e);
                Serializable::HoundError("HoundError".to_string())
            }
        }
    }
}

impl<'a> From<&'a str> for Error {
    fn from(msg: &'a str) -> Error {
        Error::with_msg(msg)
    }
}

impl From<IdError> for Error {
    fn from(e: IdError) -> Error {
        Error {
            inner: Box::new(ErrorInner::IdError(e)),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error {
            inner: Box::new(ErrorInner::Io(e)),
        }
    }
}

#[cfg(any(feature = "app", feature = "windows"))]
impl From<weresocool_portaudio::error::Error> for Error {
    fn from(e: weresocool_portaudio::error::Error) -> Error {
        Error {
            inner: Box::new(ErrorInner::PortAudio(e)),
        }
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Error {
        Error {
            inner: Box::new(ErrorInner::SerdeJson(e)),
        }
    }
}

impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Error {
        Error {
            inner: Box::new(ErrorInner::CsvError(e)),
        }
    }
}

impl From<scop::ScopError> for Error {
    fn from(e: scop::ScopError) -> Error {
        Error {
            inner: Box::new(ErrorInner::ScopeError(e)),
        }
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Error {
        Error {
            inner: Box::new(ErrorInner::ParseError(e)),
        }
    }
}

impl From<hound::Error> for Error {
    fn from(e: hound::Error) -> Error {
        Error {
            inner: Box::new(ErrorInner::HoundError(e)),
        }
    }
}

#[cfg(all(feature = "app", not(target_os = "windows")))]
impl From<weresocool_lame::Error> for Error {
    fn from(e: weresocool_lame::Error) -> Error {
        Error {
            inner: Box::new(ErrorInner::LameError(e)),
        }
    }
}

#[cfg(all(feature = "app", not(target_os = "windows")))]
impl From<weresocool_lame::EncodeError> for Error {
    fn from(e: weresocool_lame::EncodeError) -> Error {
        Error {
            inner: Box::new(ErrorInner::LameEncodeError(e)),
        }
    }
}

#[test]
fn size_of_error_is_one_word() {
    use std::mem;
    assert_eq!(mem::size_of::<Error>(), mem::size_of::<usize>());
}
