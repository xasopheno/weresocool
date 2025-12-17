use crate::{Error, ErrorInner};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub struct ColorError {
    pub color: String,
    pub line: usize,
    pub column: usize,
}

impl ColorError {
    pub fn into_error(self) -> Error {
        Error {
            inner: Box::new(ErrorInner::ColorError(self)),
        }
    }
}

impl fmt::Display for ColorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Invalid color '{}'\n  → line {}, column {}",
            self.color, self.line, self.column
        )
    }
}
