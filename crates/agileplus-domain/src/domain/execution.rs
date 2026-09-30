//! Immutable execution records for MACE-style agent work.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecRevision {
    pub id: String,
    pub feature_id: i64,
    pub content_hash: String,
    pub parent_revision_id: Option<String>,
    pub accepted_at: DateTime<Utc>,
    pub authority: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentStatus { Active, Superseded, Cancelled }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assignment {
    pub id: String,
    pub wp_id: i64,
    pub spec_revision_id: String,
    pub created_at: DateTime<Utc>,
    pub supersedes_assignment_id: Option<String>,
    pub status: AssignmentStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStatus { Pending, Running, Failed, Cancelled, Completed, Expired }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attempt {
    pub id: String,
    pub assignment_id: String,
    pub worker_id: String,
    pub backend: String,
    pub job_id: Option<String>,
    pub worktree_path: Option<String>,
    pub base_candidate_ref: Option<String>,
    pub result_candidate_ref: Option<String>,
    pub status: AttemptStatus,
    pub failure_class: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationResult {
    Satisfied,
    Unsatisfied,
    Inconclusive,
    Unknown,
    NotConfigured,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub id: String,
    pub assignment_id: String,
    pub attempt_id: Option<String>,
    pub candidate_ref: String,
    pub evaluator_id: String,
    pub evaluator_version: String,
    pub result: EvaluationResult,
    pub evidence_refs: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn evaluation_result_is_not_binary() {
        assert_ne!(EvaluationResult::NotConfigured, EvaluationResult::Satisfied);
        assert_ne!(EvaluationResult::Unknown, EvaluationResult::Satisfied);
    }
}
