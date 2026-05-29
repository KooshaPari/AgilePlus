//! Story repository port.

use async_trait::async_trait;

use crate::domain::story::{Story, StoryStatus};
use crate::error::DomainError;

/// Repository port for Story aggregates.
#[async_trait]
pub trait StoryRepository: Send + Sync {
    async fn create(&self, story: &Story) -> Result<i64, DomainError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Story>, DomainError>;
    async fn update_status(&self, id: i64, status: StoryStatus) -> Result<(), DomainError>;
    async fn list_by_epic(&self, epic_id: i64) -> Result<Vec<Story>, DomainError>;
}
