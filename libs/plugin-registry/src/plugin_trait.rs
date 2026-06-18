//! Plugin trait definition.
//!
//! All plugins must implement this trait to be managed by the registry.

use async_trait::async_trait;

use crate::error::Result;

/// Configuration passed to a plugin during initialization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginConfig {
    /// Plugin-specific configuration as JSON.
    pub config: serde_json::Value,
    /// Path to plugin data directory.
    pub data_dir: Option<std::path::PathBuf>,
    /// Host version for compatibility checking.
    pub host_version: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            config: serde_json::Value::Null,
            data_dir: None,
            host_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Metadata about a plugin.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginMetadata {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Minimum host version required.
    pub min_host_version: Option<String>,
    /// Plugin description.
    pub description: Option<String>,
}

/// Core trait that all plugins must implement.
///
/// Implementors must be `Send + Sync` to allow safe concurrent access
/// from the registry.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Returns the plugin's name.
    fn name(&self) -> &str;

    /// Returns the plugin's version.
    fn version(&self) -> &str;

    /// Returns optional metadata about the plugin.
    fn metadata(&self) -> Option<PluginMetadata> {
        None
    }

    /// Initializes the plugin with the given configuration.
    ///
    /// Called once when the plugin is loaded.
    async fn initialize(&self, config: PluginConfig) -> Result<()>;

    /// Shuts down the plugin gracefully.
    ///
    /// Called when the plugin is unloaded or the host is shutting down.
    async fn shutdown(&self) -> Result<()>;
}
