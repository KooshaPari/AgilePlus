//! Error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum XddError {
    #[error("parse error: {0}")]
    ParseError(String),

    #[error("convert error: {0}")]
    ConvertError(String),

    #[error("unsupported dialect: {0}")]
    UnsupportedDialect(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("dialect not registered: {0}")]
    DialectNotFound(String),
}

pub type XddResult<T> = Result<T, XddError>;
