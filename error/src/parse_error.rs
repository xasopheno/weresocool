use crate::error::Error;
use crate::error_inner::ErrorInner;
use failure::Fail;
use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "message: {}, line: {}, column: {}",
            self.message, self.line, self.column
        )
    }
}
