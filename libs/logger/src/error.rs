//! Error types for logger operations.

use thiserror::Error;

/// Errors that can occur during logging operations.
#[derive(Error, Debug)]
pub enum LogError {
    #[error("failed to write log: {0}")]
    WriteFailed(String),

    #[error("log output closed: {0}")]
    OutputClosed(String),

    #[error("serialization failed: {0}")]
    SerializationFailed(String),

    #[error("invalid log level: {0}")]
    InvalidLevel(String),
}

/// Result type alias for logger operations.
pub type Result<T> = std::result::Result<T, LogError>;
