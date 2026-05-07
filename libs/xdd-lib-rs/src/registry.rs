//! Dialect registry for dynamic dialect resolution

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use super::dialect::{Dialect, DialectType};
use super::error::{XddError, XddResult};

/// Registry for managing multiple dialects
#[derive(Default, Clone)]
pub struct DialectRegistry {
    dialects: Arc<RwLock<HashMap<DialectType, Arc<dyn Dialect>>>>,
}

impl DialectRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry with default dialects
    pub fn with_defaults() -> Self {
        let registry = Self::new();
        registry.register_default_dialects();
        registry
    }

    /// Register a dialect
    pub fn register<D: Dialect + 'static>(&self, dialect: D) {
        let dialect_type = dialect.dialect_type();
        self.dialects
            .write()
            .insert(dialect_type, Arc::new(dialect));
    }

    /// Register the default JSON, TOML, and YAML dialects
    pub fn register_default_dialects(&self) {
        use super::dialect::{JsonDialect, TomlDialect, YamlDialect};
        self.register(JsonDialect::new());
        self.register(TomlDialect::new());
        self.register(YamlDialect::new());
    }

    /// Get a dialect by type
    pub fn get(&self, dialect_type: DialectType) -> XddResult<Arc<dyn Dialect>> {
        self.dialects
            .read()
            .get(&dialect_type)
            .cloned()
            .ok_or_else(|| XddError::DialectNotFound(format!("{:?}", dialect_type)))
    }

    /// Check if a dialect is registered
    pub fn contains(&self, dialect_type: DialectType) -> bool {
        self.dialects.read().contains_key(&dialect_type)
    }

    /// List all registered dialect types
    pub fn list(&self) -> Vec<DialectType> {
        self.dialects.read().keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::JsonDialect;

    #[test]
    fn register_and_get() {
        let registry = DialectRegistry::new();
        registry.register(JsonDialect::new());
        assert!(registry.contains(DialectType::Json));
        assert!(!registry.contains(DialectType::Toml));
    }

    #[test]
    fn with_defaults() {
        let registry = DialectRegistry::with_defaults();
        assert!(registry.contains(DialectType::Json));
        assert!(registry.contains(DialectType::Toml));
        assert!(registry.contains(DialectType::Yaml));
    }

    #[test]
    fn get_dialect() {
        let registry = DialectRegistry::with_defaults();
        let dialect = registry.get(DialectType::Json).unwrap();
        let value = dialect.parse(r#"{"test": true}"#).unwrap();
        assert_eq!(value["test"], true);
    }
}
