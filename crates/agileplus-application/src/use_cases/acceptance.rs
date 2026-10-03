// SPDX-License-Identifier: MIT OR Apache-2.0
//! Read-only preflight shares its pure correctness oracle with atomic acceptance.

use crate::error::AppError;
pub use agileplus_domain::domain::acceptance::AcceptedCandidate;
use agileplus_domain::{
    domain::{acceptance::validate_candidate, work_package::WorkPackage},
    error::DomainError,
    ports::{ExecutionRecordPort, StoragePort},
};

pub async fn accepted_candidate_for_wp<S>(
    storage: &S,
    wp: &WorkPackage,
) -> Result<AcceptedCandidate, AppError>
where
    S: StoragePort + ExecutionRecordPort,
{
    let assignment = storage.get_active_assignment(wp.id).await?.ok_or_else(|| {
        DomainError::Validation(format!("WP{:02} has no active Assignment", wp.sequence))
    })?;
    let evaluations = storage.list_evaluations(&assignment.id).await?;
    let evaluation = evaluations.last().ok_or_else(|| {
        DomainError::Validation(format!("WP{:02} has no Evaluation", wp.sequence))
    })?;
    let criteria = storage.list_assignment_criteria(&assignment.id).await?;
    let results = storage.list_criterion_results(&evaluation.id).await?;
    let attempts = storage.list_attempts(&assignment.id).await?;
    Ok(validate_candidate(
        wp,
        &assignment,
        evaluation,
        &criteria,
        &results,
        &attempts,
    )?)
}

/// Read-only preflight is not a token authorizing subsequent writes.
/// The transactional capability rechecks the live records at commit time.
pub async fn require_feature_acceptance<S>(
    storage: &S,
    feature_id: i64,
) -> Result<Vec<AcceptedCandidate>, AppError>
where
    S: StoragePort + ExecutionRecordPort,
{
    let work_packages = storage.list_wps_by_feature(feature_id).await?;
    if work_packages.is_empty() {
        return Err(DomainError::Validation(format!(
            "Feature {feature_id} has no work packages; terminal acceptance cannot be vacuous"
        ))
        .into());
    }
    let mut accepted = Vec::with_capacity(work_packages.len());
    for wp in &work_packages {
        accepted.push(accepted_candidate_for_wp(storage, wp).await?);
    }
    Ok(accepted)
}
