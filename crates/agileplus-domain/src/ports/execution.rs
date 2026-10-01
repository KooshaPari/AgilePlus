//! Focused persistence port for immutable execution records.

use crate::{
    domain::execution::{Assignment, Attempt, Evaluation, SpecRevision},
    error::DomainError,
};
use async_trait::async_trait;

#[async_trait]
pub trait ExecutionRecordPort: Send + Sync {
    async fn create_spec_revision(&self, revision: &SpecRevision) -> Result<(), DomainError>;
    async fn create_assignment(&self, assignment: &Assignment) -> Result<(), DomainError>;
    async fn create_attempt(&self, attempt: &Attempt) -> Result<(), DomainError>;
    async fn update_attempt_runtime(
        &self,
        attempt_id: &str,
        status: crate::domain::execution::AttemptStatus,
        job_id: Option<&str>,
        result_candidate_ref: Option<&str>,
        failure_class: Option<&str>,
        ended_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), DomainError>;
    async fn create_evaluation(&self, evaluation: &Evaluation) -> Result<(), DomainError>;
    async fn list_attempts(&self, assignment_id: &str) -> Result<Vec<Attempt>, DomainError>;
    async fn list_evaluations(&self, assignment_id: &str) -> Result<Vec<Evaluation>, DomainError>;
}
