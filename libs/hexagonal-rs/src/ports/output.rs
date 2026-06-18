//! Output port traits (repository interfaces)

use async_trait::async_trait;
use thiserror::Error;

use super::super::domain::{EntityId, Spec, Task};

/// Repository errors
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("duplicate entity: {0}")]
    Duplicate(String),

    #[error("repository error: {0}")]
    Other(String),
}

/// Result type for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Output port trait - called by domain/use cases
#[async_trait]
pub trait OutputPort: Send + Sync {
    /// Get the port name for logging
    fn name(&self) -> &'static str;
}

/// Spec repository trait
#[async_trait]
pub trait SpecRepository: OutputPort {
    async fn save(&self, spec: Spec) -> RepositoryResult<Spec>;
    async fn find_by_id(&self, id: &EntityId) -> RepositoryResult<Spec>;
    async fn find_all(&self) -> RepositoryResult<Vec<Spec>>;
    async fn delete(&self, id: &EntityId) -> RepositoryResult<()>;
}

/// Task repository trait
#[async_trait]
pub trait TaskRepository: OutputPort {
    async fn save(&self, task: Task) -> RepositoryResult<Task>;
    async fn find_by_id(&self, id: &EntityId) -> RepositoryResult<Task>;
    async fn find_by_work_package(&self, work_package_id: &EntityId)
    -> RepositoryResult<Vec<Task>>;
    async fn find_all(&self) -> RepositoryResult<Vec<Task>>;
    async fn delete(&self, id: &EntityId) -> RepositoryResult<()>;
}
