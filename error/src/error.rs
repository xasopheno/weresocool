use crate::ErrorInner;
use std::convert::From;
use std::fmt;
use wasm_bindgen::JsValue;

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
