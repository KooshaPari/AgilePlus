//! Spec use cases

use async_trait::async_trait;
use std::sync::Arc;

use super::super::domain::{EntityId, Spec, SpecStatus};
use super::super::ports::{InputPort, SpecRepository};

/// Input for creating a spec
#[derive(Debug)]
pub struct CreateSpecInput {
    pub title: String,
    pub description: String,
}

/// Result of creating a spec
#[derive(Debug)]
pub struct CreateSpecResult {
    pub spec: Spec,
}

/// Use case for creating a new spec
pub struct CreateSpecUseCase<R: SpecRepository> {
    repository: Arc<R>,
}

impl<R: SpecRepository> CreateSpecUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: SpecRepository> InputPort<CreateSpecInput, CreateSpecResult> for CreateSpecUseCase<R> {
    async fn execute(&self, input: CreateSpecInput) -> CreateSpecResult {
        let spec = Spec::new(input.title, input.description);
        let saved = self
            .repository
            .save(spec)
            .await
            .expect("Failed to save spec");
        CreateSpecResult { spec: saved }
    }
}

/// Input for updating a spec
#[derive(Debug)]
pub struct UpdateSpecInput {
    pub id: EntityId,
    pub title: Option<String>,
    pub status: Option<SpecStatus>,
}

/// Result of updating a spec
#[derive(Debug)]
pub struct UpdateSpecResult {
    pub spec: Spec,
}

/// Use case for updating a spec
pub struct UpdateSpecUseCase<R: SpecRepository> {
    repository: Arc<R>,
}

impl<R: SpecRepository> UpdateSpecUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: SpecRepository> InputPort<UpdateSpecInput, UpdateSpecResult> for UpdateSpecUseCase<R> {
    async fn execute(&self, input: UpdateSpecInput) -> UpdateSpecResult {
        let mut spec = self
            .repository
            .find_by_id(&input.id)
            .await
            .expect("Spec not found");

        if let Some(title) = input.title {
            spec.set_title(title);
        }
        if let Some(status) = input.status {
            spec.set_status(status);
        }

        let saved = self
            .repository
            .save(spec)
            .await
            .expect("Failed to save spec");
        UpdateSpecResult { spec: saved }
    }
}

/// Use case for listing all specs
pub struct ListSpecsUseCase<R: SpecRepository> {
    repository: Arc<R>,
}

impl<R: SpecRepository> ListSpecsUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: SpecRepository> InputPort<(), Vec<Spec>> for ListSpecsUseCase<R> {
    async fn execute(&self, _input: ()) -> Vec<Spec> {
        self.repository
            .find_all()
            .await
            .expect("Failed to list specs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::spec_repository::InMemorySpecRepository;
    use crate::domain::entity::Entity;
    use crate::ports::SpecRepository;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_spec_use_case() {
        let repo = Arc::new(InMemorySpecRepository::default());
        let use_case = CreateSpecUseCase::new(repo);
        let input = CreateSpecInput {
            title: "Test Spec".to_string(),
            description: "Test Description".to_string(),
        };

        let result = use_case.execute(input).await;
        assert_eq!(result.spec.title(), "Test Spec");
    }

    #[tokio::test]
    async fn test_update_spec_use_case() {
        let repo = Arc::new(InMemorySpecRepository::default());
        let spec = Spec::new("Original", "Desc");
        let spec_id = spec.id().clone();
        repo.save(spec).await.unwrap();

        let use_case = UpdateSpecUseCase::new(repo);
        let input = UpdateSpecInput {
            id: spec_id,
            title: Some("Updated".to_string()),
            status: Some(SpecStatus::Active),
        };

        let result = use_case.execute(input).await;
        assert_eq!(result.spec.title(), "Updated");
        assert_eq!(result.spec.status(), SpecStatus::Active);
    }

    #[tokio::test]
    async fn test_list_specs_use_case() {
        let repo = Arc::new(InMemorySpecRepository::default());
        repo.save(Spec::new("Spec 1", "Desc 1")).await.unwrap();
        repo.save(Spec::new("Spec 2", "Desc 2")).await.unwrap();

        let use_case = ListSpecsUseCase::new(repo);
        let result = use_case.execute(()).await;
        assert_eq!(result.len(), 2);
    }
}
