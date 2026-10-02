// SPDX-License-Identifier: MIT OR Apache-2.0
//! Canonical terminal acceptance mutation.
//!
//! The use case performs all read-only preflight before the first state write.
//! It deliberately does not own governance evaluation: callers must present a
//! governance receipt produced by the shared evaluator.

use chrono::Utc;

use agileplus_domain::{
    domain::{
        audit::{AuditEntry, hash_entry},
        state_machine::FeatureState,
        work_package::WpState,
    },
    ports::{ExecutionRecordPort, StoragePort},
};

use crate::{
    error::AppError,
    use_cases::acceptance::{AcceptedCandidate, require_feature_acceptance},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceReceipt {
    pub passed: bool,
    pub authoritative: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureAcceptanceReceipt {
    pub feature_id: i64,
    pub accepted_candidates: Vec<AcceptedCandidate>,
}

fn validation(message: impl Into<String>) -> AppError {
    agileplus_domain::error::DomainError::Validation(message.into()).into()
}

async fn latest_hash<S: StoragePort>(storage: &S, feature_id: i64) -> Result<[u8; 32], AppError> {
    Ok(storage
        .get_latest_audit_entry(feature_id)
        .await?
        .map(|entry| entry.hash)
        .unwrap_or([0u8; 32]))
}

/// Award terminal work acceptance after independent correctness and governance
/// have both passed. All work packages are preflighted before mutation.
pub async fn accept_feature<S>(
    storage: &S,
    feature_id: i64,
    governance: GovernanceReceipt,
    actor: &str,
) -> Result<FeatureAcceptanceReceipt, AppError>
where
    S: StoragePort + ExecutionRecordPort,
{
    if !governance.passed {
        return Err(validation("governance evaluation did not pass"));
    }
    if !governance.authoritative {
        return Err(validation(
            "diagnostic/exception governance evaluation cannot authorize terminal acceptance",
        ));
    }

    let feature = storage
        .get_feature_by_id(feature_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("feature {feature_id}")))?;
    if feature.state != FeatureState::Implementing {
        return Err(validation(format!(
            "feature '{}' is {:?}; terminal acceptance requires Implementing",
            feature.slug, feature.state
        )));
    }

    // Critical atomicity boundary: every correctness receipt is checked before
    // any WP or Feature state is changed.
    let accepted_candidates = require_feature_acceptance(storage, feature_id).await?;

    for accepted in &accepted_candidates {
        let wp = storage
            .get_work_package(accepted.wp_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("work package {}", accepted.wp_id)))?;

        if wp.state == WpState::Review {
            storage.update_wp_state(wp.id, WpState::Done).await?;

            let prev_hash = latest_hash(storage, feature_id).await?;
            let mut audit = AuditEntry {
                id: 0,
                feature_id,
                wp_id: Some(wp.id),
                timestamp: Utc::now(),
                actor: actor.to_string(),
                transition: format!(
                    "WP{:02} Review -> Done (exact-candidate correctness + governance accepted)",
                    wp.sequence
                ),
                evidence_refs: vec![],
                prev_hash,
                hash: [0u8; 32],
                event_id: None,
                archived_to: None,
            };
            audit.hash = hash_entry(&audit);
            storage.append_audit_entry(&audit).await?;
        }
    }

    storage
        .update_feature_state(feature_id, FeatureState::Validated)
        .await?;

    let prev_hash = latest_hash(storage, feature_id).await?;
    let mut audit = AuditEntry {
        id: 0,
        feature_id,
        wp_id: None,
        timestamp: Utc::now(),
        actor: actor.to_string(),
        transition: "Implementing -> Validated".into(),
        evidence_refs: vec![],
        prev_hash,
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    audit.hash = hash_entry(&audit);
    storage.append_audit_entry(&audit).await?;

    Ok(FeatureAcceptanceReceipt {
        feature_id,
        accepted_candidates,
    })
}
