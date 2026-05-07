//! Git plugin for AgilePlus repository operations.
//!
//! Provides git operations: branch management, worktree handling, and repo discovery.

use async_trait::async_trait;
use std::path::PathBuf;
use tracing::info;

use plugin_registry::{Plugin, PluginConfig, PluginMetadata, Result};

/// Git plugin configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GitConfig {
    /// Default branch name.
    pub default_branch: Option<String>,
    /// Path to search for git repos.
    pub search_path: Option<PathBuf>,
    /// Enable worktree support.
    pub worktree_enabled: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            default_branch: Some("main".to_string()),
            search_path: None,
            worktree_enabled: true,
        }
    }
}

/// Git plugin providing repository operations.
pub struct GitPlugin {
    config: GitConfig,
}

impl GitPlugin {
    /// Creates a new Git plugin with default configuration.
    pub fn new() -> Self {
        Self {
            config: GitConfig::default(),
        }
    }

    /// Creates a new Git plugin with custom configuration.
    pub fn with_config(config: GitConfig) -> Self {
        Self { config }
    }
}

impl Default for GitPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for GitPlugin {
    fn name(&self) -> &str {
        "agileplus-git"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn metadata(&self) -> Option<PluginMetadata> {
        Some(PluginMetadata {
            name: self.name().to_string(),
            version: self.version().to_string(),
            min_host_version: Some("0.1.0".to_string()),
            description: Some("Git operations plugin for AgilePlus".to_string()),
        })
    }

    async fn initialize(&self, config: PluginConfig) -> Result<()> {
        let data_dir = config
            .data_dir
            .as_ref()
            .map(|p: &std::path::PathBuf| p.display().to_string())
            .unwrap_or_else(|| "<none>".to_string());

        info!(
            default_branch = %self.config.default_branch.as_deref().unwrap_or("main"),
            search_path = %self.config.search_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "<none>".to_string()),
            worktree_enabled = self.config.worktree_enabled,
            data_dir = %data_dir,
            "git plugin initialized"
        );

        // In a real implementation, this would:
        // - Initialize git2/Gix context
        // - Set up worktree watchers
        // - Configure repository search paths

        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        info!("git plugin shutting down");
        Ok(())
    }
}
