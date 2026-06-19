//! Error types for the intent registry

use thiserror::Error;

/// Errors that can occur in the intent registry
#[derive(Error, Debug)]
pub enum IntentRegistryError {
    #[error("service '{0}' not found in registry")]
    ServiceNotFound(String),

    #[error("service '{0}' already registered")]
    ServiceAlreadyRegistered(String),

    #[error("no services found matching intent '{0}'")]
    NoMatchingServices(String),

    #[error("invalid intent format: {0}")]
    InvalidIntentFormat(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("registry error: {0}")]
    RegistryError(String),
}

impl From<serde_json::Error> for IntentRegistryError {
    fn from(e: serde_json::Error) -> Self {
        IntentRegistryError::SerializationError(e.to_string())
    }
}
