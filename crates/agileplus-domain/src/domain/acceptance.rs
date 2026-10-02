//! Exact-candidate correctness and the durable terminal-acceptance contract.
//! A receipt reports a committed historical decision, not perpetual applicability.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{
    execution::{
        Assignment, AssignmentCriterion, AssignmentStatus, Attempt, AttemptStatus,
        CriterionEvaluation, Evaluation, EvaluationResult, aggregate_evidence_refs,
        reduce_criterion_results, validate_criterion_receipt,
    },
    work_package::{WorkPackage, WpState},
};
use crate::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedCandidate {
    pub wp_id: i64,
    pub assignment_id: String,
    pub attempt_id: String,
    pub evaluation_id: String,
    pub candidate_ref: String,
    pub worktree_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptFeatureCommand {
    pub request_id: String,
    pub feature_id: i64,
    /// Supplied by the authenticated transport, not a policy override.
    pub actor: String,
    /// Optional optimistic precondition; current governance is always evaluated.
    pub expected_governance_version: Option<i32>,
}

impl AcceptFeatureCommand {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.feature_id <= 0
            || self.request_id.trim().is_empty()
            || self.request_id.len() > 128
            || self.actor.trim().is_empty()
            || self.actor.len() > 256
            || self.request_id.chars().any(char::is_control)
            || self.actor.chars().any(char::is_control)
        {
            return Err(DomainError::Validation(
                "invalid acceptance command identity".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureAcceptanceReceipt {
    pub request_id: String,
    pub feature_id: i64,
    pub actor: String,
    pub governance_contract_id: i64,
    pub governance_version: i32,
    pub accepted_candidates: Vec<AcceptedCandidate>,
    pub audit_ids: Vec<i64>,
    pub event_id: i64,
    pub committed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceOutcome {
    pub receipt: FeatureAcceptanceReceipt,
    /// Replay returns the original receipt without asserting current validity.
    pub replayed: bool,
}

/// The same pure correctness oracle is used by shipping and atomic acceptance.
/// This checks recorded candidate identity; shipping separately resolves Git refs.
pub fn validate_candidate(
    wp: &WorkPackage,
    assignment: &Assignment,
    evaluation: &Evaluation,
    criteria: &[AssignmentCriterion],
    results: &[CriterionEvaluation],
    attempts: &[Attempt],
) -> Result<AcceptedCandidate, DomainError> {
    let invalid =
        |reason: String| DomainError::Validation(format!("WP{:02}: {reason}", wp.sequence));
    if !matches!(wp.state, WpState::Review | WpState::Done) {
        return Err(invalid(
            "exact-candidate acceptance requires Review or Done".into(),
        ));
    }
    if assignment.wp_id != wp.id || assignment.status != AssignmentStatus::Active {
        return Err(invalid(
            "Assignment is not active for this work package".into(),
        ));
    }
    if evaluation.assignment_id != assignment.id {
        return Err(invalid("Evaluation belongs to another Assignment".into()));
    }
    if evaluation.result != EvaluationResult::Satisfied {
        return Err(invalid(format!(
            "latest Evaluation {} is {:?}, not Satisfied",
            evaluation.id, evaluation.result
        )));
    }
    if criteria.is_empty() {
        return Err(invalid(
            "no frozen Assignment criteria; legacy naked grades are not authoritative".into(),
        ));
    }
    validate_criterion_receipt(criteria, results).map_err(invalid)?;
    if reduce_criterion_results(criteria, results) != EvaluationResult::Satisfied {
        return Err(invalid(
            "Evaluation aggregate is inconsistent with its criterion receipt".into(),
        ));
    }
    if aggregate_evidence_refs(results) != evaluation.evidence_refs {
        return Err(invalid(
            "Evaluation evidence union does not match its criterion receipt".into(),
        ));
    }
    if evaluation
        .candidate_ref
        .strip_prefix("git:")
        .is_none_or(str::is_empty)
    {
        return Err(invalid(
            "Evaluation is not bound to an exact Git candidate".into(),
        ));
    }
    let attempt_id = evaluation
        .attempt_id
        .as_deref()
        .ok_or_else(|| invalid("Evaluation is not bound to an Attempt".into()))?;
    let attempt = attempts
        .iter()
        .find(|a| a.id == attempt_id)
        .ok_or_else(|| invalid("Evaluation references a missing Attempt".into()))?;
    if attempt.assignment_id != assignment.id || attempt.status != AttemptStatus::Completed {
        return Err(invalid(
            "evaluated Attempt is not Completed for this Assignment".into(),
        ));
    }
    if attempt.result_candidate_ref.as_deref() != Some(evaluation.candidate_ref.as_str()) {
        return Err(invalid(
            "candidate mismatch between Attempt and Evaluation".into(),
        ));
    }
    if evaluation.evaluator_id.trim().is_empty()
        || evaluation.evaluator_version.trim().is_empty()
        || evaluation.evaluator_id == attempt.worker_id
    {
        return Err(invalid(
            "Evaluation requires an identified independent evaluator".into(),
        ));
    }
    if evaluation.finished_at < evaluation.started_at {
        return Err(invalid("Evaluation timestamps are reversed".into()));
    }
    Ok(AcceptedCandidate {
        wp_id: wp.id,
        assignment_id: assignment.id.clone(),
        attempt_id: attempt.id.clone(),
        evaluation_id: evaluation.id.clone(),
        candidate_ref: evaluation.candidate_ref.clone(),
        worktree_path: attempt.worktree_path.clone(),
    })
}
