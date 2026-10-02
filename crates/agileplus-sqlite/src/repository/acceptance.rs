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


#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::{
        domain::{
            acceptance::AcceptFeatureCommand,
            execution::{
                Assignment, AssignmentStatus, Attempt, AttemptStatus, CriterionEvaluation,
                Evaluation, EvaluationResult, SpecRevision, aggregate_evidence_refs,
                reduce_criterion_results, snapshot_acceptance_criteria,
            },
            feature::Feature,
            governance::{
                Evidence, EvidenceType, GovernanceContract, GovernanceRule,
            },
            state_machine::FeatureState,
            work_package::{WorkPackage, WpState},
        },
        ports::{ExecutionRecordPort, StoragePort},
    };
    use crate::SqliteStorageAdapter;

    async fn fixture() -> (SqliteStorageAdapter, i64, i64) {
        let db = SqliteStorageAdapter::in_memory().expect("db");
        let mut feature = Feature::new("atomic", "Atomic", [1; 32], None);
        feature.state = FeatureState::Implementing;
        let feature_id = StoragePort::create_feature(&db, &feature).await.expect("feature");

        let mut wp = WorkPackage::new(feature_id, "WP", 1, "criterion");
        wp.state = WpState::Review;
        let wp_id = StoragePort::create_work_package(&db, &wp).await.expect("wp");

        StoragePort::create_governance_contract(
            &db,
            &GovernanceContract {
                id: 0,
                feature_id,
                version: 1,
                rules: vec![GovernanceRule {
                    transition: "Implementing->Validated".into(),
                    required_evidence: vec!["FR-1:test_result".into()],
                    policy_refs: vec![],
                }],
                bound_at: Utc::now(),
            },
        )
        .await
        .expect("contract");
        StoragePort::create_evidence(
            &db,
            &Evidence {
                id: 0,
                wp_id,
                fr_id: "FR-1".into(),
                evidence_type: EvidenceType::TestResult,
                artifact_path: "artifact:test".into(),
                metadata: None,
                created_at: Utc::now(),
            },
        )
        .await
        .expect("evidence");

        let spec = SpecRevision {
            id: "spec:1".into(),
            feature_id,
            content_hash: "sha256:1".into(),
            parent_revision_id: None,
            accepted_at: Utc::now(),
            authority: "test".into(),
        };
        ExecutionRecordPort::create_spec_revision(&db, &spec)
            .await
            .expect("spec");
        let assignment = Assignment {
            id: "assignment:1".into(),
            wp_id,
            spec_revision_id: spec.id,
            created_at: Utc::now(),
            supersedes_assignment_id: None,
            status: AssignmentStatus::Active,
        };
        let criteria = snapshot_acceptance_criteria("criterion");
        ExecutionRecordPort::create_assignment_with_criteria(&db, &assignment, &criteria)
            .await
            .expect("assignment");
        let attempt = Attempt {
            id: "attempt:1".into(),
            assignment_id: assignment.id.clone(),
            worker_id: "worker".into(),
            backend: "test".into(),
            job_id: None,
            worktree_path: Some("/tmp/atomic".into()),
            base_candidate_ref: Some("git:base".into()),
            result_candidate_ref: Some("git:candidate".into()),
            status: AttemptStatus::Completed,
            failure_class: None,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
        };
        ExecutionRecordPort::create_attempt(&db, &attempt)
            .await
            .expect("attempt");
        let criterion_results = vec![CriterionEvaluation {
            criterion_id: criteria[0].id.clone(),
            result: EvaluationResult::Satisfied,
            evidence_refs: vec!["evidence:criterion".into()],
            rationale: None,
        }];
        let evaluation = Evaluation {
            id: "evaluation:1".into(),
            assignment_id: assignment.id,
            attempt_id: Some(attempt.id),
            candidate_ref: "git:candidate".into(),
            evaluator_id: "independent-evaluator".into(),
            evaluator_version: "1".into(),
            result: reduce_criterion_results(&criteria, &criterion_results),
            evidence_refs: aggregate_evidence_refs(&criterion_results),
            started_at: Utc::now(),
            finished_at: Utc::now(),
        };
        ExecutionRecordPort::create_evaluation_receipt(&db, &evaluation, &criterion_results)
            .await
            .expect("evaluation");
        (db, feature_id, wp_id)
    }

    fn command(feature_id: i64) -> AcceptFeatureCommand {
        AcceptFeatureCommand {
            request_id: "request:1".into(),
            feature_id,
            actor: "tester".into(),
            expected_governance_version: Some(1),
        }
    }

    #[tokio::test]
    async fn acceptance_commits_state_audit_event_and_receipt_together() {
        let (db, feature_id, wp_id) = fixture().await;
        let outcome = agileplus_domain::ports::execution::AtomicAcceptancePort::accept_feature_atomic(
            &db,
            &command(feature_id),
        )
        .await
        .expect("accept");

        assert!(!outcome.replayed);
        assert_eq!(
            StoragePort::get_feature_by_id(&db, feature_id)
                .await
                .unwrap()
                .unwrap()
                .state,
            FeatureState::Validated
        );
        assert_eq!(
            StoragePort::get_work_package(&db, wp_id)
                .await
                .unwrap()
                .unwrap()
                .state,
            WpState::Done
        );
        assert_eq!(
            StoragePort::get_audit_trail(&db, feature_id)
                .await
                .unwrap()
                .len(),
            2
        );
        let conn = db.conn_for_bench().unwrap();
        let receipts: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM feature_acceptance_receipts WHERE request_id='request:1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(receipts, 1);
    }

    #[tokio::test]
    async fn exact_request_replays_receipt_but_changed_request_conflicts() {
        let (db, feature_id, _) = fixture().await;
        let first = agileplus_domain::ports::execution::AtomicAcceptancePort::accept_feature_atomic(
            &db,
            &command(feature_id),
        )
        .await
        .expect("first");
        let replay = agileplus_domain::ports::execution::AtomicAcceptancePort::accept_feature_atomic(
            &db,
            &command(feature_id),
        )
        .await
        .expect("replay");
        assert!(replay.replayed);
        assert_eq!(replay.receipt, first.receipt);

        let mut conflicting = command(feature_id);
        conflicting.actor = "different".into();
        let err = agileplus_domain::ports::execution::AtomicAcceptancePort::accept_feature_atomic(
            &db,
            &conflicting,
        )
        .await
        .expect_err("conflict");
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn late_audit_failure_rolls_back_wp_feature_event_and_receipt() {
        let (db, feature_id, wp_id) = fixture().await;
        {
            let conn = db.conn_for_bench().unwrap();
            conn.execute(
                "INSERT INTO audit_log
                 (feature_id,wp_id,timestamp,actor,transition,evidence_refs,prev_hash,hash)
                 VALUES (?1,NULL,?2,'corrupt','seed','[]',?3,?4)",
                params![
                    feature_id,
                    Utc::now().to_rfc3339(),
                    vec![0_u8; 32],
                    vec![9_u8; 3]
                ],
            )
            .unwrap();
        }

        let err = agileplus_domain::ports::execution::AtomicAcceptancePort::accept_feature_atomic(
            &db,
            &command(feature_id),
        )
        .await
        .expect_err("corrupt audit must fail");
        assert!(matches!(err, DomainError::Storage(_)));

        assert_eq!(
            StoragePort::get_feature_by_id(&db, feature_id)
                .await
                .unwrap()
                .unwrap()
                .state,
            FeatureState::Implementing
        );
        assert_eq!(
            StoragePort::get_work_package(&db, wp_id)
                .await
                .unwrap()
                .unwrap()
                .state,
            WpState::Review
        );
        let conn = db.conn_for_bench().unwrap();
        let receipts: i64 = conn
            .query_row("SELECT COUNT(*) FROM feature_acceptance_receipts", [], |row| row.get(0))
            .unwrap();
        let events: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE entity_type='feature' AND entity_id=?1",
                [feature_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(receipts, 0);
        assert_eq!(events, 0);
    }
}
