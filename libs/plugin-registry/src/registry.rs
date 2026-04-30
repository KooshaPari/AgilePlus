//! Plugin registry implementation.
//!
//! Manages plugin lifecycle: discovery, loading, initialization, and shutdown.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::{PluginError, Result};
use crate::plugin_trait::{Plugin, PluginConfig, PluginMetadata};

/// Main registry for managing plugins.
pub struct PluginRegistry {
    /// Loaded plugins indexed by name.
    plugins: Arc<RwLock<HashMap<String, Arc<dyn Plugin>>>>,
    /// Plugin metadata cache.
    metadata: Arc<RwLock<HashMap<String, PluginMetadata>>>,
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

    /// Loads a plugin into the registry.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError::AlreadyLoaded`] if the plugin is already loaded.
    pub async fn load(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let name = plugin.name().to_string();
        let version = plugin.version().to_string();

        // Check if already loaded
        {
            let plugins = self.plugins.read().await;
            if plugins.contains_key(&name) {
                return Err(PluginError::AlreadyLoaded(name));
            }
        }

        // Store metadata
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

        // Store plugin
        {
            let mut plugins = self.plugins.write().await;
            plugins.insert(name.clone(), plugin);
        }

        info!(plugin = %name, version = %version, "plugin loaded");
        Ok(())
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

        // Version check
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
        // Shutdown first
        self.shutdown(name).await?;

        // Remove from registry
        {
            let mut plugins = self.plugins.write().await;
            plugins.remove(name);
        }
        {
            let mut metadata = self.metadata.write().await;
            metadata.remove(name);
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
mod tests {
    use super::*;
    use crate::plugin_trait::PluginConfig;

    struct TestPlugin {
        name: String,
        version: String,
    }

    impl TestPlugin {
        fn new(name: &str, version: &str) -> Self {
            Self {
                name: name.to_string(),
                version: version.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            &self.version
        }

        async fn initialize(&self, _config: PluginConfig) -> Result<()> {
            Ok(())
        }

        async fn shutdown(&self) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_load_unload_plugin() {
        let registry = PluginRegistry::new();

        // Initially empty
        assert!(registry.is_empty().await);

        // Load a plugin
        let plugin = Arc::new(TestPlugin::new("test-plugin", "1.0.0"));
        registry.load(plugin).await.unwrap();

        assert_eq!(registry.len().await, 1);
        assert!(!registry.is_empty().await);

        // Unload the plugin
        registry.unload("test-plugin").await.unwrap();

        assert!(registry.is_empty().await);
    }

    #[tokio::test]
    async fn test_load_duplicate_plugin() {
        let registry = PluginRegistry::new();

        let plugin1 = Arc::new(TestPlugin::new("test-plugin", "1.0.0"));
        registry.load(plugin1).await.unwrap();

        let plugin2 = Arc::new(TestPlugin::new("test-plugin", "2.0.0"));
        let result = registry.load(plugin2).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::AlreadyLoaded(_)));
    }

    #[tokio::test]
    async fn test_initialize_nonexistent_plugin() {
        let registry = PluginRegistry::new();

        let result = registry
            .initialize("nonexistent", PluginConfig::default())
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_get_plugin() {
        let registry = PluginRegistry::new();

        let plugin = Arc::new(TestPlugin::new("test-plugin", "1.0.0"));
        registry.load(plugin).await.unwrap();

        let retrieved = registry.get("test-plugin").await.unwrap();
        assert_eq!(retrieved.name(), "test-plugin");
        assert_eq!(retrieved.version(), "1.0.0");
    }
}
