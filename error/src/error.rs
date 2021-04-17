use crate::ErrorInner;
use std::fmt;
use thiserror::Error;

#[cfg(feature = "wasm")]
use std::convert::From;
#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;

#[derive(Debug, Error)]
pub struct Error {
    pub inner: Box<ErrorInner>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(feature = "wasm")]
impl From<Error> for JsValue {
    fn from(error: Error) -> Self {
        let inner = *error.inner;
        JsValue::from_serde(&inner.into_serializeable()).unwrap()
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
