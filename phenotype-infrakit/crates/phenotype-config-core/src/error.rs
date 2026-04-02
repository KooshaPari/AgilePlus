//! Configuration error types

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    #[error("invalid value: {0}")]
    InvalidValue(String),
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("unsupported value type: {0}")]
    UnsupportedType(String),
}

pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
