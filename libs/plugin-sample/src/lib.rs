//! Sample plugin demonstrating Plugin trait implementation.
//!
//! This is a reference implementation showing how to create plugins
//! that integrate with the plugin-registry.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::info;

use plugin_registry::{Plugin, PluginConfig, PluginMetadata, Result};

/// Sample plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleConfig {
    /// Greeting message to log on initialize.
    pub greeting: Option<String>,
    /// Enable verbose logging.
    pub verbose: bool,
}

impl Default for SampleConfig {
    fn default() -> Self {
        Self {
            greeting: Some("Hello from sample plugin!".to_string()),
            verbose: false,
        }
    }
}

/// Sample plugin for demonstration purposes.
pub struct SamplePlugin {
    config: SampleConfig,
}

impl SamplePlugin {
    /// Creates a new SamplePlugin with default configuration.
    pub fn new() -> Self {
        Self {
            config: SampleConfig::default(),
        }
    }

    /// Creates a new SamplePlugin with custom configuration.
    pub fn with_config(config: SampleConfig) -> Self {
        Self { config }
    }
}

impl Default for SamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for SamplePlugin {
    fn name(&self) -> &str {
        "sample"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn metadata(&self) -> Option<PluginMetadata> {
        Some(PluginMetadata {
            name: self.name().to_string(),
            version: self.version().to_string(),
            min_host_version: Some("0.1.0".to_string()),
            description: Some("Sample plugin demonstrating the Plugin trait".to_string()),
        })
    }

    async fn initialize(&self, _config: PluginConfig) -> Result<()> {
        info!(
            greeting = %self.config.greeting.as_deref().unwrap_or("Hello!"),
            verbose = self.config.verbose,
            "sample plugin initialized"
        );
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        info!("sample plugin shutting down");
        Ok(())
    }
}
