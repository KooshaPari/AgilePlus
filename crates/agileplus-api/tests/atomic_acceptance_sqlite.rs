// SPDX-License-Identifier: MIT OR Apache-2.0
//! Real-SQLite HTTP witness for canonical terminal acceptance.

use std::sync::Arc;

use agileplus_api::{AppState, create_router};
use agileplus_domain::{
    config::AppConfig,
    credentials::{
        CredentialStore, InMemoryCredentialStore, format_api_key_hash, keys as cred_keys,
    },
    domain::{
        execution::{
            Assignment, AssignmentStatus, Attempt, AttemptStatus, CriterionEvaluation, Evaluation,
            EvaluationResult, SpecRevision, aggregate_evidence_refs, reduce_criterion_results,
            snapshot_acceptance_criteria,
        },
        feature::Feature,
        governance::{Evidence, EvidenceType, GovernanceContract, GovernanceRule},
        state_machine::FeatureState,
        work_package::{WorkPackage, WpState},
    },
    ports::{ExecutionRecordPort, StoragePort},
};
use agileplus_sqlite::SqliteStorageAdapter;
use axum::http::StatusCode;
use axum_test::TestServer;
use chrono::Utc;

#[path = "api_integration/support/observability.rs"]
mod observability;
#[path = "api_integration/support/vcs.rs"]
mod vcs;

use observability::MockObs;
use vcs::MockVcs;

const API_KEY: &str = "atomic-http-key";

fn credentials() -> Arc<dyn CredentialStore> {
    let store = InMemoryCredentialStore::new();
    store
        .set(
            "agileplus",
            cred_keys::API_KEYS,
            &format_api_key_hash(API_KEY),
        )
        .expect("api key");
    Arc::new(store)
}

async fn fixture() -> (TestServer, Arc<SqliteStorageAdapter>, i64, i64) {
    let storage = Arc::new(SqliteStorageAdapter::in_memory().expect("sqlite"));
    let mut feature = Feature::new("atomic-http", "Atomic HTTP", [5; 32], None);
    feature.state = FeatureState::Implementing;
    let feature_id = StoragePort::create_feature(storage.as_ref(), &feature)
        .await
        .expect("feature");

    let mut wp = WorkPackage::new(feature_id, "WP", 1, "criterion");
    wp.state = WpState::Review;
    let wp_id = StoragePort::create_work_package(storage.as_ref(), &wp)
        .await
        .expect("wp");

    StoragePort::create_governance_contract(
        storage.as_ref(),
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
        storage.as_ref(),
        &Evidence {
            id: 0,
            wp_id,
            fr_id: "FR-1".into(),
            evidence_type: EvidenceType::TestResult,
            artifact_path: "artifact:http".into(),
            metadata: None,
            created_at: Utc::now(),
        },
    )
    .await
    .expect("evidence");

    let spec = SpecRevision {
        id: "spec:http".into(),
        feature_id,
        content_hash: "sha256:http".into(),
        parent_revision_id: None,
        accepted_at: Utc::now(),
        authority: "test".into(),
    };
    ExecutionRecordPort::create_spec_revision(storage.as_ref(), &spec)
        .await
        .expect("spec");
    let assignment = Assignment {
        id: "assignment:http".into(),
        wp_id,
        spec_revision_id: spec.id,
        created_at: Utc::now(),
        supersedes_assignment_id: None,
        status: AssignmentStatus::Active,
    };
    let criteria = snapshot_acceptance_criteria("criterion");
    ExecutionRecordPort::create_assignment_with_criteria(storage.as_ref(), &assignment, &criteria)
        .await
        .expect("assignment");
    let attempt = Attempt {
        id: "attempt:http".into(),
        assignment_id: assignment.id.clone(),
        worker_id: "worker".into(),
        backend: "test".into(),
        job_id: None,
        worktree_path: Some("/tmp/http".into()),
        base_candidate_ref: Some("git:base".into()),
        result_candidate_ref: Some("git:http-candidate".into()),
        status: AttemptStatus::Completed,
        failure_class: None,
        started_at: Utc::now(),
        ended_at: Some(Utc::now()),
    };
    ExecutionRecordPort::create_attempt(storage.as_ref(), &attempt)
        .await
        .expect("attempt");
    let results = vec![CriterionEvaluation {
        criterion_id: criteria[0].id.clone(),
        result: EvaluationResult::Satisfied,
        evidence_refs: vec!["evidence:http".into()],
        rationale: None,
    }];
    let evaluation = Evaluation {
        id: "evaluation:http".into(),
        assignment_id: assignment.id,
        attempt_id: Some(attempt.id),
        candidate_ref: "git:http-candidate".into(),
        evaluator_id: "independent".into(),
        evaluator_version: "1".into(),
        result: reduce_criterion_results(&criteria, &results),
        evidence_refs: aggregate_evidence_refs(&results),
        started_at: Utc::now(),
        finished_at: Utc::now(),
    };
    ExecutionRecordPort::create_evaluation_receipt(storage.as_ref(), &evaluation, &results)
        .await
        .expect("evaluation");

    let state = AppState::new(
        storage.clone(),
        Arc::new(MockVcs::new()),
        Arc::new(MockObs),
        Arc::new(AppConfig::default()),
        credentials(),
    )
    .with_atomic_acceptance();
    (
        TestServer::new(create_router(state)),
        storage,
        feature_id,
        wp_id,
    )
}

#[tokio::test]
async fn acceptance_endpoint_requires_auth_and_commits_real_sqlite_transaction() {
    let (server, storage, feature_id, wp_id) = fixture().await;
    let body = serde_json::json!({
        "request_id": "http-request:1",
        "expected_governance_version": 1
    });

    server
        .post("/api/v1/features/atomic-http/accept")
        .json(&body)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    let response = server
        .post("/api/v1/features/atomic-http/accept")
        .add_header("X-API-Key", API_KEY)
        .json(&body)
        .await;
    response.assert_status_ok();
    let value: serde_json::Value = response.json();
    assert_eq!(value["replayed"], false);
    assert_eq!(value["receipt"]["feature_id"], feature_id);

    assert_eq!(
        StoragePort::get_feature_by_id(storage.as_ref(), feature_id)
            .await
            .unwrap()
            .unwrap()
            .state,
        FeatureState::Validated
    );
    assert_eq!(
        StoragePort::get_work_package(storage.as_ref(), wp_id)
            .await
            .unwrap()
            .unwrap()
            .state,
        WpState::Done
    );
}

#[tokio::test]
async fn acceptance_endpoint_replays_exact_request_and_rejects_conflicting_reuse() {
    let (server, _storage, _feature_id, _wp_id) = fixture().await;
    let first = serde_json::json!({
        "request_id": "http-request:replay",
        "expected_governance_version": 1
    });
    server
        .post("/api/v1/features/atomic-http/accept")
        .add_header("X-API-Key", API_KEY)
        .json(&first)
        .await
        .assert_status_ok();

    let replay = server
        .post("/api/v1/features/atomic-http/accept")
        .add_header("X-API-Key", API_KEY)
        .json(&first)
        .await;
    replay.assert_status_ok();
    let value: serde_json::Value = replay.json();
    assert_eq!(value["replayed"], true);

    let conflict = server
        .post("/api/v1/features/atomic-http/accept")
        .add_header("X-API-Key", API_KEY)
        .json(&serde_json::json!({
            "request_id": "http-request:replay",
            "expected_governance_version": 999
        }))
        .await;
    conflict.assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn acceptance_precondition_failure_is_422_and_does_not_mutate_terminal_state() {
    let (server, storage, feature_id, wp_id) = fixture().await;
    {
        let conn = storage.conn_for_bench().expect("sqlite connection");
        conn.execute("DELETE FROM evaluations", [])
            .expect("remove evaluation");
    }

    let response = server
        .post("/api/v1/features/atomic-http/accept")
        .add_header("X-API-Key", API_KEY)
        .json(&serde_json::json!({
            "request_id": "http-request:invalid",
            "expected_governance_version": 1
        }))
        .await;
    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);

    assert_eq!(
        StoragePort::get_feature_by_id(storage.as_ref(), feature_id)
            .await
            .unwrap()
            .unwrap()
            .state,
        FeatureState::Implementing
    );
    assert_eq!(
        StoragePort::get_work_package(storage.as_ref(), wp_id)
            .await
            .unwrap()
            .unwrap()
            .state,
        WpState::Review
    );
}
