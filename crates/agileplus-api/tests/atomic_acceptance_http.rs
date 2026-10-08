//! Mounted HTTP acceptance witnesses: authentication, real SQLite, commit and rollback.
use agileplus_api::{AppState, create_router};
use agileplus_domain::{
    config::AppConfig,
    credentials::{CredentialStore, InMemoryCredentialStore},
    ports::observability::{LogEntry, ObservabilityPort, SpanContext},
};
use agileplus_git::GitVcsAdapter;
use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};
use std::sync::Arc;

use agileplus_domain::{
    domain::{
        execution::{
            Assignment, AssignmentStatus, Attempt, AttemptStatus, CriterionEvaluation, Evaluation,
            EvaluationResult, SpecRevision, snapshot_acceptance_criteria,
        },
        feature::Feature,
        governance::{Evidence, EvidenceType, GovernanceContract, GovernanceRule},
        state_machine::FeatureState,
        work_package::{WorkPackage, WpState},
    },
    ports::{ExecutionRecordPort, StoragePort},
};
use agileplus_sqlite::SqliteStorageAdapter;
use chrono::Utc;

pub(crate) async fn seed(db: &SqliteStorageAdapter) -> i64 {
    let mut feature = Feature::new("atomic", "Atomic acceptance", [7; 32], None);
    feature.state = FeatureState::Implementing;
    let feature_id = StoragePort::create_feature(db, &feature).await.unwrap();
    let now = Utc::now();
    let revision = SpecRevision {
        id: "spec:1".into(),
        feature_id,
        content_hash: "sha256:accepted".into(),
        parent_revision_id: None,
        accepted_at: now,
        authority: "owner".into(),
    };
    db.create_spec_revision(&revision).await.unwrap();
    for seq in [1, 2] {
        let mut wp = WorkPackage::new(feature_id, "Atomic WP", seq, "result meets the contract");
        wp.state = WpState::Review;
        let wp_id = StoragePort::create_work_package(db, &wp).await.unwrap();
        StoragePort::create_evidence(
            db,
            &Evidence {
                id: 0,
                wp_id,
                fr_id: format!("FR-{seq}"),
                evidence_type: EvidenceType::TestResult,
                artifact_path: "artifact:fixture".into(),
                metadata: None,
                created_at: now,
            },
        )
        .await
        .unwrap();
        let assignment = Assignment {
            id: format!("assignment:{seq}"),
            wp_id,
            spec_revision_id: revision.id.clone(),
            created_at: now,
            supersedes_assignment_id: None,
            status: AssignmentStatus::Active,
        };
        let criteria = snapshot_acceptance_criteria("result meets the contract");
        db.create_assignment_with_criteria(&assignment, &criteria)
            .await
            .unwrap();
        let candidate_ref = format!("git:{:040x}", seq);
        let attempt = Attempt {
            id: format!("attempt:{seq}"),
            assignment_id: assignment.id.clone(),
            worker_id: "worker".into(),
            backend: "test".into(),
            job_id: None,
            worktree_path: None,
            base_candidate_ref: Some(format!("git:{:040x}", 0)),
            result_candidate_ref: Some(candidate_ref.clone()),
            status: AttemptStatus::Completed,
            failure_class: None,
            started_at: now,
            ended_at: Some(now),
        };
        db.create_attempt(&attempt).await.unwrap();
        let result = CriterionEvaluation {
            criterion_id: criteria[0].id.clone(),
            result: EvaluationResult::Satisfied,
            evidence_refs: vec![format!("artifact:proof-{seq}")],
            rationale: Some("fixture".into()),
        };
        db.create_evaluation_receipt(
            &Evaluation {
                id: format!("evaluation:{seq}"),
                assignment_id: assignment.id,
                attempt_id: Some(attempt.id),
                candidate_ref,
                evaluator_id: "independent-grader".into(),
                evaluator_version: "1".into(),
                result: EvaluationResult::Satisfied,
                evidence_refs: result.evidence_refs.clone(),
                started_at: now,
                finished_at: now,
            },
            &[result],
        )
        .await
        .unwrap();
    }
    StoragePort::create_governance_contract(
        db,
        &GovernanceContract {
            id: 0,
            feature_id,
            version: 1,
            bound_at: now,
            rules: vec![GovernanceRule {
                transition: "Implementing -> Validated".into(),
                required_evidence: vec!["FR-1:test_result".into(), "FR-2:test_result".into()],
                policy_refs: Vec::new(),
            }],
        },
    )
    .await
    .unwrap();
    feature_id
}

const KEY: &str = "atomic-acceptance-fixture-only";
pub(crate) struct Noop;
impl ObservabilityPort for Noop {
    fn start_span(&self, _: &str, _: Option<&SpanContext>) -> SpanContext {
        SpanContext {
            trace_id: String::new(),
            span_id: String::new(),
            parent_span_id: None,
        }
    }
    fn end_span(&self, _: &SpanContext) {}
    fn add_span_event(&self, _: &SpanContext, _: &str, _: &[(&str, &str)]) {}
    fn set_span_error(&self, _: &SpanContext, _: &str) {}
    fn record_counter(&self, _: &str, _: u64, _: &[(&str, &str)]) {}
    fn record_histogram(&self, _: &str, _: f64, _: &[(&str, &str)]) {}
    fn record_gauge(&self, _: &str, _: f64, _: &[(&str, &str)]) {}
    fn log(&self, _: &LogEntry) {}
    fn log_info(&self, _: &str) {}
    fn log_warn(&self, _: &str) {}
    fn log_error(&self, _: &str) {}
}
async fn server(configured: bool) -> (TestServer, Arc<SqliteStorageAdapter>, i64) {
    let db = Arc::new(SqliteStorageAdapter::in_memory().unwrap());
    let id = seed(db.as_ref()).await;
    (mounted_server(db.clone(), configured), db, id)
}

fn mounted_server(db: Arc<SqliteStorageAdapter>, configured: bool) -> TestServer {
    let credentials: Arc<dyn CredentialStore> = Arc::new(InMemoryCredentialStore::new());
    agileplus_api::api_key::import_api_key(credentials.as_ref(), KEY).unwrap();
    let mut state = AppState::new(
        db.clone(),
        Arc::new(GitVcsAdapter::new(std::env::temp_dir())),
        Arc::new(Noop),
        Arc::new(AppConfig::default()),
        credentials,
    );
    if configured {
        state = state.with_atomic_acceptance();
    }
    TestServer::new(create_router(state))
}
fn request() -> Value {
    json!({"request_id":"http:request:1", "expected_governance_version":1})
}
fn count(db: &SqliteStorageAdapter, table: &str) -> i64 {
    db.conn_for_bench()
        .unwrap()
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[tokio::test]
async fn receipt_lookup_requires_auth_and_never_awards_acceptance() {
    let (api, db, id) = server(true).await;
    api.get("/api/v1/features/atomic/acceptance-receipt")
        .await
        .assert_status_unauthorized();
    let missing = api
        .get("/api/v1/features/atomic/acceptance-receipt")
        .add_header("X-API-Key", KEY)
        .await;
    missing.assert_status(StatusCode::NOT_FOUND);
    assert_eq!(
        missing.json::<Value>()["error"],
        "acceptance_receipt_not_found"
    );
    api.get("/api/v1/features/missing/acceptance-receipt")
        .add_header("X-API-Key", KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
    assert_eq!(
        StoragePort::get_feature_by_id(db.as_ref(), id)
            .await
            .unwrap()
            .unwrap()
            .state,
        FeatureState::Implementing
    );
    for table in ["audit_log", "events", "feature_acceptance_receipts"] {
        assert_eq!(count(&db, table), 0);
    }
}

#[tokio::test]
async fn mounted_receipt_survives_database_reopen_without_regrading() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("receipt.db");
    let original = {
        let db = Arc::new(SqliteStorageAdapter::new(&path).unwrap());
        seed(db.as_ref()).await;
        let api = mounted_server(db, true);
        api.post("/api/v1/features/atomic/accept")
            .add_header("X-API-Key", KEY)
            .json(&request())
            .await
            .json::<Value>()["receipt"]
            .clone()
    };
    let db = Arc::new(SqliteStorageAdapter::new(&path).unwrap());
    let api = mounted_server(db.clone(), true);
    let result = api
        .get("/api/v1/features/atomic/acceptance-receipt")
        .add_header("X-API-Key", KEY)
        .await;
    result.assert_status_ok();
    assert_eq!(result.json::<Value>(), original);
    assert_eq!(count(&db, "events"), 1);
    assert_eq!(count(&db, "feature_acceptance_receipts"), 1);
}

#[tokio::test]
async fn receipt_lookup_redacts_storage_failures() {
    let (api, db, _) = server(true).await;
    db.conn_for_bench()
        .unwrap()
        .execute_batch("DROP TABLE feature_acceptance_receipts")
        .unwrap();
    let result = api
        .get("/api/v1/features/atomic/acceptance-receipt")
        .add_header("X-API-Key", KEY)
        .await;
    result.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    assert!(!result.json::<Value>().to_string().contains("no such table"));
}

#[tokio::test]
async fn authenticated_http_acceptance_commits_and_replays_one_receipt() {
    let (api, db, id) = server(true).await;
    let response = api
        .post("/api/v1/features/atomic/accept")
        .add_header("X-API-Key", KEY)
        .json(&request())
        .await;
    response.assert_status_ok();
    let first: Value = response.json();
    assert_eq!(first["replayed"], false);
    assert_eq!(first["receipt"]["actor"], "http:api-key");
    assert_eq!(
        first["receipt"]["accepted_candidates"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let response = api
        .post("/api/v1/features/atomic/accept")
        .add_header("X-API-Key", KEY)
        .json(&request())
        .await;
    response.assert_status_ok();
    let replay: Value = response.json();
    assert_eq!(replay["replayed"], true);
    assert_eq!(first["receipt"], replay["receipt"]);
    assert_eq!(
        StoragePort::get_feature_by_id(db.as_ref(), id)
            .await
            .unwrap()
            .unwrap()
            .state,
        FeatureState::Validated
    );
    assert_eq!(count(&db, "events"), 1);
}

#[tokio::test]
async fn unauthorized_http_cannot_reach_acceptance() {
    let (api, db, id) = server(true).await;
    api.post("/api/v1/features/atomic/accept")
        .json(&request())
        .await
        .assert_status_unauthorized();
    assert_eq!(
        StoragePort::get_feature_by_id(db.as_ref(), id)
            .await
            .unwrap()
            .unwrap()
            .state,
        FeatureState::Implementing
    );
    assert_eq!(count(&db, "feature_acceptance_receipts"), 0);
}

#[tokio::test]
async fn missing_feature_returns_404_without_acceptance_side_effects() {
    let (api, db, _) = server(true).await;
    api.post("/api/v1/features/missing/accept")
        .add_header("X-API-Key", KEY)
        .json(&request())
        .await
        .assert_status(StatusCode::NOT_FOUND);
    for table in ["audit_log", "events", "feature_acceptance_receipts"] {
        assert_eq!(count(&db, table), 0);
    }
}

#[tokio::test]
async fn stale_governance_expectation_cannot_accept_over_http() {
    let (api, db, id) = server(true).await;
    let mut payload = request();
    payload["expected_governance_version"] = json!(0);
    api.post("/api/v1/features/atomic/accept")
        .add_header("X-API-Key", KEY)
        .json(&payload)
        .await
        .assert_status(StatusCode::CONFLICT);
    assert_eq!(
        StoragePort::get_feature_by_id(db.as_ref(), id)
            .await
            .unwrap()
            .unwrap()
            .state,
        FeatureState::Implementing
    );
    for table in ["audit_log", "events", "feature_acceptance_receipts"] {
        assert_eq!(count(&db, table), 0);
    }
}

#[tokio::test]
async fn client_cannot_supply_a_pass_or_actor_override() {
    let (api, db, _) = server(true).await;
    for extra in [
        json!({"passed":true}),
        json!({"authoritative":true}),
        json!({"actor":"owner"}),
    ] {
        let mut payload = request();
        payload
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let response = api
            .post("/api/v1/features/atomic/accept")
            .add_header("X-API-Key", KEY)
            .json(&payload)
            .await;
        assert!(response.status_code().is_client_error());
    }
    assert_eq!(count(&db, "feature_acceptance_receipts"), 0);
}

#[tokio::test]
async fn persistence_failure_over_http_leaves_no_terminal_state_or_receipt() {
    let (api, db, id) = server(true).await;
    db.conn_for_bench().unwrap().execute_batch("CREATE TRIGGER reject_event BEFORE INSERT ON events BEGIN SELECT RAISE(ABORT,'private database detail'); END;").unwrap();
    let response = api
        .post("/api/v1/features/atomic/accept")
        .add_header("X-API-Key", KEY)
        .json(&request())
        .await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    let body: Value = response.json();
    assert!(!body.to_string().contains("private database detail"));
    assert_eq!(
        StoragePort::get_feature_by_id(db.as_ref(), id)
            .await
            .unwrap()
            .unwrap()
            .state,
        FeatureState::Implementing
    );
    for wp in StoragePort::list_wps_by_feature(db.as_ref(), id)
        .await
        .unwrap()
    {
        assert_eq!(wp.state, WpState::Review);
    }
    for table in ["audit_log", "events", "feature_acceptance_receipts"] {
        assert_eq!(count(&db, table), 0);
    }
}

#[tokio::test]
async fn unsupported_embedder_returns_501_not_an_optimistic_success() {
    let (api, db, _) = server(false).await;
    api.post("/api/v1/features/atomic/accept")
        .add_header("X-API-Key", KEY)
        .json(&request())
        .await
        .assert_status(StatusCode::NOT_IMPLEMENTED);
    assert_eq!(count(&db, "feature_acceptance_receipts"), 0);
}
