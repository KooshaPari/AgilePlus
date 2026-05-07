//! Error types for configuration operations.

use thiserror::Error;

/// Errors that can occur during configuration operations.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to load configuration: {0}")]
    LoadFailed(String),

    #[error("failed to save configuration: {0}")]
    SaveFailed(String),

    #[error("configuration not found: {0}")]
    NotFound(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("invalid format: expected {expected}, found {found}")]
    InvalidFormat { expected: String, found: String },

    #[error("validation failed: {0}")]
    ValidationFailed(String),
}

/// Result type alias for configuration operations.
pub type Result<T> = std::result::Result<T, ConfigError>;
