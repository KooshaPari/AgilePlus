//! Atomic terminal acceptance transaction.

use agileplus_domain::{
    domain::{
        acceptance::{
            AcceptFeatureCommand, AcceptanceOutcome, AcceptedCandidate, FeatureAcceptanceReceipt,
            validate_candidate,
        },
        audit::{AuditEntry, hash_entry},
        event::Event,
        governance_evaluator::{GovernanceEvaluationOptions, evaluate_governance_snapshot},
        state_machine::FeatureState,
        work_package::WpState,
    },
    error::DomainError,
};
use agileplus_events::compute_hash;
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};

use super::{audit, events, evidence, execution, features, governance, metrics, work_packages};

fn err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn load_replay(
    c: &Connection,
    command: &AcceptFeatureCommand,
) -> Result<Option<AcceptanceOutcome>, DomainError> {
    let row: Option<(String, String)> = c
        .query_row(
            "SELECT command_json,receipt_json FROM feature_acceptance_receipts WHERE request_id=?1",
            [&command.request_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(err)?;
    let Some((command_json, receipt_json)) = row else {
        return Ok(None);
    };
    let recorded: AcceptFeatureCommand =
        serde_json::from_str(&command_json).map_err(|e| DomainError::Storage(e.to_string()))?;
    if recorded != *command {
        return Err(DomainError::Conflict(format!(
            "acceptance request_id {} already exists with different immutable command",
            command.request_id
        )));
    }
    let receipt: FeatureAcceptanceReceipt =
        serde_json::from_str(&receipt_json).map_err(|e| DomainError::Storage(e.to_string()))?;
    Ok(Some(AcceptanceOutcome {
        receipt,
        replayed: true,
    }))
}

pub fn accept_feature_atomic(
    c: &mut Connection,
    command: &AcceptFeatureCommand,
) -> Result<AcceptanceOutcome, DomainError> {
    command.validate()?;
    let tx = c.transaction().map_err(err)?;

    if let Some(replay) = load_replay(&tx, command)? {
        tx.commit().map_err(err)?;
        return Ok(replay);
    }

    let feature = features::get_feature_by_id(&tx, command.feature_id)?
        .ok_or_else(|| DomainError::NotFound(format!("feature {}", command.feature_id)))?;
    if feature.state != FeatureState::Implementing {
        return Err(DomainError::Validation(format!(
            "feature '{}' must be Implementing for terminal acceptance",
            feature.slug
        )));
    }

    let contract = governance::get_latest_governance_contract(&tx, feature.id)?
        .ok_or_else(|| DomainError::Validation("governance contract is not configured".into()))?;
    if command
        .expected_governance_version
        .is_some_and(|version| version != contract.version)
    {
        return Err(DomainError::Conflict(format!(
            "governance version changed: expected {:?}, current {}",
            command.expected_governance_version, contract.version
        )));
    }

    let work_packages = work_packages::list_wps_by_feature(&tx, feature.id)?;
    if work_packages.is_empty() {
        return Err(DomainError::Validation(
            "terminal acceptance cannot be vacuous: feature has no work packages".into(),
        ));
    }

    let mut governance_evidence = Vec::new();
    for wp in &work_packages {
        governance_evidence.extend(evidence::get_evidence_by_wp(&tx, wp.id)?);
    }
    let policies = governance::list_active_policies(&tx)?;
    let governance_metrics = metrics::get_metrics_by_feature(&tx, feature.id)?;
    let governance_result = evaluate_governance_snapshot(
        &contract,
        &governance_evidence,
        &policies,
        &governance_metrics,
        GovernanceEvaluationOptions::default(),
    );
    if !governance_result.passed(&contract) {
        return Err(DomainError::Validation(
            "current governance evaluation does not authorize terminal acceptance".into(),
        ));
    }

    let mut accepted_candidates = Vec::<AcceptedCandidate>::with_capacity(work_packages.len());
    for wp in &work_packages {
        let assignment = execution::get_active_assignment(&tx, wp.id)?
            .ok_or_else(|| DomainError::Validation(format!("WP{:02}: no active Assignment", wp.sequence)))?;
        let evaluations = execution::list_evaluations(&tx, &assignment.id)?;
        let evaluation = evaluations.last().ok_or_else(|| {
            DomainError::Validation(format!("WP{:02}: no Evaluation", wp.sequence))
        })?;
        let criteria = execution::list_assignment_criteria(&tx, &assignment.id)?;
        let results = execution::list_criterion_results(&tx, &evaluation.id)?;
        let attempts = execution::list_attempts(&tx, &assignment.id)?;
        accepted_candidates.push(validate_candidate(
            wp,
            &assignment,
            evaluation,
            &criteria,
            &results,
            &attempts,
        )?);
    }

    let mut audit_ids = Vec::new();
    let mut prev_hash = audit::get_latest_audit_entry(&tx, feature.id)?
        .map(|entry| entry.hash)
        .unwrap_or([0u8; 32]);
    for wp in &work_packages {
        if wp.state == WpState::Review {
            work_packages::update_wp_state(&tx, wp.id, WpState::Done)?;
            let mut entry = AuditEntry {
                id: 0,
                feature_id: feature.id,
                wp_id: Some(wp.id),
                timestamp: Utc::now(),
                actor: command.actor.clone(),
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
            entry.hash = hash_entry(&entry);
            prev_hash = entry.hash;
            audit_ids.push(audit::append_audit_entry(&tx, &entry)?);
        }
    }

    features::update_feature_state(&tx, feature.id, FeatureState::Validated)?;
    let mut feature_audit = AuditEntry {
        id: 0,
        feature_id: feature.id,
        wp_id: None,
        timestamp: Utc::now(),
        actor: command.actor.clone(),
        transition: "Implementing -> Validated".into(),
        evidence_refs: vec![],
        prev_hash,
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    feature_audit.hash = hash_entry(&feature_audit);
    audit_ids.push(audit::append_audit_entry(&tx, &feature_audit)?);

    let previous_events = events::get_events(&tx, "feature", feature.id)?;
    let previous_hash = previous_events
        .last()
        .map(|event| event.hash)
        .unwrap_or([0u8; 32]);
    let sequence = previous_events.last().map(|event| event.sequence).unwrap_or(0) + 1;
    let payload = serde_json::json!({"from":"Implementing","to":"Validated"});
    let mut event = Event::new(
        "feature",
        feature.id,
        "state_transitioned",
        payload,
        &command.actor,
    );
    event.sequence = sequence;
    event.prev_hash = previous_hash;
    event.hash = compute_hash(
        event.entity_id,
        &event.entity_type,
        &event.event_type,
        &event.payload,
        event.timestamp,
        &event.actor,
        &event.prev_hash,
    )
    .map_err(|e| DomainError::Storage(e.to_string()))?;
    let event_id = events::append_event(&tx, &event)?;

    let committed_at = Utc::now();
    let receipt = FeatureAcceptanceReceipt {
        request_id: command.request_id.clone(),
        feature_id: feature.id,
        actor: command.actor.clone(),
        governance_contract_id: contract.id,
        governance_version: contract.version,
        accepted_candidates,
        audit_ids,
        event_id,
        committed_at,
    };
    let command_json =
        serde_json::to_string(command).map_err(|e| DomainError::Storage(e.to_string()))?;
    let receipt_json =
        serde_json::to_string(&receipt).map_err(|e| DomainError::Storage(e.to_string()))?;
    tx.execute(
        "INSERT INTO feature_acceptance_receipts
         (request_id,feature_id,command_json,receipt_json,committed_at)
         VALUES (?1,?2,?3,?4,?5)",
        params![
            command.request_id,
            feature.id,
            command_json,
            receipt_json,
            committed_at.to_rfc3339()
        ],
    )
    .map_err(err)?;

    tx.commit().map_err(err)?;
    Ok(AcceptanceOutcome {
        receipt,
        replayed: false,
    })
}
