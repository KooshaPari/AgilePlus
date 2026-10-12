//! SQLite terminal acceptance: one immediate transaction owns reads and writes.
//! No await, network operation, or separately locked adapter call occurs in it.

use super::{audit, events, evidence, execution, features, governance, metrics, work_packages};
use crate::SqliteStorageAdapter;
use agileplus_domain::{
    domain::{
        acceptance::{
            AcceptFeatureCommand, AcceptanceOutcome, FeatureAcceptanceReceipt, validate_candidate,
        },
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

fn storage(error: impl std::fmt::Display) -> DomainError {
    DomainError::Storage(error.to_string())
}
fn invalid(message: impl Into<String>) -> DomainError {
    DomainError::Validation(message.into())
}

#[async_trait::async_trait]
impl AtomicAcceptancePort for SqliteStorageAdapter {
    async fn accept_feature_atomic(
        &self,
        command: &AcceptFeatureCommand,
    ) -> Result<AcceptanceOutcome, DomainError> {
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
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    let prior: Option<(String, String)> = tx.query_row(
        "SELECT command_json, receipt_json FROM feature_acceptance_receipts WHERE request_id=?1",
        [&command.request_id], |r| Ok((r.get(0)?, r.get(1)?)),
    ).optional().map_err(storage)?;
    if let Some((original, receipt_json)) = prior {
        let original: AcceptFeatureCommand = serde_json::from_str(&original).map_err(storage)?;
        if original != *command {
            return Err(DomainError::Conflict(
                "acceptance request ID was used for a different command".into(),
            ));
        }
        let receipt = serde_json::from_str(&receipt_json).map_err(storage)?;
        tx.commit().map_err(storage)?;
        return Ok(AcceptanceOutcome {
            receipt,
            replayed: true,
        });
    }

    let feature = features::get_feature_by_id(&tx, command.feature_id)?
        .ok_or_else(|| DomainError::NotFound(format!("feature {}", command.feature_id)))?;
    if feature.state != FeatureState::Implementing {
        return Err(invalid("terminal acceptance requires Implementing"));
    }
    let contract = governance::get_latest_governance_contract(&tx, feature.id)?
        .ok_or_else(|| invalid("no governance contract; acceptance is NotConfigured"))?;
    if command
        .expected_governance_version
        .is_some_and(|version| version != contract.version)
    {
        return Err(DomainError::Conflict(
            "governance changed after preflight".into(),
        ));
    }
    let wps = work_packages::list_wps_by_feature(&tx, feature.id)?;
    if wps.is_empty() {
        return Err(invalid(
            "no work packages; terminal acceptance cannot be vacuous",
        ));
    }
    let mut evidence_rows = Vec::new();
    for wp in &wps {
        evidence_rows.extend(evidence::get_evidence_by_wp(&tx, wp.id)?);
    }
    let policy_rows = governance::list_active_policies(&tx)?;
    let metric_rows = metrics::get_metrics_by_feature(&tx, feature.id)?;
    let governance = evaluate_governance_snapshot(
        &contract,
        &evidence_rows,
        &policy_rows,
        &metric_rows,
        GovernanceEvaluationOptions::default(),
    );
    if !governance.passed(&contract) {
        return Err(invalid(
            "current governance evaluation did not pass; acceptance denied",
        ));
    }

    let mut accepted_candidates = Vec::with_capacity(wps.len());
    for wp in &wps {
        let assignment = execution::get_active_assignment(&tx, wp.id)?
            .ok_or_else(|| invalid(format!("WP{:02} has no active Assignment", wp.sequence)))?;
        let spec_feature: Option<i64> = tx
            .query_row(
                "SELECT feature_id FROM spec_revisions WHERE id=?1",
                [&assignment.spec_revision_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage)?;
        if spec_feature != Some(feature.id) {
            return Err(invalid("Assignment SpecRevision is outside feature scope"));
        }
        let evaluations = execution::list_evaluations(&tx, &assignment.id)?;
        let evaluation = evaluations
            .last()
            .ok_or_else(|| invalid(format!("WP{:02} has no Evaluation", wp.sequence)))?;
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
    let history = audit::get_audit_trail(&tx, feature.id)?;
    if !history.is_empty() {
        AuditChain { entries: history }
            .verify_chain()
            .map_err(invalid)?;
    }

    let now = Utc::now();
    let mut audit_ids = Vec::new();
    for wp in &wps {
        if wp.state == WpState::Review {
            let changed = tx.execute(
                "UPDATE work_packages SET state='done',updated_at=?1 WHERE id=?2 AND feature_id=?3 AND state='review'",
                params![now.to_rfc3339(), wp.id, feature.id],
            ).map_err(storage)?;
            if changed != 1 {
                return Err(DomainError::Conflict(
                    "work package changed during acceptance".into(),
                ));
            }
            audit_ids.push(append_audit(
                &tx,
                command,
                Some(wp.id),
                &format!(
                    "WP{:02} Review -> Done (exact-candidate correctness + governance accepted)",
                    wp.sequence
                ),
                now,
            )?);
        }
    }
    let changed = tx.execute(
        "UPDATE features SET state='validated',updated_at=?1 WHERE id=?2 AND state='implementing'",
        params![now.to_rfc3339(), feature.id],
    ).map_err(storage)?;
    if changed != 1 {
        return Err(DomainError::Conflict(
            "feature changed during acceptance".into(),
        ));
    }
    audit_ids.push(append_audit(
        &tx,
        command,
        None,
        "Implementing -> Validated",
        now,
    )?);

    let previous_events = events::get_events(&tx, "feature", feature.id)?;
    let sequence = events::get_latest_sequence(&tx, "feature", feature.id)? + 1;
    let previous_hash = previous_events.last().map(|e| e.hash).unwrap_or([0; 32]);
    let payload = serde_json::json!({
        "from":"Implementing", "to":"Validated", "acceptance_request_id":command.request_id,
        "governance_contract_id":contract.id, "governance_version":contract.version,
        "accepted_candidates":accepted_candidates, "audit_ids":audit_ids,
    });
    let mut event = Event::new(
        "feature",
        feature.id,
        "state_transitioned",
        payload,
        command.actor.as_str(),
    );
    event.timestamp = now;
    event.sequence = sequence;
    event.prev_hash = previous_hash;
    event.hash = agileplus_events::compute_hash(
        event.entity_id,
        &event.entity_type,
        &event.event_type,
        &event.payload,
        event.timestamp,
        &event.actor,
        &event.prev_hash,
    )
    .map_err(storage)?;
    let event_id = events::append_event(&tx, &event)?;
    let receipt = FeatureAcceptanceReceipt {
        request_id: command.request_id.clone(),
        feature_id: feature.id,
        actor: command.actor.clone(),
        governance_contract_id: contract.id,
        governance_version: contract.version,
        accepted_candidates,
        audit_ids,
        event_id,
        committed_at: now,
    };
    tx.execute(
        "INSERT INTO feature_acceptance_receipts (request_id,feature_id,command_json,receipt_json,committed_at) VALUES (?1,?2,?3,?4,?5)",
        params![command.request_id, feature.id, serde_json::to_string(command).map_err(storage)?,
            serde_json::to_string(&receipt).map_err(storage)?, now.to_rfc3339()],
    ).map_err(storage)?;
    tx.commit().map_err(storage)?;
    Ok(AcceptanceOutcome {
        receipt,
        replayed: false,
    })
}

fn append_audit(
    connection: &Connection,
    command: &AcceptFeatureCommand,
    wp_id: Option<i64>,
    transition: &str,
    timestamp: chrono::DateTime<Utc>,
) -> Result<i64, DomainError> {
    let prev_hash = audit::get_latest_audit_entry(connection, command.feature_id)?
        .map(|entry| entry.hash)
        .unwrap_or([0; 32]);
    let mut entry = AuditEntry {
        id: 0,
        feature_id: command.feature_id,
        wp_id,
        timestamp,
        actor: command.actor.clone(),
        transition: transition.into(),
        evidence_refs: Vec::new(),
        prev_hash,
        hash: [0; 32],
        event_id: None,
        archived_to: None,
    };
    entry.hash = hash_entry(&entry);
    audit::append_audit_entry(connection, &entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;
    use agileplus_domain::{
        domain::{
            execution::{
                Assignment, AssignmentStatus, Attempt, AttemptStatus, CriterionEvaluation,
                Evaluation, EvaluationResult, SpecRevision, aggregate_evidence_refs,
                reduce_criterion_results, snapshot_acceptance_criteria,
            },
            feature::Feature,
            governance::{Evidence, EvidenceType, GovernanceContract, GovernanceRule},
            work_package::WorkPackage,
        },
        ports::{ExecutionRecordPort, StoragePort},
    };

    async fn fixture() -> (SqliteStorageAdapter, i64, i64) {
        let db = SqliteStorageAdapter::in_memory().expect("in-memory sqlite");
        let mut feature = Feature::new("atomic-acceptance", "Atomic acceptance", [7; 32], None);
        feature.state = FeatureState::Implementing;
        let feature_id = StoragePort::create_feature(&db, &feature)
            .await
            .expect("feature");

        let mut wp = WorkPackage::new(feature_id, "WP", 1, "criterion");
        wp.state = WpState::Review;
        let wp_id = StoragePort::create_work_package(&db, &wp)
            .await
            .expect("wp");

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
        .expect("governance");
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
            id: "spec:atomic".into(),
            feature_id,
            content_hash: "sha256:atomic".into(),
            parent_revision_id: None,
            accepted_at: Utc::now(),
            authority: "test".into(),
        };
        ExecutionRecordPort::create_spec_revision(&db, &spec)
            .await
            .expect("spec");
        let assignment = Assignment {
            id: "assignment:atomic".into(),
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
            id: "attempt:atomic".into(),
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
            id: "evaluation:atomic".into(),
            assignment_id: assignment.id,
            attempt_id: Some(attempt.id),
            candidate_ref: "git:candidate".into(),
            evaluator_id: "independent".into(),
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
            request_id: "acceptance-request:atomic".into(),
            feature_id,
            actor: "test-actor".into(),
            expected_governance_version: Some(1),
        }
    }

    #[tokio::test]
    async fn commit_is_atomic_and_persists_receipt_event_and_terminal_states() {
        let (db, feature_id, wp_id) = fixture().await;
        let outcome = AtomicAcceptancePort::accept_feature_atomic(&db, &command(feature_id))
            .await
            .expect("acceptance");

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
        let receipt_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM feature_acceptance_receipts WHERE request_id=?1",
                [&command(feature_id).request_id],
                |row| row.get(0),
            )
            .unwrap();
        let event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE entity_type='feature' AND entity_id=?1",
                [feature_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(receipt_count, 1);
        assert_eq!(event_count, 1);
    }

    #[tokio::test]
    async fn exact_replay_returns_original_receipt_and_changed_command_conflicts() {
        let (db, feature_id, _) = fixture().await;
        let first = AtomicAcceptancePort::accept_feature_atomic(&db, &command(feature_id))
            .await
            .expect("first");
        let replay = AtomicAcceptancePort::accept_feature_atomic(&db, &command(feature_id))
            .await
            .expect("replay");
        assert!(replay.replayed);
        assert_eq!(replay.receipt, first.receipt);

        let mut changed = command(feature_id);
        changed.actor = "different-actor".into();
        let err = AtomicAcceptancePort::accept_feature_atomic(&db, &changed)
            .await
            .expect_err("same request ID with changed command must conflict");
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn late_receipt_failure_rolls_back_terminal_states_audits_and_event() {
        let (db, feature_id, wp_id) = fixture().await;
        {
            let conn = db.conn_for_bench().unwrap();
            conn.execute_batch(
                "CREATE TRIGGER fail_acceptance_receipt
                 BEFORE INSERT ON feature_acceptance_receipts
                 BEGIN
                   SELECT RAISE(ABORT, 'forced late receipt failure');
                 END;",
            )
            .unwrap();
        }

        AtomicAcceptancePort::accept_feature_atomic(&db, &command(feature_id))
            .await
            .expect_err("forced late receipt failure must roll back the transaction");

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
        assert!(
            StoragePort::get_audit_trail(&db, feature_id)
                .await
                .unwrap()
                .is_empty()
        );
        let conn = db.conn_for_bench().unwrap();
        let receipts: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM feature_acceptance_receipts",
                [],
                |row| row.get(0),
            )
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

    #[tokio::test]
    async fn invalid_existing_audit_chain_fails_before_any_terminal_mutation() {
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
                    vec![9_u8; 32]
                ],
            )
            .unwrap();
        }

        AtomicAcceptancePort::accept_feature_atomic(&db, &command(feature_id))
            .await
            .expect_err("corrupt audit chain must fail acceptance");

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
            .query_row(
                "SELECT COUNT(*) FROM feature_acceptance_receipts",
                [],
                |row| row.get(0),
            )
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
