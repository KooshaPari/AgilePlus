// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared exact-candidate acceptance oracle.
//!
//! This module is intentionally read-only. Transports and mutation use cases
//! consume the same receipt rather than reimplementing MACE acceptance.

use agileplus_domain::{
    domain::{
        execution::{
            aggregate_evidence_refs, reduce_criterion_results, validate_criterion_receipt,
            AttemptStatus, EvaluationResult,
        },
        work_package::{WorkPackage, WpState},
    },
    ports::{ExecutionRecordPort, StoragePort},
};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedCandidate {
    pub wp_id: i64,
    pub assignment_id: String,
    pub attempt_id: String,
    pub evaluation_id: String,
    pub candidate_ref: String,
    pub worktree_path: Option<String>,
}

fn validation(message: impl Into<String>) -> AppError {
    agileplus_domain::error::DomainError::Validation(message.into()).into()
}

/// Require an independently satisfied, criterion-complete evaluation bound to
/// the exact completed attempt candidate for one work package.
pub async fn accepted_candidate_for_wp<S>(
    storage: &S,
    wp: &WorkPackage,
) -> Result<AcceptedCandidate, AppError>
where
    S: StoragePort + ExecutionRecordPort,
{
    if !matches!(wp.state, WpState::Review | WpState::Done) {
        return Err(validation(format!(
            "WP{:02} '{}' is in state {:?}; exact-candidate acceptance requires Review or Done",
            wp.sequence, wp.title, wp.state
        )));
    }

    let assignment = storage
        .get_active_assignment(wp.id)
        .await?
        .ok_or_else(|| validation(format!("WP{:02} has no active Assignment", wp.sequence)))?;

    let evaluations = storage.list_evaluations(&assignment.id).await?;
    let evaluation = evaluations
        .last()
        .ok_or_else(|| validation(format!("WP{:02} has no Evaluation", wp.sequence)))?;

    if evaluation.result != EvaluationResult::Satisfied {
        return Err(validation(format!(
            "WP{:02} latest Evaluation {} is {:?}, not Satisfied",
            wp.sequence, evaluation.id, evaluation.result
        )));
    }

    let criteria = storage.list_assignment_criteria(&assignment.id).await?;
    if criteria.is_empty() {
        return Err(validation(format!(
            "WP{:02} Satisfied Evaluation {} has no frozen Assignment criteria; legacy naked grades are not authoritative",
            wp.sequence, evaluation.id
        )));
    }

    let criterion_results = storage.list_criterion_results(&evaluation.id).await?;
    validate_criterion_receipt(&criteria, &criterion_results)
        .map_err(validation)?;
    if reduce_criterion_results(&criteria, &criterion_results) != EvaluationResult::Satisfied {
        return Err(validation(format!(
            "WP{:02} Evaluation {} aggregate is inconsistent with its criterion receipt",
            wp.sequence, evaluation.id
        )));
    }
    if aggregate_evidence_refs(&criterion_results) != evaluation.evidence_refs {
        return Err(validation(format!(
            "WP{:02} Evaluation {} evidence union does not match its criterion receipt",
            wp.sequence, evaluation.id
        )));
    }

    if !evaluation.candidate_ref.starts_with("git:") {
        return Err(validation(format!(
            "WP{:02} Satisfied Evaluation {} is not bound to an exact Git candidate: {}",
            wp.sequence, evaluation.id, evaluation.candidate_ref
        )));
    }

    let attempt_id = evaluation.attempt_id.as_deref().ok_or_else(|| {
        validation(format!(
            "WP{:02} Satisfied Evaluation {} is not bound to an Attempt",
            wp.sequence, evaluation.id
        ))
    })?;
    let attempts = storage.list_attempts(&assignment.id).await?;
    let attempt = attempts
        .iter()
        .find(|attempt| attempt.id == attempt_id)
        .ok_or_else(|| {
            validation(format!(
                "WP{:02} Evaluation {} references missing Attempt {}",
                wp.sequence, evaluation.id, attempt_id
            ))
        })?;

    if attempt.status != AttemptStatus::Completed {
        return Err(validation(format!(
            "WP{:02} evaluated Attempt {} is {:?}, not Completed",
            wp.sequence, attempt.id, attempt.status
        )));
    }
    if attempt.result_candidate_ref.as_deref() != Some(evaluation.candidate_ref.as_str()) {
        return Err(validation(format!(
            "WP{:02} candidate mismatch: Attempt {:?}, Evaluation {}",
            wp.sequence, attempt.result_candidate_ref, evaluation.candidate_ref
        )));
    }

    Ok(AcceptedCandidate {
        wp_id: wp.id,
        assignment_id: assignment.id,
        attempt_id: attempt.id.clone(),
        evaluation_id: evaluation.id.clone(),
        candidate_ref: evaluation.candidate_ref.clone(),
        worktree_path: attempt.worktree_path.clone(),
    })
}

/// Preflight every work package before a caller performs any terminal mutation.
pub async fn require_feature_acceptance<S>(
    storage: &S,
    feature_id: i64,
) -> Result<Vec<AcceptedCandidate>, AppError>
where
    S: StoragePort + ExecutionRecordPort,
{
    let work_packages = storage.list_wps_by_feature(feature_id).await?;
    if work_packages.is_empty() {
        return Err(validation(format!(
            "Feature {feature_id} has no work packages; terminal acceptance cannot be vacuous"
        )));
    }

    let mut accepted = Vec::with_capacity(work_packages.len());
    for wp in &work_packages {
        accepted.push(accepted_candidate_for_wp(storage, wp).await?);
    }
    Ok(accepted)
}
