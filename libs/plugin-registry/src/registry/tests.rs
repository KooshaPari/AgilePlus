use std::path::Path;
use std::sync::Arc;

use super::{PluginRegistry, PluginState};
use crate::error::{PluginError, Result};
use crate::plugin_trait::{Plugin, PluginConfig, PluginDiscovery, PluginMetadata};

struct TestPlugin {
    name: String,
    version: String,
    metadata: Option<PluginMetadata>,
}

impl TestPlugin {
    fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            metadata: None,
        }
    }

    fn with_metadata(mut self, metadata: PluginMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

fn metadata(
    name: &str,
    version: &str,
    min_host_version: Option<&str>,
    description: Option<&str>,
) -> PluginMetadata {
    PluginMetadata {
        name: name.to_string(),
        version: version.to_string(),
        min_host_version: min_host_version.map(str::to_string),
        description: description.map(str::to_string),
    }
}

struct StaticDiscovery {
    plugins: Vec<Arc<dyn Plugin>>,
}

#[async_trait::async_trait]
impl PluginDiscovery for StaticDiscovery {
    async fn discover(&self, _path: &Path) -> Result<Vec<Arc<dyn Plugin>>> {
        Ok(self.plugins.clone())
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

    fn metadata(&self) -> Option<PluginMetadata> {
        self.metadata.clone()
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

    assert!(registry.is_empty().await);

    let plugin = Arc::new(TestPlugin::new("test-plugin", "1.0.0"));
    registry.load(plugin).await.unwrap();

    assert_eq!(registry.len().await, 1);
    assert!(!registry.is_empty().await);

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

#[tokio::test]
async fn test_discover_and_load_plugins() {
    let registry = PluginRegistry::new();
    let discoverer = StaticDiscovery {
        plugins: vec![
            Arc::new(TestPlugin::new("plugin-one", "1.0.0")),
            Arc::new(TestPlugin::new("plugin-two", "1.0.0")),
        ],
    };

    let loaded_names = registry
        .discover_and_load(Path::new("plugins"), &discoverer)
        .await
        .unwrap();

    assert_eq!(loaded_names, vec!["plugin-one", "plugin-two"]);
    assert_eq!(registry.len().await, 2);
    assert!(registry.get("plugin-one").await.is_ok());
    assert!(registry.get("plugin-two").await.is_ok());
}

#[tokio::test]
async fn test_state_transitions() {
    let registry = PluginRegistry::new();
    let plugin = Arc::new(TestPlugin::new("lifecycle-plugin", "1.0.0"));

    registry.load(plugin).await.unwrap();
    assert_eq!(
        registry.get_state("lifecycle-plugin").await,
        Some(PluginState::Loaded)
    );
    assert!(
        registry
            .list_plugins()
            .await
            .contains(&"lifecycle-plugin".to_string())
    );

    registry
        .initialize("lifecycle-plugin", PluginConfig::default())
        .await
        .unwrap();
    assert_eq!(
        registry.get_state("lifecycle-plugin").await,
        Some(PluginState::Running)
    );

    registry.shutdown("lifecycle-plugin").await.unwrap();
    assert_eq!(
        registry.get_state("lifecycle-plugin").await,
        Some(PluginState::Stopped)
    );

    registry.unload("lifecycle-plugin").await.unwrap();
    assert_eq!(registry.get_state("lifecycle-plugin").await, None);
    assert!(
        !registry
            .list_plugins()
            .await
            .contains(&"lifecycle-plugin".to_string())
    );
}

#[tokio::test]
async fn test_version_mismatch() {
    let registry = PluginRegistry::new();
    let plugin = Arc::new(
        TestPlugin::new("future-plugin", "1.0.0").with_metadata(metadata(
            "future-plugin",
            "1.0.0",
            Some("99.0.0"),
            None,
        )),
    );

    registry.load(plugin).await.unwrap();

    let result = registry
        .initialize("future-plugin", PluginConfig::default())
        .await;

    assert!(matches!(
        result,
        Err(PluginError::VersionMismatch {
            plugin,
            expected,
            ..
        }) if plugin == "future-plugin" && expected == "99.0.0"
    ));
    assert_eq!(
        registry.get_state("future-plugin").await,
        Some(PluginState::Loaded)
    );
}

#[tokio::test]
async fn test_plugin_metadata() {
    let registry = PluginRegistry::new();
    let plugin = Arc::new(
        TestPlugin::new("metadata-plugin", "2.1.0").with_metadata(metadata(
            "metadata-plugin",
            "2.1.0",
            Some("1.2.3"),
            Some("plugin used for metadata tests"),
        )),
    );

    registry.load(plugin).await.unwrap();

    let metadata = registry.get_all_metadata().await;
    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].name, "metadata-plugin");
    assert_eq!(metadata[0].version, "2.1.0");
    assert_eq!(metadata[0].min_host_version.as_deref(), Some("1.2.3"));
    assert_eq!(
        metadata[0].description.as_deref(),
        Some("plugin used for metadata tests")
    );
}

#[tokio::test]
async fn test_shutdown_nonexistent() {
    let registry = PluginRegistry::new();

    let result = registry.shutdown("missing-plugin").await;

    assert!(matches!(result, Err(PluginError::NotFound(plugin)) if plugin == "missing-plugin"));
}

#[tokio::test]
async fn test_concurrent_load() {
    let registry = PluginRegistry::new();

    let (first, second, third) = tokio::join!(
        registry.load(Arc::new(TestPlugin::new("plugin-one", "1.0.0"))),
        registry.load(Arc::new(TestPlugin::new("plugin-two", "1.0.0"))),
        registry.load(Arc::new(TestPlugin::new("plugin-three", "1.0.0"))),
    );

    first.unwrap();
    second.unwrap();
    third.unwrap();

    let mut loaded = registry.list_plugins().await;
    loaded.sort();

    assert_eq!(loaded, vec!["plugin-one", "plugin-three", "plugin-two"]);
    assert_eq!(registry.len().await, 3);
}
