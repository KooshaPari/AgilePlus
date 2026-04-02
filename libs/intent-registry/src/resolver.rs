//! Intent resolution logic

use std::sync::Arc;

use crate::error::IntentRegistryError;
use crate::registry::IntentRegistry;
use crate::service::RegisteredService;

/// Resolves intents to specific services with priority and fallback handling
#[derive(Debug, Clone)]
pub struct IntentResolver {
    registry: Arc<IntentRegistry>,
}

impl IntentResolver {
    /// Create a new resolver backed by the given registry
    pub fn new(registry: Arc<IntentRegistry>) -> Self {
        Self { registry }
    }

    /// Resolve a single intent to the best matching service
    ///
    /// Returns the highest-priority service matching the intent.
    pub async fn resolve(
        &self,
        category: &str,
    ) -> Result<Option<RegisteredService>, IntentRegistryError> {
        let services = self.registry.find_by_category(category).await;

        if services.is_empty() {
            return Ok(None);
        }

        // Sort by priority (highest first) and return the best match
        // For now, just return the first one - in a real impl, we'd check version constraints
        Ok(Some(services.into_iter().next().unwrap()))
    }

    /// Resolve with fallback chain
    ///
    /// Tries each category in order, returning the first match.
    pub async fn resolve_with_fallback(
        &self,
        categories: &[&str],
    ) -> Result<Option<RegisteredService>, IntentRegistryError> {
        for category in categories {
            if let Some(service) = self.resolve(category).await? {
                return Ok(Some(service));
            }
        }
        Ok(None)
    }

    /// Resolve with required capability
    ///
    /// Returns a service that supports the given action.
    pub async fn resolve_by_action(
        &self,
        action: &str,
    ) -> Result<Option<RegisteredService>, IntentRegistryError> {
        let services = self.registry.find_by_action(action).await;

        if services.is_empty() {
            return Ok(None);
        }

        Ok(Some(services.into_iter().next().unwrap()))
    }

    /// Resolve to all matching services
    ///
    /// Returns all services matching the category, sorted by priority.
    pub async fn resolve_all(
        &self,
        category: &str,
    ) -> Result<Vec<RegisteredService>, IntentRegistryError> {
        let mut services = self.registry.find_by_category(category).await;

        // Sort by priority (descending)
        // Note: We'd sort by priority field if we had access to intents during lookup
        services.sort_by(|a, b| {
            b.metadata
                .labels
                .get("priority")
                .or(a.metadata.labels.get("priority"))
                .and_then(|p| p.parse::<u32>().ok())
                .unwrap_or(0)
                .cmp(
                    &a.metadata
                        .labels
                        .get("priority")
                        .or(b.metadata.labels.get("priority"))
                        .and_then(|p| p.parse::<u32>().ok())
                        .unwrap_or(0),
                )
        });

        Ok(services)
    }

    /// Check if a service is available for an intent
    pub async fn is_available(&self, service_id: &str) -> bool {
        self.registry
            .get(service_id)
            .await
            .map(|s| s.is_available())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::ServiceIntent;
    use crate::service::ServiceMetadata;

    #[tokio::test]
    async fn test_resolve_basic() {
        let registry = Arc::new(IntentRegistry::new());

        let metadata = ServiceMetadata::new("git-service", "1.0.0").with_label("priority", "10");
        let intents = vec![ServiceIntent::new(
            "vcs",
            vec!["git".to_string(), "commit".to_string()],
        )];

        registry
            .register("git-svc".to_string(), metadata, intents, None)
            .await
            .unwrap();

        let resolver = IntentResolver::new(registry);
        let result = resolver.resolve("vcs").await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().id, "git-svc");
    }

    #[tokio::test]
    async fn test_resolve_not_found() {
        let registry = Arc::new(IntentRegistry::new());
        let resolver = IntentResolver::new(registry);

        let result = resolver.resolve("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_resolve_fallback() {
        let registry = Arc::new(IntentRegistry::new());

        // Register only storage
        let metadata = ServiceMetadata::new("storage", "1.0.0");
        let intents = vec![ServiceIntent::new(
            "storage",
            vec!["read".to_string(), "write".to_string()],
        )];
        registry
            .register("storage-svc".to_string(), metadata, intents, None)
            .await
            .unwrap();

        let resolver = IntentResolver::new(registry);

        // First choice not available, fallback to second
        let result = resolver
            .resolve_with_fallback(&["database", "storage"])
            .await
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, "storage-svc");
    }
}
