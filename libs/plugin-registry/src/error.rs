//! Error types for plugin registry operations.

use thiserror::Error;

/// Errors that can occur during plugin operations.
#[derive(Debug, Error)]
pub enum PluginError {
    /// Plugin not found in registry.
    #[error("plugin not found: {0}")]
    NotFound(String),

    /// Plugin already loaded.
    #[error("plugin already loaded: {0}")]
    AlreadyLoaded(String),

    /// Plugin initialization failed.
    #[error("failed to initialize plugin '{0}': {1}")]
    InitializationFailed(String, String),

    /// Plugin shutdown failed.
    #[error("failed to shutdown plugin '{0}': {1}")]
    ShutdownFailed(String, String),

    /// Invalid plugin configuration.
    #[error("invalid plugin configuration: {0}")]
    InvalidConfig(String),

    /// Discovery failed.
    #[error("failed to discover plugins: {0}")]
    DiscoveryFailed(String),

    /// Version mismatch between host and plugin.
    #[error("version mismatch for plugin '{plugin}': expected {expected}, found {found}")]
    VersionMismatch {
        plugin: String,
        expected: String,
        found: String,
    },
}

/// Result type alias for plugin operations.
pub type Result<T> = std::result::Result<T, PluginError>;
