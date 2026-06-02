use std::result;

use thiserror::Error;

pub type Result<T> = result::Result<T, DnjError>;

#[derive(Error, Debug)]
pub enum DnjError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),
}
