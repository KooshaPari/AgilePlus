//! Error types for hexkit.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HexkitError {
    #[error("port not found: {0}")]
    PortNotFound(String),

    #[error("port already registered: {0}")]
    PortAlreadyRegistered(String),

    #[error("builder validation error: {0}")]
    BuilderValidation(String),

    #[error("operation timed out after {0} seconds")]
    Timeout(u64),

    #[error("retry exhausted after {0} attempts")]
    RetryExhausted(u32),
}

pub type HexkitResult<T> = Result<T, HexkitError>;
