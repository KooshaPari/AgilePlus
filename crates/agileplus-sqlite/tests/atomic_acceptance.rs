//! Real SQLite acceptance tests: proof, writes, audit, event, receipt, replay.
#[path = "atomic_acceptance/fixture.rs"]
mod fixture;

use agileplus_domain::{
    domain::{audit::AuditChain, state_machine::FeatureState, work_package::WpState},
    ports::{StoragePort, execution::AtomicAcceptancePort},
};
use agileplus_sqlite::SqliteStorageAdapter;
use fixture::seed;

#[tokio::test]
async fn acceptance_commits_states_audit_event_and_receipt_together() {
    let f = seed(SqliteStorageAdapter::in_memory().unwrap()).await;
    let outcome = f.db.accept_feature_atomic(&f.command()).await.unwrap();
    assert!(!outcome.replayed);
    assert_eq!(outcome.receipt.accepted_candidates.len(), 2);
    assert_eq!(outcome.receipt.audit_ids.len(), 3);
    assert_eq!(f.count("events"), 1);
    assert_eq!(f.count("feature_acceptance_receipts"), 1);
    assert_eq!(
        StoragePort::get_feature_by_id(&f.db, f.feature_id)
            .await
            .unwrap()
            .unwrap()
            .state,
        FeatureState::Validated
    );
    for wp in &f.wp_ids {
        assert_eq!(
            StoragePort::get_work_package(&f.db, *wp)
                .await
                .unwrap()
                .unwrap()
                .state,
            WpState::Done
        );
    }
    AuditChain {
        entries: StoragePort::get_audit_trail(&f.db, f.feature_id)
            .await
            .unwrap(),
    }
    .verify_chain()
    .unwrap();
    let events = agileplus_sqlite::repository::events::get_events(
        &f.db.conn_for_bench().unwrap(),
        "feature",
        f.feature_id,
    )
    .unwrap();
    assert_eq!(events[0].payload["acceptance_request_id"], "request:atomic");
    assert_eq!(
        events[0].payload["accepted_candidates"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        events[0].hash,
        agileplus_events::compute_hash(
            events[0].entity_id,
            &events[0].entity_type,
            &events[0].event_type,
            &events[0].payload,
            events[0].timestamp,
            &events[0].actor,
            &events[0].prev_hash
        )
        .unwrap()
    );
}

#[tokio::test]
async fn failures_at_every_write_boundary_roll_back_everything() {
    for (table, operation, condition) in [
        (
            "work_packages",
            "UPDATE",
            "NEW.sequence=2 AND NEW.state='done'",
        ),
        ("audit_log", "INSERT", "1"),
        ("features", "UPDATE", "NEW.state='validated'"),
        ("events", "INSERT", "1"),
        ("feature_acceptance_receipts", "INSERT", "1"),
    ] {
        let f = seed(SqliteStorageAdapter::in_memory().unwrap()).await;
        f.sql(&format!("CREATE TRIGGER reject_acceptance BEFORE {operation} ON {table} WHEN {condition} BEGIN SELECT RAISE(ABORT,'injected failure'); END;"));
        assert!(
            f.db.accept_feature_atomic(&f.command()).await.is_err(),
            "{table}"
        );
        f.assert_unchanged().await;
        f.sql("DROP TRIGGER reject_acceptance;");
        f.db.accept_feature_atomic(&f.command())
            .await
            .expect("same request can retry after rollback");
        assert_eq!(f.count("feature_acceptance_receipts"), 1);
    }
}

#[tokio::test]
async fn request_replay_is_idempotent_but_changed_intent_conflicts() {
    let f = seed(SqliteStorageAdapter::in_memory().unwrap()).await;
    let command = f.command();
    let first = f.db.accept_feature_atomic(&command).await.unwrap();
    let second = f.db.accept_feature_atomic(&command).await.unwrap();
    assert!(second.replayed);
    assert_eq!(first.receipt, second.receipt);
    let mut changed = command;
    changed.actor = "other-owner".into();
    assert!(f.db.accept_feature_atomic(&changed).await.is_err());
    assert_eq!(f.count("audit_log"), 3);
    assert_eq!(f.count("events"), 1);
    assert_eq!(f.count("feature_acceptance_receipts"), 1);
}

#[tokio::test]
async fn current_missing_evidence_is_not_overridden_by_old_preflight() {
    let f = seed(SqliteStorageAdapter::in_memory().unwrap()).await;
    f.sql("DELETE FROM evidence WHERE fr_id='FR-2';");
    assert!(f.db.accept_feature_atomic(&f.command()).await.is_err());
    f.assert_unchanged().await;
}

#[tokio::test]
async fn current_attempt_or_candidate_drift_prevents_any_acceptance() {
    for mutation in [
        "UPDATE attempts SET status='expired' WHERE id='attempt:2';",
        "UPDATE attempts SET result_candidate_ref='git:different' WHERE id='attempt:2';",
        "UPDATE assignments SET status='superseded' WHERE id='assignment:2';",
        "UPDATE evaluations SET evaluator_id='worker' WHERE id='evaluation:2';",
    ] {
        let f = seed(SqliteStorageAdapter::in_memory().unwrap()).await;
        f.sql(mutation);
        assert!(
            f.db.accept_feature_atomic(&f.command()).await.is_err(),
            "{mutation}"
        );
        f.assert_unchanged().await;
    }
}

#[tokio::test]
async fn newer_governance_or_empty_placeholder_cannot_authorize_acceptance() {
    let f = seed(SqliteStorageAdapter::in_memory().unwrap()).await;
    f.sql("UPDATE governance_contracts SET version=2;");
    assert!(f.db.accept_feature_atomic(&f.command()).await.is_err());
    f.assert_unchanged().await;
    let f = seed(SqliteStorageAdapter::in_memory().unwrap()).await;
    f.sql("UPDATE governance_contracts SET rules='[{\"transition\":\"\",\"required_evidence\":[],\"policy_refs\":[]}]';");
    assert!(f.db.accept_feature_atomic(&f.command()).await.is_err());
    f.assert_unchanged().await;
}

#[tokio::test]
async fn malformed_typed_evidence_requirement_is_not_a_wildcard() {
    let f = seed(SqliteStorageAdapter::in_memory().unwrap()).await;
    f.sql("UPDATE governance_contracts SET rules='[{\"transition\":\"\",\"required_evidence\":[\"FR-1:typo\"],\"policy_refs\":[]}]';");
    assert!(f.db.accept_feature_atomic(&f.command()).await.is_err());
    f.assert_unchanged().await;
}

#[tokio::test]
async fn receipts_cannot_be_silently_rewritten_or_deleted() {
    let f = seed(SqliteStorageAdapter::in_memory().unwrap()).await;
    f.db.accept_feature_atomic(&f.command()).await.unwrap();
    let conn = f.db.conn_for_bench().unwrap();
    assert!(
        conn.execute(
            "UPDATE feature_acceptance_receipts SET receipt_json='{}'",
            []
        )
        .is_err()
    );
    assert!(
        conn.execute("DELETE FROM feature_acceptance_receipts", [])
            .is_err()
    );
}

#[tokio::test]
async fn file_reopen_preserves_atomic_receipt_and_replay() {
    let path = std::env::temp_dir().join(format!(
        "agileplus-acceptance-{}-{}.db",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    let command;
    let receipt;
    {
        let f = seed(SqliteStorageAdapter::new(&path).unwrap()).await;
        command = f.command();
        receipt = f.db.accept_feature_atomic(&command).await.unwrap().receipt;
    }
    {
        let db = SqliteStorageAdapter::new(&path).unwrap();
        let replay = db.accept_feature_atomic(&command).await.unwrap();
        assert!(replay.replayed);
        assert_eq!(replay.receipt, receipt);
        assert_eq!(
            StoragePort::get_feature_by_id(&db, command.feature_id)
                .await
                .unwrap()
                .unwrap()
                .state,
            FeatureState::Validated
        );
    }
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn concurrent_same_request_commits_once() {
    let f = seed(SqliteStorageAdapter::in_memory().unwrap()).await;
    let command = f.command();
    let db = std::sync::Arc::new(f.db);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let workers: Vec<_> = (0..2)
        .map(|_| {
            let db = db.clone();
            let command = command.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                barrier.wait();
                runtime
                    .block_on(db.accept_feature_atomic(&command))
                    .unwrap()
            })
        })
        .collect();
    let outcomes: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
    assert_eq!(outcomes[0].receipt, outcomes[1].receipt);
    assert_ne!(outcomes[0].replayed, outcomes[1].replayed);
}
