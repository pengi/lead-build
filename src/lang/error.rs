use std::result;

use super::{expr, immap, parser};

pub type Result<T> = result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ExprSet error: {0}")]
    ImMapError(#[from] immap::Error),

    #[error("Parse error: {0}")]
    ParseError(#[from] parser::Error),

    #[error("Expression error: {0}")]
    ExprError(#[from] expr::Error),

    #[error("{0}")]
    CustomError(String),
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::CustomError(value)
    }
}
