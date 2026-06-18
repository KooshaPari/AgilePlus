//! Dynamic port registry implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{HexkitError, HexkitResult};

#[derive(Debug, Clone)]
pub struct RegisteredPort {
    pub name: String,
    pub port: Arc<dyn std::any::Any + Send + Sync>,
}

#[derive(Debug, Clone, Default)]
pub struct PortRegistry {
    ports: Arc<RwLock<HashMap<String, RegisteredPort>>>,
}

impl PortRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn register(
        &self,
        key: &str,
        name: &str,
        port: Arc<dyn std::any::Any + Send + Sync>,
    ) -> HexkitResult<()> {
        let mut ports = self.ports.write().await;
        if ports.contains_key(key) {
            return Err(HexkitError::PortAlreadyRegistered(key.to_string()));
        }
        ports.insert(
            key.to_string(),
            RegisteredPort {
                name: name.to_string(),
                port,
            },
        );
        Ok(())
    }

    pub async fn get(&self, key: &str) -> HexkitResult<Arc<dyn std::any::Any + Send + Sync>> {
        let ports = self.ports.read().await;
        ports
            .get(key)
            .map(|r| r.port.clone())
            .ok_or_else(|| HexkitError::PortNotFound(key.to_string()))
    }

    pub async fn unregister(&self, key: &str) -> HexkitResult<()> {
        let mut ports = self.ports.write().await;
        ports
            .remove(key)
            .ok_or_else(|| HexkitError::PortNotFound(key.to_string()))?;
        Ok(())
    }

    pub async fn len(&self) -> usize {
        self.ports.read().await.len()
    }
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_get() {
        let registry = PortRegistry::new();
        let port: Arc<dyn std::any::Any + Send + Sync> = Arc::new("test");
        registry
            .register("storage", "TestStorage", port.clone())
            .await
            .unwrap();
        let retrieved = registry.get("storage").await.unwrap();
        assert!(Arc::ptr_eq(&port, &retrieved));
    }

    #[tokio::test]
    async fn not_found() {
        let registry = PortRegistry::new();
        assert!(registry.get("missing").await.is_err());
    }
}
