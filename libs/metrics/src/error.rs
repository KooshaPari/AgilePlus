//! Error types for metrics operations.

use thiserror::Error;

/// Errors that can occur during metrics operations.
#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("metric '{0}' not found")]
    NotFound(String),

    #[error("metric '{0}' already exists")]
    AlreadyExists(String),

    #[error("invalid metric value: {0}")]
    InvalidValue(String),

    #[error("serialization failed: {0}")]
    SerializationFailed(String),
}

/// Result type alias for metrics operations.
pub type Result<T> = std::result::Result<T, MetricsError>;
