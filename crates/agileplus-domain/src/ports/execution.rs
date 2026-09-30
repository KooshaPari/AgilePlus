//! Focused persistence port for immutable execution records.

use async_trait::async_trait;
use crate::{domain::execution::{Assignment, Attempt, Evaluation, SpecRevision}, error::DomainError};

#[async_trait]
pub trait ExecutionRecordPort: Send + Sync {
    async fn create_spec_revision(&self, revision:&SpecRevision)->Result<(),DomainError>;
    async fn create_assignment(&self, assignment:&Assignment)->Result<(),DomainError>;
    async fn create_attempt(&self, attempt:&Attempt)->Result<(),DomainError>;
    async fn create_evaluation(&self, evaluation:&Evaluation)->Result<(),DomainError>;
    async fn list_attempts(&self, assignment_id:&str)->Result<Vec<Attempt>,DomainError>;
    async fn list_evaluations(&self, assignment_id:&str)->Result<Vec<Evaluation>,DomainError>;
}
