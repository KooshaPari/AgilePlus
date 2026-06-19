//! In-memory spec repository adapter

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use async_trait::async_trait;

use crate::domain::entity::Entity;
use crate::domain::{EntityId, Spec};
use crate::ports::output::{RepositoryError, RepositoryResult, SpecRepository};

/// In-memory spec repository implementation
#[derive(Debug, Default)]
pub struct InMemorySpecRepository {
    specs: Arc<RwLock<HashMap<String, Spec>>>,
}

impl InMemorySpecRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SpecRepository for InMemorySpecRepository {
    async fn save(&self, spec: Spec) -> RepositoryResult<Spec> {
        let id = spec.id().as_str().to_string();
        let mut specs = self.specs.write().await;
        specs.insert(id, spec.clone());
        Ok(spec)
    }

    async fn find_by_id(&self, id: &EntityId) -> RepositoryResult<Spec> {
        let specs = self.specs.read().await;
        specs
            .get(id.as_str())
            .cloned()
            .ok_or_else(|| RepositoryError::NotFound(id.as_str().to_string()))
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Spec>> {
        let specs = self.specs.read().await;
        Ok(specs.values().cloned().collect())
    }

    async fn delete(&self, id: &EntityId) -> RepositoryResult<()> {
        let mut specs = self.specs.write().await;
        specs
            .remove(id.as_str())
            .ok_or_else(|| RepositoryError::NotFound(id.as_str().to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl crate::ports::OutputPort for InMemorySpecRepository {
    fn name(&self) -> &'static str {
        "InMemorySpecRepository"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_find() {
        let repo = InMemorySpecRepository::new();
        let spec = Spec::new("Test Spec", "Description");
        let id = spec.id().clone();

        repo.save(spec).await.unwrap();
        let found = repo.find_by_id(&id).await.unwrap();
        assert_eq!(found.title(), "Test Spec");
    }

    #[tokio::test]
    async fn test_not_found() {
        let repo = InMemorySpecRepository::new();
        let id = EntityId::new();
        let result = repo.find_by_id(&id).await;
        assert!(result.is_err());
    }
}
