//! Plugin registry implementation.
//!
//! Manages plugin lifecycle: discovery, loading, initialization, and shutdown.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::{PluginError, Result};
use crate::plugin_trait::{Plugin, PluginConfig, PluginDiscovery, PluginMetadata};

/// Lifecycle state for a loaded plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin is loaded but not initialized.
    Loaded,
    /// Plugin is initialized and running.
    Running,
    /// Plugin was shut down but remains loaded.
    Stopped,
}

/// Main registry for managing plugins.
pub struct PluginRegistry {
    /// Loaded plugins indexed by name.
    plugins: Arc<RwLock<HashMap<String, Arc<dyn Plugin>>>>,
    /// Plugin metadata cache.
    metadata: Arc<RwLock<HashMap<String, PluginMetadata>>>,
    /// Plugin lifecycle states indexed by name.
    states: Arc<RwLock<HashMap<String, PluginState>>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    /// Creates a new empty plugin registry.
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns the number of currently loaded plugins.
    pub async fn len(&self) -> usize {
        self.plugins.read().await.len()
    }

    /// Returns true if no plugins are loaded.
    pub async fn is_empty(&self) -> bool {
        self.plugins.read().await.is_empty()
    }

    /// Returns a list of all loaded plugin names.
    pub async fn list_plugins(&self) -> Vec<String> {
        self.plugins.read().await.keys().cloned().collect()
    }

    /// Returns metadata for all loaded plugins.
    pub async fn get_all_metadata(&self) -> Vec<PluginMetadata> {
        self.metadata.read().await.values().cloned().collect()
    }

    /// Returns the current lifecycle state for a loaded plugin.
    pub async fn get_state(&self, name: &str) -> Option<PluginState> {
        self.states.read().await.get(name).copied()
    }

    /// Loads a plugin into the registry.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError::AlreadyLoaded`] if the plugin is already loaded.
    pub async fn load(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let name = plugin.name().to_string();
        let version = plugin.version().to_string();

        {
            let plugins = self.plugins.read().await;
            if plugins.contains_key(&name) {
                return Err(PluginError::AlreadyLoaded(name));
            }
        }

        {
            let mut metadata = self.metadata.write().await;
            metadata.insert(
                name.clone(),
                plugin.metadata().unwrap_or(PluginMetadata {
                    name: name.clone(),
                    version: version.clone(),
                    min_host_version: None,
                    description: None,
                }),
            );
        }

        {
            let mut plugins = self.plugins.write().await;
            plugins.insert(name.clone(), plugin);
        }
        {
            let mut states = self.states.write().await;
            states.insert(name.clone(), PluginState::Loaded);
        }

        info!(plugin = %name, version = %version, "plugin loaded");
        Ok(())
    }

    /// Discovers plugins through the provided discovery strategy and loads them.
    ///
    /// Returns the names of plugins successfully loaded in discovery order.
    ///
    /// # Errors
    ///
    /// Returns discovery errors from the discoverer or load errors from the registry.
    pub async fn discover_and_load<D>(&self, path: &Path, discoverer: &D) -> Result<Vec<String>>
    where
        D: PluginDiscovery,
    {
        let plugins = discoverer.discover(path).await?;
        let mut loaded_names = Vec::with_capacity(plugins.len());

        for plugin in plugins {
            let name = plugin.name().to_string();
            self.load(plugin).await?;
            loaded_names.push(name);
        }

        Ok(loaded_names)
    }

    /// Initializes a loaded plugin with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError::NotFound`] if the plugin is not loaded.
    pub async fn initialize(&self, name: &str, config: PluginConfig) -> Result<()> {
        let plugin = {
            let plugins = self.plugins.read().await;
            plugins.get(name).cloned()
        };

        let plugin = plugin.ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        if let Some(min_version) = &plugin
            .metadata()
            .as_ref()
            .and_then(|m| m.min_host_version.clone())
        {
            if config.host_version < *min_version {
                return Err(PluginError::VersionMismatch {
                    plugin: name.to_string(),
                    expected: min_version.clone(),
                    found: config.host_version.clone(),
                });
            }
        }

        debug!(plugin = %name, "initializing plugin");
        plugin.initialize(config).await.map_err(|e| {
            error!(plugin = %name, error = %e, "plugin initialization failed");
            PluginError::InitializationFailed(name.to_string(), e.to_string())
        })?;

        {
            let mut states = self.states.write().await;
            states.insert(name.to_string(), PluginState::Running);
        }

        info!(plugin = %name, "plugin initialized");
        Ok(())
    }

    /// Shuts down a loaded plugin.
    ///
    /// The plugin remains loaded but is gracefully terminated.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError::NotFound`] if the plugin is not loaded.
    pub async fn shutdown(&self, name: &str) -> Result<()> {
        let plugin = {
            let plugins = self.plugins.read().await;
            plugins.get(name).cloned()
        };

        let plugin = plugin.ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        debug!(plugin = %name, "shutting down plugin");
        plugin.shutdown().await.map_err(|e| {
            warn!(plugin = %name, error = %e, "plugin shutdown warning");
            PluginError::ShutdownFailed(name.to_string(), e.to_string())
        })?;

        {
            let mut states = self.states.write().await;
            states.insert(name.to_string(), PluginState::Stopped);
        }

        info!(plugin = %name, "plugin shutdown complete");
        Ok(())
    }

    /// Unloads a plugin from the registry.
    ///
    /// First shuts down the plugin, then removes it from the registry.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError::NotFound`] if the plugin is not loaded.
    pub async fn unload(&self, name: &str) -> Result<()> {
        self.shutdown(name).await?;

        {
            let mut plugins = self.plugins.write().await;
            plugins.remove(name);
        }
        {
            let mut metadata = self.metadata.write().await;
            metadata.remove(name);
        }
        {
            let mut states = self.states.write().await;
            states.remove(name);
        }

        info!(plugin = %name, "plugin unloaded");
        Ok(())
    }

    /// Gets a reference-counted pointer to a loaded plugin by name.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError::NotFound`] if the plugin is not loaded.
    pub async fn get(&self, name: &str) -> Result<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins
            .get(name)
            .cloned()
            .ok_or_else(|| PluginError::NotFound(name.to_string()))
    }
}

#[cfg(test)]
#[path = "registry/tests.rs"]
mod tests;
