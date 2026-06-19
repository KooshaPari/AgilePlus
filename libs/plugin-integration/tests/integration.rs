//! Integration tests for plugin-integration.
//!
//! These tests verify end-to-end plugin lifecycle management.

use std::sync::Arc;

use plugin_integration::{PluginConfig, PluginError, PluginMetadata, UnifiedPluginRegistry};

// Test helper: Create a test plugin
struct TestPlugin {
    name: String,
    version: String,
}

#[async_trait::async_trait]
impl plugin_integration::Plugin for TestPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn metadata(&self) -> Option<PluginMetadata> {
        Some(PluginMetadata {
            name: self.name.clone(),
            version: self.version.clone(),
            description: Some(format!("Test plugin: {}", self.name)),
            min_host_version: None,
        })
    }

    async fn initialize(&self, _config: PluginConfig) -> Result<(), PluginError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[tokio::test]
async fn test_plugin_registry_empty_state() {
    let registry = UnifiedPluginRegistry::new();
    assert!(registry.list_plugins().await.is_empty());
}

#[tokio::test]
async fn test_plugin_load_and_list() {
    let registry = UnifiedPluginRegistry::new();

    let plugin = Arc::new(TestPlugin {
        name: "test-plugin".to_string(),
        version: "1.0.0".to_string(),
    });

    let config = PluginConfig::default();
    registry.load_plugin(plugin.clone(), config).await.unwrap();

    let plugins = registry.list_plugins().await;
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0], "test-plugin");
}

#[tokio::test]
async fn test_plugin_get() {
    let registry = UnifiedPluginRegistry::new();

    let plugin = Arc::new(TestPlugin {
        name: "get-test".to_string(),
        version: "2.0.0".to_string(),
    });

    let config = PluginConfig::default();
    registry.load_plugin(plugin.clone(), config).await.unwrap();

    let retrieved = registry.get_plugin("get-test").await.unwrap();
    assert_eq!(retrieved.name(), "get-test");
    assert_eq!(retrieved.version(), "2.0.0");
}

#[tokio::test]
async fn test_plugin_not_found() {
    let registry = UnifiedPluginRegistry::new();

    let result = registry.get_plugin("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_plugin_shutdown() {
    let registry = UnifiedPluginRegistry::new();

    let plugin = Arc::new(TestPlugin {
        name: "shutdown-test".to_string(),
        version: "1.0.0".to_string(),
    });

    let config = PluginConfig::default();
    registry.load_plugin(plugin.clone(), config).await.unwrap();

    registry.shutdown_plugin("shutdown-test").await.unwrap();

    // Plugin should still be listed after shutdown
    let plugins = registry.list_plugins().await;
    assert_eq!(plugins.len(), 1);
}

#[tokio::test]
async fn test_plugin_unload() {
    let registry = UnifiedPluginRegistry::new();

    let plugin = Arc::new(TestPlugin {
        name: "unload-test".to_string(),
        version: "1.0.0".to_string(),
    });

    let config = PluginConfig::default();
    registry.load_plugin(plugin.clone(), config).await.unwrap();

    registry.unload_plugin("unload-test").await.unwrap();

    assert!(registry.list_plugins().await.is_empty());
}

#[tokio::test]
async fn test_multiple_plugins() {
    let registry = UnifiedPluginRegistry::new();

    // Load multiple plugins
    for i in 1..=3 {
        let plugin = Arc::new(TestPlugin {
            name: format!("plugin-{}", i),
            version: "1.0.0".to_string(),
        });
        let config = PluginConfig::default();
        registry.load_plugin(plugin, config).await.unwrap();
    }

    let plugins = registry.list_plugins().await;
    assert_eq!(plugins.len(), 3);
}

#[tokio::test]
async fn test_duplicate_plugin_rejected() {
    let registry = UnifiedPluginRegistry::new();

    let plugin = Arc::new(TestPlugin {
        name: "duplicate-test".to_string(),
        version: "1.0.0".to_string(),
    });

    let config = PluginConfig::default();
    registry.load_plugin(plugin.clone(), config).await.unwrap();
    // Try to load again - should fail
    let result = registry
        .load_plugin(plugin.clone(), PluginConfig::default())
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_unload_nonexistent() {
    let registry = UnifiedPluginRegistry::new();

    let result = registry.unload_plugin("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_plugin_metadata() {
    let registry = UnifiedPluginRegistry::new();

    let plugin = Arc::new(TestPlugin {
        name: "metadata-test".to_string(),
        version: "3.0.0".to_string(),
    });

    let config = PluginConfig::default();
    registry.load_plugin(plugin.clone(), config).await.unwrap();

    let retrieved = registry.get_plugin("metadata-test").await.unwrap();
    let metadata = retrieved.metadata().unwrap();

    assert_eq!(metadata.name, "metadata-test");
    assert_eq!(metadata.version, "3.0.0");
    assert!(metadata.description.is_some());
}
