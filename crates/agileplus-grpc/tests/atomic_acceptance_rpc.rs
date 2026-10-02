//! Direct tonic service tests over real SQLite, not a network/TLS claim.
mod support;
use agileplus_domain::{
    credentials::{CredentialStore, InMemoryCredentialStore, format_api_key_hash, keys},
    domain::{
        execution::{
            Assignment, AssignmentStatus, Attempt, AttemptStatus, CriterionEvaluation, Evaluation,
            EvaluationResult, SpecRevision, snapshot_acceptance_criteria,
        },
        governance::EvidenceType,
        state_machine::FeatureState,
        work_package::WpState,
    },
    ports::{ExecutionRecordPort, StoragePort},
};
use agileplus_grpc::acceptance::AcceptanceCore;
use agileplus_proto::agileplus::v1::{
    DispatchCommandRequest, agile_plus_core_service_server::AgilePlusCoreService,
};
use agileplus_sqlite::SqliteStorageAdapter;
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;
use support::{Harness, REPO_ROOT, TestServer, command_request, rule, scope};
use tonic::{Code, Request};

const KEY: &str = "rpc-atomic-acceptance-fixture-only";
type Server = AcceptanceCore<TestServer, SqliteStorageAdapter>;

async fn setup() -> (Server, Arc<SqliteStorageAdapter>, i64) {
    let h = Harness::new().await;
    let f = h.seed_feature("alpha", FeatureState::Implementing).await;
    let wp = h.seed_wp(f.id, 1, WpState::Review).await;
    h.seed_evidence(wp.id, "FR-1", EvidenceType::TestResult)
        .await;
    h.seed_contract(
        f.id,
        1,
        vec![rule("Implementing -> Validated", &["FR-1:test_result"])],
    )
    .await;
    let now = Utc::now();
    let spec = SpecRevision {
        id: "rpc-spec".into(),
        feature_id: f.id,
        content_hash: "sha256:rpc".into(),
        parent_revision_id: None,
        accepted_at: now,
        authority: "owner".into(),
    };
    h.storage.create_spec_revision(&spec).await.unwrap();
    let assignment = Assignment {
        id: "rpc-assignment".into(),
        wp_id: wp.id,
        spec_revision_id: spec.id,
        created_at: now,
        supersedes_assignment_id: None,
        status: AssignmentStatus::Active,
    };
    let criteria = snapshot_acceptance_criteria("acceptance criteria met");
    h.storage
        .create_assignment_with_criteria(&assignment, &criteria)
        .await
        .unwrap();
    let candidate = format!("git:{:040x}", 1);
    let attempt = Attempt {
        id: "rpc-attempt".into(),
        assignment_id: assignment.id.clone(),
        worker_id: "worker".into(),
        backend: "fixture".into(),
        job_id: None,
        worktree_path: None,
        base_candidate_ref: None,
        result_candidate_ref: Some(candidate.clone()),
        status: AttemptStatus::Completed,
        failure_class: None,
        started_at: now,
        ended_at: Some(now),
    };
    h.storage.create_attempt(&attempt).await.unwrap();
    let result = CriterionEvaluation {
        criterion_id: criteria[0].id.clone(),
        result: EvaluationResult::Satisfied,
        evidence_refs: vec!["artifact:rpc-proof".into()],
        rationale: None,
    };
    h.storage
        .create_evaluation_receipt(
            &Evaluation {
                id: "rpc-evaluation".into(),
                assignment_id: assignment.id,
                attempt_id: Some(attempt.id),
                candidate_ref: candidate,
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
    let credentials: Arc<dyn CredentialStore> = Arc::new(InMemoryCredentialStore::new());
    credentials
        .set("agileplus", keys::API_KEYS, &format_api_key_hash(KEY))
        .unwrap();
    let db = h.storage.clone();
    (
        AcceptanceCore::new(h.server, db.clone(), credentials, REPO_ROOT.into()),
        db,
        f.id,
    )
}
fn request(authenticated: bool, extra: &[(&str, &str)]) -> Request<DispatchCommandRequest> {
    let mut args = vec![
        ("request_id", "rpc:request:1"),
        ("expected_governance_version", "1"),
    ];
    args.extend_from_slice(extra);
    let mut request = Request::new(DispatchCommandRequest {
        command: Some(command_request("validate", "alpha", &args)),
        project_scope: scope(),
    });
    if authenticated {
        request
            .metadata_mut()
            .insert("authorization", format!("Bearer {KEY}").parse().unwrap());
    }
    request
}
#[tokio::test]
async fn grpc_commits_and_replays_canonical_acceptance() {
    let (server, db, id) = setup().await;
    let first = server
        .dispatch_command(request(true, &[]))
        .await
        .unwrap()
        .into_inner()
        .result
        .unwrap();
    assert!(first.success);
    let receipt: Value = serde_json::from_str(&first.outputs["acceptance_receipt"]).unwrap();
    assert_eq!(receipt["actor"], "grpc:api-key");
    assert_eq!(receipt["accepted_candidates"].as_array().unwrap().len(), 1);
    let replay = server
        .dispatch_command(request(true, &[]))
        .await
        .unwrap()
        .into_inner()
        .result
        .unwrap();
    assert_eq!(replay.outputs["replayed"], "true");
    assert_eq!(
        replay.outputs["acceptance_receipt"],
        first.outputs["acceptance_receipt"]
    );
    assert_eq!(
        StoragePort::get_feature_by_id(db.as_ref(), id)
            .await
            .unwrap()
            .unwrap()
            .state,
        FeatureState::Validated
    );
}
#[tokio::test]
async fn grpc_requires_auth_and_rejects_grading_overrides_without_mutation() {
    let (server, db, id) = setup().await;
    assert_eq!(
        server
            .dispatch_command(request(false, &[]))
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    assert_eq!(
        server
            .dispatch_command(request(true, &[("passed", "true")]))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        StoragePort::get_feature_by_id(db.as_ref(), id)
            .await
            .unwrap()
            .unwrap()
            .state,
        FeatureState::Implementing
    );
}
#[tokio::test]
async fn grpc_rejects_foreign_scope() {
    let (server, db, id) = setup().await;
    let mut req = request(true, &[]);
    req.get_mut()
        .project_scope
        .as_mut()
        .unwrap()
        .canonical_repo_root = "/foreign".into();
    assert_eq!(
        server.dispatch_command(req).await.unwrap_err().code(),
        Code::PermissionDenied
    );
    assert_eq!(
        StoragePort::get_feature_by_id(db.as_ref(), id)
            .await
            .unwrap()
            .unwrap()
            .state,
        FeatureState::Implementing
    );
}
#[tokio::test]
async fn grpc_storage_failure_rolls_back_and_redacts_details() {
    let (server, db, id) = setup().await;
    db.conn_for_bench().unwrap().execute_batch("CREATE TRIGGER refuse_event BEFORE INSERT ON events BEGIN SELECT RAISE(ABORT,'private detail'); END;").unwrap();
    let error = server
        .dispatch_command(request(true, &[]))
        .await
        .unwrap_err();
    assert_eq!(error.code(), Code::Internal);
    assert!(!error.message().contains("private detail"));
    assert_eq!(
        StoragePort::get_feature_by_id(db.as_ref(), id)
            .await
            .unwrap()
            .unwrap()
            .state,
        FeatureState::Implementing
    );
    let conn = db.conn_for_bench().unwrap();
    for table in ["audit_log", "events", "feature_acceptance_receipts"] {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "{table}");
    }
}
