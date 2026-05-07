//! The main intent registry implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::error::IntentRegistryError;
use crate::intent::ServiceIntent;
use crate::service::{RegisteredService, ServiceMetadata};

/// The intent registry - main entry point for service discovery
#[derive(Debug, Default)]
pub struct IntentRegistry {
    /// Services indexed by ID
    services: Arc<RwLock<HashMap<String, RegisteredService>>>,

    /// Services indexed by capability category
    by_category: Arc<RwLock<HashMap<String, Vec<String>>>>,

    /// Services indexed by individual action
    by_action: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl IntentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a service with its intents
    pub async fn register(
        &self,
        service_id: String,
        metadata: ServiceMetadata,
        intents: Vec<ServiceIntent>,
        endpoint: Option<String>,
    ) -> Result<(), IntentRegistryError> {
        // Check for duplicate
        {
            let services = self.services.read().await;
            if services.contains_key(&service_id) {
                return Err(IntentRegistryError::ServiceAlreadyRegistered(service_id));
            }
        }

        // Create registered service
        let mut service = RegisteredService::new(service_id.clone(), metadata);
        if let Some(ep) = endpoint {
            service = service.with_endpoint(ep);
        }

        // Register by service ID
        {
            let mut services = self.services.write().await;
            services.insert(service_id.clone(), service.clone());
        }

        // Index by capabilities
        for intent in &intents {
            // By category
            {
                let mut by_cat = self.by_category.write().await;
                by_cat
                    .entry(intent.capability.category.clone())
                    .or_default()
                    .push(service_id.clone());
            }

            // By individual actions
            for action in &intent.capability.actions {
                let mut by_act = self.by_action.write().await;
                by_act
                    .entry(action.clone())
                    .or_default()
                    .push(service_id.clone());
            }
        }

        info!(
            "Registered service '{}' with {} intents",
            service_id,
            intents.len()
        );
        Ok(())
    }

    /// Unregister a service
    pub async fn unregister(
        &self,
        service_id: &str,
    ) -> Result<RegisteredService, IntentRegistryError> {
        let service = {
            let mut services = self.services.write().await;
            services
                .remove(service_id)
                .ok_or_else(|| IntentRegistryError::ServiceNotFound(service_id.to_string()))?
        };

        // Clean up indices
        let mut by_cat = self.by_category.write().await;
        let mut by_act = self.by_action.write().await;

        for (_, ids) in by_cat.iter_mut() {
            ids.retain(|id| id != service_id);
        }
        for (_, ids) in by_act.iter_mut() {
            ids.retain(|id| id != service_id);
        }

        info!("Unregistered service '{}'", service_id);
        Ok(service)
    }

    /// Find services by capability category
    pub async fn find_by_category(&self, category: &str) -> Vec<RegisteredService> {
        let service_ids = {
            let by_cat = self.by_category.read().await;
            by_cat.get(category).cloned().unwrap_or_default()
        };

        self.get_services(service_ids).await
    }

    /// Find services by action
    pub async fn find_by_action(&self, action: &str) -> Vec<RegisteredService> {
        let service_ids = {
            let by_act = self.by_action.read().await;
            by_act.get(action).cloned().unwrap_or_default()
        };

        self.get_services(service_ids).await
    }

    /// Find a single service by ID
    pub async fn get(&self, service_id: &str) -> Option<RegisteredService> {
        let services = self.services.read().await;
        services.get(service_id).cloned()
    }

    /// List all registered services
    pub async fn list_all(&self) -> Vec<RegisteredService> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }

    /// Update service availability status
    pub async fn set_available(
        &self,
        service_id: &str,
        available: bool,
    ) -> Result<(), IntentRegistryError> {
        let mut services = self.services.write().await;
        let service = services
            .get_mut(service_id)
            .ok_or_else(|| IntentRegistryError::ServiceNotFound(service_id.to_string()))?;

        if available {
            service.mark_available();
        } else {
            service.mark_unavailable();
        }

        Ok(())
    }

    /// Get a service's intents (metadata contains category info)
    pub async fn get_service_intents(
        &self,
        service_id: &str,
    ) -> Result<Vec<String>, IntentRegistryError> {
        let services = self.services.read().await;
        let service = services
            .get(service_id)
            .ok_or_else(|| IntentRegistryError::ServiceNotFound(service_id.to_string()))?;

        // Extract categories from labels (in a real impl, this would be stored separately)
        let categories: Vec<String> = service
            .metadata
            .labels
            .get("categories")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default();

        Ok(categories)
    }

    /// Internal helper to get services by IDs
    async fn get_services(&self, ids: Vec<String>) -> Vec<RegisteredService> {
        let services = self.services.read().await;
        ids.into_iter()
            .filter_map(|id| services.get(&id).cloned())
            .filter(|s| s.is_available())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_find() {
        let registry = IntentRegistry::new();

        let metadata = ServiceMetadata::new("git-service", "1.0.0");
        let intents = vec![ServiceIntent::new(
            "vcs",
            vec!["git".to_string(), "commit".to_string(), "push".to_string()],
        )];

        registry
            .register(
                "git-svc".to_string(),
                metadata,
                intents,
                Some("http://localhost:8080".to_string()),
            )
            .await
            .unwrap();

        // Find by category
        let found = registry.find_by_category("vcs").await;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "git-svc");

        // Find by action
        let found = registry.find_by_action("commit").await;
        assert_eq!(found.len(), 1);

        // List all
        let all = registry.list_all().await;
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn test_unregister() {
        let registry = IntentRegistry::new();

        let metadata = ServiceMetadata::new("test", "1.0.0");
        let intents = vec![ServiceIntent::new("test", vec!["test".to_string()])];

        registry
            .register("test".to_string(), metadata, intents, None)
            .await
            .unwrap();

        let removed = registry.unregister("test").await.unwrap();
        assert_eq!(removed.id, "test");

        assert!(registry.get("test").await.is_none());
    }

    #[tokio::test]
    async fn test_availability() {
        let registry = IntentRegistry::new();

        let metadata = ServiceMetadata::new("test", "1.0.0");
        let intents = vec![ServiceIntent::new("test", vec!["test".to_string()])];

        registry
            .register("test".to_string(), metadata, intents, None)
            .await
            .unwrap();

        registry.set_available("test", false).await.unwrap();
        assert!(!registry.get("test").await.unwrap().is_available());

        registry.set_available("test", true).await.unwrap();
        assert!(registry.get("test").await.unwrap().is_available());
    }
}
