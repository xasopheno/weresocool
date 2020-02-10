use std::fmt;
use std::io;

use failure::Fail;
use portaudio;
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Fail, Serialize, Deserialize)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
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
}

impl<'a> From<&'a str> for Error {
    fn from(msg: &'a str) -> Error {
        Error::with_msg(msg)
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
