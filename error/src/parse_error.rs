use crate::{Error, ErrorInner};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
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
