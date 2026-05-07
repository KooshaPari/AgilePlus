//! Error types for CLI framework operations.

use thiserror::Error;

/// Errors that can occur during CLI operations.
#[derive(Error, Debug)]
pub enum CliError {
    #[error("command '{0}' not found")]
    CommandNotFound(String),

    #[error("argument '{0}' not found")]
    ArgumentNotFound(String),

    #[error("invalid argument value for '{0}': {1}")]
    InvalidValue(String, String),

    #[error("missing required argument: {0}")]
    MissingRequiredArgument(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("execution failed: {0}")]
    ExecutionFailed(String),
}

/// Result type alias for CLI operations.
pub type Result<T> = std::result::Result<T, CliError>;
