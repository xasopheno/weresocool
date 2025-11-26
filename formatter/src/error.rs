use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Parse error: {0}")]
    Parse(#[from] weresocool_error::Error),
}
