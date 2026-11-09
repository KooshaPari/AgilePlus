//! Atomic PM persistence boundary; mutations include audit append.
use crate::{
    domain::work_item::{AuditEvent, NewWorkItem, WorkItem, WorkItemState},
    error::DomainError,
};
#[async_trait::async_trait]
pub trait WorkItemPort: Send + Sync {
    async fn create_work_item(
        &self,
        input: NewWorkItem,
        actor: &str,
    ) -> Result<WorkItem, DomainError>;
    async fn transition_work_item(
        &self,
        id: i64,
        target: WorkItemState,
        actor: &str,
    ) -> Result<WorkItem, DomainError>;
    async fn list_work_item_audit(&self, id: i64) -> Result<Vec<AuditEvent>, DomainError>;
}
