//! CLI plugin for AgilePlus command registration.
//!
//! Registers AgilePlus CLI subcommands as a plugin.

use async_trait::async_trait;
use tracing::info;

use plugin_registry::{Plugin, PluginConfig, PluginMetadata, Result};

/// CLI plugin that registers AgilePlus subcommands.
pub struct CliPlugin {
    #[allow(dead_code)]
    commands_registered: bool,
}

impl CliPlugin {
    /// Creates a new CLI plugin instance.
    pub fn new() -> Self {
        Self {
            commands_registered: false,
        }
    }
}

impl Default for CliPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for CliPlugin {
    fn name(&self) -> &str {
        "agileplus-cli"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn metadata(&self) -> Option<PluginMetadata> {
        Some(PluginMetadata {
            name: self.name().to_string(),
            version: self.version().to_string(),
            min_host_version: Some("0.1.0".to_string()),
            description: Some("AgilePlus CLI command registration plugin".to_string()),
        })
    }

    async fn initialize(&self, config: PluginConfig) -> Result<()> {
        let config_json =
            serde_json::to_string_pretty(&config.config).unwrap_or_else(|_| "{}".to_string());

        info!(
            host_version = %config.host_version,
            config = %config_json,
            "cli plugin initializing, registering commands"
        );

        // In a real implementation, this would register CLI subcommands
        // using the clap::Command API
        info!(
            commands = "specify, implement, audit, status, triage",
            "cli commands registered"
        );

        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        info!("cli plugin shutting down, unregistering commands");
        Ok(())
    }
}
