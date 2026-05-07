//! Error types for tracing operations.

use thiserror::Error;

/// Errors that can occur during tracing operations.
#[derive(Error, Debug)]
pub enum TraceError {
    #[error("span '{0}' not found")]
    SpanNotFound(String),

    #[error("trace context error: {0}")]
    ContextError(String),

    #[error("event recording failed: {0}")]
    EventFailed(String),

    #[error("invalid span state: {0}")]
    InvalidState(String),
}

/// Result type alias for tracing operations.
pub type Result<T> = std::result::Result<T, TraceError>;
