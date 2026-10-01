//! Focused persistence port for immutable execution records.

use crate::{
    domain::execution::{
        Assignment, AssignmentCriterion, Attempt, CriterionEvaluation, Evaluation, SpecRevision,
    },
    error::DomainError,
};
use async_trait::async_trait;

#[async_trait]
pub trait ExecutionRecordPort: Send + Sync {
    async fn create_spec_revision(&self, revision: &SpecRevision) -> Result<(), DomainError>;
    async fn create_assignment(&self, assignment: &Assignment) -> Result<(), DomainError>;
    async fn create_assignment_with_criteria(
        &self,
        assignment: &Assignment,
        criteria: &[AssignmentCriterion],
    ) -> Result<(), DomainError>;
    async fn get_active_assignment(&self, wp_id: i64) -> Result<Option<Assignment>, DomainError>;
    async fn supersede_assignment(
        &self,
        previous_assignment_id: &str,
        replacement: &Assignment,
    ) -> Result<(), DomainError>;
    async fn supersede_assignment_with_criteria(
        &self,
        previous_assignment_id: &str,
        replacement: &Assignment,
        criteria: &[AssignmentCriterion],
    ) -> Result<(), DomainError>;
    async fn list_assignment_criteria(
        &self,
        assignment_id: &str,
    ) -> Result<Vec<AssignmentCriterion>, DomainError>;
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
    async fn create_evaluation_receipt(
        &self,
        evaluation: &Evaluation,
        criterion_results: &[CriterionEvaluation],
    ) -> Result<(), DomainError>;
    async fn list_attempts(&self, assignment_id: &str) -> Result<Vec<Attempt>, DomainError>;
    async fn list_evaluations(&self, assignment_id: &str) -> Result<Vec<Evaluation>, DomainError>;
    async fn list_criterion_results(
        &self,
        evaluation_id: &str,
    ) -> Result<Vec<CriterionEvaluation>, DomainError>;
}
