mod portaudio_error;

use failure::Fail;
use portaudio;
use portaudio_error::PortAudioError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;

#[derive(Debug)]
pub struct Error {
    pub inner: Box<ErrorInner>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl failure::Fail for Error {
    fn cause(&self) -> Option<&dyn failure::Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        self.inner.backtrace()
    }
}

impl Error {
    /// Create an error with the given message.
    pub fn with_msg<S: Into<String>>(msg: S) -> Error {
        Error {
            inner: Box::new(ErrorInner::Msg(msg.into())),
        }
    }

    pub fn inner(self) -> ErrorInner {
        *self.inner
    }
}

#[derive(Debug, Fail, Serialize, Deserialize)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ParseError {
    pub fn into_error(self) -> Error {
        Error {
            inner: Box::new(ErrorInner::ParseError(self)),
        }
    }
}

#[derive(Debug, Fail, Serialize, Deserialize)]
pub struct IdError {
    pub id: String,
}

#[derive(Debug, Fail, Serialize, Deserialize)]
pub struct IndexError {
    pub len_list: usize,
    pub index: usize,
    pub message: String,
}

impl IndexError {
    pub fn as_error(self) -> Error {
        Error {
            inner: Box::new(ErrorInner::IndexError(self)),
        }
    }
}

impl IndexError {
    pub fn into_error(self) -> Error {
        Error {
            inner: Box::new(ErrorInner::IndexError(self)),
        }
    }
}

impl IdError {
    pub fn into_error(self) -> Error {
        Error {
            inner: Box::new(ErrorInner::IdError(self)),
        }
    }
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not find id: {}", self.id)
    }
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "message: {}, line: {}, column: {}",
            self.message, self.line, self.column
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Serializable {
    Msg(String),
    IoError(String),
    #[serde(with = "PortAudioError")]
    PortAudio(portaudio::error::Error),
    SerdeJsonError(String),
    CSVError(String),
    ParseError(ParseError),
    IdError(IdError),
    IndexError(IndexError),
}

impl ErrorInner {
    pub fn into_serializeable(self) -> Serializable {
        match self {
            ErrorInner::Msg(e) => Serializable::Msg(e),
            ErrorInner::ParseError(e) => Serializable::ParseError(e),
            ErrorInner::IdError(e) => Serializable::IdError(e),
            ErrorInner::IndexError(e) => Serializable::IndexError(e),
            ErrorInner::Io(e) => {
                println!("{:#?}", e);
                Serializable::IoError("".to_string())
            }
            ErrorInner::SerdeJson(e) => {
                println!("{:#?}", e);
                Serializable::SerdeJsonError("SerdeJson Error".to_string())
            }
            ErrorInner::CSVError(e) => {
                println!("{:#?}", e);
                Serializable::CSVError("CSVError".to_string())
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Fail)]
pub enum ErrorInner {
    #[fail(display = "{}", _0)]
    Msg(String),

    #[fail(display = "I/O error: {}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "PortAudio error: {}", _0)]
    PortAudio(#[cause] portaudio::error::Error),

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

impl From<IndexError> for Error {
    fn from(e: IndexError) -> Error {
        Error {
            inner: Box::new(ErrorInner::IndexError(e)),
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

impl From<portaudio::error::Error> for Error {
    fn from(e: portaudio::error::Error) -> Error {
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
            inner: Box::new(ErrorInner::CSVError(e)),
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

#[test]
fn size_of_error_is_one_word() {
    use std::mem;
    assert_eq!(mem::size_of::<Error>(), mem::size_of::<usize>());
}
