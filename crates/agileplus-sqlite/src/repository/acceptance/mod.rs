//! SQLite terminal acceptance: one immediate transaction owns reads and writes.
//! No await, network operation, or separately locked adapter call occurs in it.

use agileplus_domain::{
    domain::{
        acceptance::{AcceptFeatureCommand, AcceptanceOutcome, FeatureAcceptanceReceipt, validate_candidate},
        audit::{AuditChain, AuditEntry, hash_entry},
        event::Event,
        governance_evaluator::{GovernanceEvaluationOptions, evaluate_governance_snapshot},
        state_machine::FeatureState,
        work_package::WpState,
    },
    error::DomainError,
    ports::execution::AtomicAcceptancePort,
};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use super::{audit, events, evidence, execution, features, governance, metrics, work_packages};
use crate::SqliteStorageAdapter;

fn storage(error: impl std::fmt::Display) -> DomainError { DomainError::Storage(error.to_string()) }
fn invalid(message: impl Into<String>) -> DomainError { DomainError::Validation(message.into()) }

#[async_trait::async_trait]
impl AtomicAcceptancePort for SqliteStorageAdapter {
    async fn accept_feature_atomic(&self, command: &AcceptFeatureCommand) -> Result<AcceptanceOutcome, DomainError> {
        let mut connection = self.lock()?;
        commit_acceptance(&mut connection, command)
    }
}

pub fn commit_acceptance(
    connection: &mut Connection,
    command: &AcceptFeatureCommand,
) -> Result<AcceptanceOutcome, DomainError> {
    command.validate()?;
    // The writer reservation precedes all acceptance-relevant reads.
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
    let prior: Option<(String, String)> = tx.query_row(
        "SELECT command_json, receipt_json FROM feature_acceptance_receipts WHERE request_id=?1",
        [&command.request_id], |r| Ok((r.get(0)?, r.get(1)?)),
    ).optional().map_err(storage)?;
    if let Some((original, receipt_json)) = prior {
        let original: AcceptFeatureCommand = serde_json::from_str(&original).map_err(storage)?;
        if original != *command {
            return Err(DomainError::Conflict("acceptance request ID was used for a different command".into()));
        }
        let receipt = serde_json::from_str(&receipt_json).map_err(storage)?;
        tx.commit().map_err(storage)?;
        return Ok(AcceptanceOutcome { receipt, replayed: true });
    }

    let feature = features::get_feature_by_id(&tx, command.feature_id)?
        .ok_or_else(|| DomainError::NotFound(format!("feature {}", command.feature_id)))?;
    if feature.state != FeatureState::Implementing {
        return Err(invalid("terminal acceptance requires Implementing"));
    }
    let contract = governance::get_latest_governance_contract(&tx, feature.id)?
        .ok_or_else(|| invalid("no governance contract; acceptance is NotConfigured"))?;
    if command.expected_governance_version.is_some_and(|version| version != contract.version) {
        return Err(DomainError::Conflict("governance changed after preflight".into()));
    }
    let wps = work_packages::list_wps_by_feature(&tx, feature.id)?;
    if wps.is_empty() { return Err(invalid("no work packages; terminal acceptance cannot be vacuous")); }
    let mut evidence_rows = Vec::new();
    for wp in &wps { evidence_rows.extend(evidence::get_evidence_by_wp(&tx, wp.id)?); }
    let policy_rows = governance::list_active_policies(&tx)?;
    let metric_rows = metrics::get_metrics_by_feature(&tx, feature.id)?;
    let governance = evaluate_governance_snapshot(
        &contract, &evidence_rows, &policy_rows, &metric_rows, GovernanceEvaluationOptions::default(),
    );
    if !governance.passed(&contract) {
        return Err(invalid("current governance evaluation did not pass; acceptance denied"));
    }

    let mut accepted_candidates = Vec::with_capacity(wps.len());
    for wp in &wps {
        let assignment = execution::get_active_assignment(&tx, wp.id)?
            .ok_or_else(|| invalid(format!("WP{:02} has no active Assignment", wp.sequence)))?;
        let spec_feature: Option<i64> = tx.query_row(
            "SELECT feature_id FROM spec_revisions WHERE id=?1", [&assignment.spec_revision_id], |r| r.get(0),
        ).optional().map_err(storage)?;
        if spec_feature != Some(feature.id) { return Err(invalid("Assignment SpecRevision is outside feature scope")); }
        let evaluations = execution::list_evaluations(&tx, &assignment.id)?;
        let evaluation = evaluations.last().ok_or_else(|| invalid(format!("WP{:02} has no Evaluation", wp.sequence)))?;
        let criteria = execution::list_assignment_criteria(&tx, &assignment.id)?;
        let results = execution::list_criterion_results(&tx, &evaluation.id)?;
        let attempts = execution::list_attempts(&tx, &assignment.id)?;
        accepted_candidates.push(validate_candidate(wp, &assignment, evaluation, &criteria, &results, &attempts)?);
    }
    let history = audit::get_audit_trail(&tx, feature.id)?;
    if !history.is_empty() { AuditChain { entries: history }.verify_chain().map_err(invalid)?; }

    let now = Utc::now();
    let mut audit_ids = Vec::new();
    for wp in &wps {
        if wp.state == WpState::Review {
            let changed = tx.execute(
                "UPDATE work_packages SET state='done',updated_at=?1 WHERE id=?2 AND feature_id=?3 AND state='review'",
                params![now.to_rfc3339(), wp.id, feature.id],
            ).map_err(storage)?;
            if changed != 1 { return Err(DomainError::Conflict("work package changed during acceptance".into())); }
            audit_ids.push(append_audit(&tx, command, Some(wp.id), &format!(
                "WP{:02} Review -> Done (exact-candidate correctness + governance accepted)", wp.sequence
            ), now)?);
        }
    }
    let changed = tx.execute(
        "UPDATE features SET state='validated',updated_at=?1 WHERE id=?2 AND state='implementing'",
        params![now.to_rfc3339(), feature.id],
    ).map_err(storage)?;
    if changed != 1 { return Err(DomainError::Conflict("feature changed during acceptance".into())); }
    audit_ids.push(append_audit(&tx, command, None, "Implementing -> Validated", now)?);

    let previous_events = events::get_events(&tx, "feature", feature.id)?;
    let sequence = events::get_latest_sequence(&tx, "feature", feature.id)? + 1;
    let previous_hash = previous_events.last().map(|e| e.hash).unwrap_or([0; 32]);
    let payload = serde_json::json!({
        "from":"Implementing", "to":"Validated", "acceptance_request_id":command.request_id,
        "governance_contract_id":contract.id, "governance_version":contract.version,
        "accepted_candidates":accepted_candidates, "audit_ids":audit_ids,
    });
    let mut event = Event::new("feature", feature.id, "state_transitioned", payload, command.actor.as_str());
    event.timestamp = now;
    event.sequence = sequence;
    event.prev_hash = previous_hash;
    event.hash = agileplus_events::compute_hash(
        event.entity_id, &event.entity_type, &event.event_type, &event.payload,
        event.timestamp, &event.actor, &event.prev_hash,
    ).map_err(storage)?;
    let event_id = events::append_event(&tx, &event)?;
    let receipt = FeatureAcceptanceReceipt {
        request_id: command.request_id.clone(), feature_id: feature.id, actor: command.actor.clone(),
        governance_contract_id: contract.id, governance_version: contract.version,
        accepted_candidates, audit_ids, event_id, committed_at: now,
    };
    tx.execute(
        "INSERT INTO feature_acceptance_receipts (request_id,feature_id,command_json,receipt_json,committed_at) VALUES (?1,?2,?3,?4,?5)",
        params![command.request_id, feature.id, serde_json::to_string(command).map_err(storage)?,
            serde_json::to_string(&receipt).map_err(storage)?, now.to_rfc3339()],
    ).map_err(storage)?;
    tx.commit().map_err(storage)?;
    Ok(AcceptanceOutcome { receipt, replayed: false })
}

fn append_audit(
    connection: &Connection, command: &AcceptFeatureCommand, wp_id: Option<i64>,
    transition: &str, timestamp: chrono::DateTime<Utc>,
) -> Result<i64, DomainError> {
    let prev_hash = audit::get_latest_audit_entry(connection, command.feature_id)?
        .map(|entry| entry.hash).unwrap_or([0; 32]);
    let mut entry = AuditEntry {
        id: 0, feature_id: command.feature_id, wp_id, timestamp,
        actor: command.actor.clone(), transition: transition.into(), evidence_refs: Vec::new(),
        prev_hash, hash: [0; 32], event_id: None, archived_to: None,
    };
    entry.hash = hash_entry(&entry);
    audit::append_audit_entry(connection, &entry)
}
