//! Integration tests for `agileplus validate` (commands/validate.rs).
//!
//! Exercises the full `run_validate` flow against an in-memory SQLite
//! storage adapter and a temp-dir git adapter: state enforcement,
//! evidence evaluation, policy evaluation, report formatting, and the
//! Implementing -> Validated transition with audit entry.

use agileplus_cli::commands::validate::{ValidateArgs, run_validate};
use agileplus_domain::domain::execution::{
    Assignment, AssignmentStatus, Attempt, AttemptStatus, CriterionEvaluation, Evaluation,
    EvaluationResult, SpecRevision, aggregate_evidence_refs, reduce_criterion_results,
    snapshot_acceptance_criteria,
};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::governance::{
    Evidence, EvidenceType, GovernanceContract, GovernanceRule,
};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use agileplus_domain::ports::{ExecutionRecordPort, StoragePort};
use agileplus_git::GitVcsAdapter;
use agileplus_sqlite::SqliteStorageAdapter;

fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    tokio_test::block_on(fut)
}

fn implementing_feature(slug: &str) -> Feature {
    Feature {
        id: 0,
        slug: slug.to_string(),
        friendly_name: "Validate Test Feature".to_string(),
        state: FeatureState::Implementing,
        spec_hash: [7u8; 32],
        target_branch: "main".to_string(),
        plane_issue_id: None,
        plane_state_id: None,
        labels: vec![],
        module_id: None,
        project_id: None,
        created_at_commit: None,
        last_modified_commit: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

fn wrong_state_feature(slug: &str) -> Feature {
    let mut f = implementing_feature(slug);
    f.state = FeatureState::Planned;
    f
}

fn contract_for(feature_id: i64, required_evidence: Vec<String>) -> GovernanceContract {
    GovernanceContract {
        id: 0,
        feature_id,
        version: 1,
        rules: vec![GovernanceRule {
            transition: "Implementing -> Validated".to_string(),
            required_evidence,
            policy_refs: vec![],
        }],
        bound_at: chrono::Utc::now(),
    }
}

async fn seed_governance_evidence(
    storage: &SqliteStorageAdapter,
    feature_id: i64,
    fr_id: &str,
) -> i64 {
    let mut wp = WorkPackage::new(feature_id, "WP one", 1, "works");
    wp.state = WpState::Review;
    let wp_id = StoragePort::create_work_package(storage, &wp)
        .await
        .expect("create WP");
    StoragePort::create_evidence(
        storage,
        &Evidence {
            id: 0,
            wp_id,
            fr_id: fr_id.to_string(),
            evidence_type: EvidenceType::TestResult,
            artifact_path: "target/test.log".into(),
            metadata: None,
            created_at: chrono::Utc::now(),
        },
    )
    .await
    .expect("create evidence");
    wp_id
}

async fn seed_exact_candidate_acceptance(
    storage: &SqliteStorageAdapter,
    feature_id: i64,
    wp_id: i64,
) {
    let t0 = chrono::Utc::now();
    let revision = SpecRevision {
        id: format!("spec:{feature_id}:accepted"),
        feature_id,
        content_hash: "sha256:accepted".into(),
        parent_revision_id: None,
        accepted_at: t0,
        authority: "test".into(),
    };
    ExecutionRecordPort::create_spec_revision(storage, &revision)
        .await
        .expect("spec revision");
    let assignment = Assignment {
        id: format!("assignment:{wp_id}:accepted"),
        wp_id,
        spec_revision_id: revision.id.clone(),
        created_at: t0,
        supersedes_assignment_id: None,
        status: AssignmentStatus::Active,
    };
    let criteria = snapshot_acceptance_criteria("works");
    ExecutionRecordPort::create_assignment_with_criteria(storage, &assignment, &criteria)
        .await
        .expect("assignment");
    let attempt = Attempt {
        id: format!("attempt:{wp_id}:accepted"),
        assignment_id: assignment.id.clone(),
        worker_id: "worker".into(),
        backend: "test".into(),
        job_id: Some("job-accepted".into()),
        worktree_path: Some("/tmp/accepted".into()),
        base_candidate_ref: Some("git:base".into()),
        result_candidate_ref: Some("git:candidate".into()),
        status: AttemptStatus::Completed,
        failure_class: None,
        started_at: t0,
        ended_at: Some(t0),
    };
    ExecutionRecordPort::create_attempt(storage, &attempt)
        .await
        .expect("attempt");
    let criterion_results = vec![CriterionEvaluation {
        criterion_id: criteria[0].id.clone(),
        result: EvaluationResult::Satisfied,
        evidence_refs: vec!["evidence:test".into()],
        rationale: Some("test fixture".into()),
    }];
    let result = reduce_criterion_results(&criteria, &criterion_results);
    ExecutionRecordPort::create_evaluation_receipt(
        storage,
        &Evaluation {
            id: format!("evaluation:{wp_id}:accepted"),
            assignment_id: assignment.id,
            attempt_id: Some(attempt.id),
            candidate_ref: "git:candidate".into(),
            evaluator_id: "independent-test-evaluator".into(),
            evaluator_version: "1".into(),
            result,
            evidence_refs: aggregate_evidence_refs(&criterion_results),
            started_at: t0,
            finished_at: t0,
        },
        &criterion_results,
    )
    .await
    .expect("evaluation");
}

fn args(feature: &str) -> ValidateArgs {
    ValidateArgs {
        feature: feature.to_string(),
        format: "markdown".to_string(),
        skip_policies: false,
        output: None,
        force: false,
    }
}

#[test]
fn validate_errors_for_unknown_feature() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        let err = run_validate(args("no-such-feature"), &storage, &vcs)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("not found"),
            "unexpected error: {err}"
        );
    })
}
#[test]
fn validate_rejects_wrong_state_without_force() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        StoragePort::create_feature(&storage, &wrong_state_feature("planned-feat"))
            .await
            .unwrap();
        let err = run_validate(args("planned-feat"), &storage, &vcs)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("Expected 'Implementing'"),
            "unexpected error: {err}"
        );
        // State must NOT have transitioned.
        let f = StoragePort::get_feature_by_slug(&storage, "planned-feat")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.state, FeatureState::Planned);
    })
}
#[test]
fn validate_force_is_diagnostic_only_and_does_not_promote_state() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        let id = StoragePort::create_feature(&storage, &wrong_state_feature("forced-feat"))
            .await
            .unwrap();
        StoragePort::create_governance_contract(
            &storage,
            &contract_for(id, vec!["FR-FORCE:test_result".to_string()]),
        )
        .await
        .unwrap();
        seed_governance_evidence(&storage, id, "FR-FORCE").await;
        let mut a = args("forced-feat");
        a.force = true;
        let err = run_validate(a, &storage, &vcs).await.unwrap_err();
        assert!(err.to_string().contains("diagnostic-only"), "got: {err}");
        let f = StoragePort::get_feature_by_slug(&storage, "forced-feat")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.state, FeatureState::Planned);
    })
}

#[test]
fn validate_skip_policies_is_diagnostic_only() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        let id = StoragePort::create_feature(&storage, &implementing_feature("skip-policies"))
            .await
            .unwrap();
        StoragePort::create_governance_contract(
            &storage,
            &contract_for(id, vec!["FR-SKIP:test_result".to_string()]),
        )
        .await
        .unwrap();
        seed_governance_evidence(&storage, id, "FR-SKIP").await;
        let mut a = args("skip-policies");
        a.skip_policies = true;
        let err = run_validate(a, &storage, &vcs).await.unwrap_err();
        assert!(err.to_string().contains("diagnostic-only"), "got: {err}");
        let f = StoragePort::get_feature_by_slug(&storage, "skip-policies")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.state, FeatureState::Implementing);
    })
}

#[test]
fn validate_fails_when_required_evidence_missing() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        let id = StoragePort::create_feature(&storage, &implementing_feature("missing-ev"))
            .await
            .unwrap();
        StoragePort::create_governance_contract(
            &storage,
            &contract_for(id, vec!["FR-001:test_result".to_string()]),
        )
        .await
        .unwrap();
        let err = run_validate(args("missing-ev"), &storage, &vcs)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("Validation FAILED"),
            "unexpected error: {err}"
        );
        // Failed validation must NOT transition the feature.
        let f = StoragePort::get_feature_by_slug(&storage, "missing-ev")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.state, FeatureState::Implementing);
    })
}
#[test]
fn validate_passes_with_evidence_and_transitions() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        // Seed feature with contract, WP, and evidence inline.
        let id = StoragePort::create_feature(&storage, &implementing_feature("happy-feat"))
            .await
            .unwrap();
        StoragePort::create_governance_contract(
            &storage,
            &contract_for(id, vec!["FR-001:test_result".to_string()]),
        )
        .await
        .unwrap();
        let wp_id = seed_governance_evidence(&storage, id, "FR-001").await;
        seed_exact_candidate_acceptance(&storage, id, wp_id).await;
        run_validate(args("happy-feat"), &storage, &vcs)
            .await
            .expect("validates with evidence");
        let f = StoragePort::get_feature_by_slug(&storage, "happy-feat")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.state, FeatureState::Validated);
        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            wp.state,
            WpState::Done,
            "WP may become Done only after governance and exact-candidate acceptance both pass"
        );
        // Audit entry appended with hash chain.
        let id = f.id;
        let trail = StoragePort::get_audit_trail(&storage, id).await.unwrap();
        assert!(!trail.is_empty());
        assert!(
            trail.iter().all(|e| e.hash != [0u8; 32]),
            "hash must be computed"
        );
        assert!(
            trail.iter().any(|entry| {
                entry.wp_id == Some(wp_id)
                    && entry.transition.contains("Review -> Done")
                    && entry.transition.contains("correctness + governance accepted")
            }),
            "terminal WP acceptance must have its own audit receipt: {trail:?}"
        );
    })
}
#[test]
fn validate_rejects_governance_green_without_exact_candidate_acceptance() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        let id = StoragePort::create_feature(&storage, &implementing_feature("governance-only"))
            .await
            .unwrap();
        StoragePort::create_governance_contract(
            &storage,
            &contract_for(id, vec!["FR-GREEN:test_result".to_string()]),
        )
        .await
        .unwrap();
        seed_governance_evidence(&storage, id, "FR-GREEN").await;

        let err = run_validate(args("governance-only"), &storage, &vcs)
            .await
            .expect_err("governance evidence alone must not validate");
        assert!(
            err.to_string().contains("exact-candidate work acceptance"),
            "unexpected error: {err}"
        );
        let feature = StoragePort::get_feature_by_id(&storage, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(feature.state, FeatureState::Implementing);
    })
}

#[test]
fn validate_rejects_legacy_satisfied_evaluation_without_criterion_receipt() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        let id = StoragePort::create_feature(&storage, &implementing_feature("legacy-grade"))
            .await
            .unwrap();
        StoragePort::create_governance_contract(
            &storage,
            &contract_for(id, vec!["FR-LEGACY:test_result".to_string()]),
        )
        .await
        .unwrap();
        let wp_id = seed_governance_evidence(&storage, id, "FR-LEGACY").await;

        let t0 = chrono::Utc::now();
        let revision = SpecRevision {
            id: "spec:legacy".into(),
            feature_id: id,
            content_hash: "sha256:legacy".into(),
            parent_revision_id: None,
            accepted_at: t0,
            authority: "legacy-test".into(),
        };
        ExecutionRecordPort::create_spec_revision(&storage, &revision)
            .await
            .unwrap();
        let assignment = Assignment {
            id: "assignment:legacy".into(),
            wp_id,
            spec_revision_id: revision.id,
            created_at: t0,
            supersedes_assignment_id: None,
            status: AssignmentStatus::Active,
        };
        // Compatibility API deliberately creates no criterion snapshot, matching
        // a pre-029 database row.
        ExecutionRecordPort::create_assignment(&storage, &assignment)
            .await
            .unwrap();
        let attempt = Attempt {
            id: "attempt:legacy".into(),
            assignment_id: assignment.id.clone(),
            worker_id: "worker".into(),
            backend: "legacy".into(),
            job_id: Some("job:legacy".into()),
            worktree_path: Some("/tmp/legacy".into()),
            base_candidate_ref: Some("git:base".into()),
            result_candidate_ref: Some("git:legacy".into()),
            status: AttemptStatus::Completed,
            failure_class: None,
            started_at: t0,
            ended_at: Some(t0),
        };
        ExecutionRecordPort::create_attempt(&storage, &attempt)
            .await
            .unwrap();

        {
            let conn = storage.conn_for_bench().unwrap();
            conn.execute(
                r#"INSERT INTO evaluations
                 (id,assignment_id,attempt_id,candidate_ref,evaluator_id,evaluator_version,result,evidence_refs,started_at,finished_at)
                 VALUES (?1,?2,?3,?4,?5,?6,'satisfied','["legacy:evidence"]',?7,?7)"#,
                rusqlite::params![
                    "evaluation:legacy",
                    assignment.id,
                    attempt.id,
                    "git:legacy",
                    "legacy-evaluator",
                    "0",
                    t0.to_rfc3339()
                ],
            )
            .unwrap();
        }

        let err = run_validate(args("legacy-grade"), &storage, &vcs)
            .await
            .expect_err("legacy naked Satisfied grade must not validate");
        assert!(
            err.to_string().contains("no frozen Assignment criteria"),
            "unexpected error: {err}"
        );

        let feature = StoragePort::get_feature_by_id(&storage, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(feature.state, FeatureState::Implementing);
        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wp.state, WpState::Review);
    })
}

#[test]
fn validate_rejects_satisfied_evaluation_when_attempt_candidate_differs() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let id = StoragePort::create_feature(&storage, &implementing_feature("candidate-mismatch"))
            .await
            .unwrap();
        StoragePort::create_governance_contract(
            &storage,
            &contract_for(id, vec!["FR-MISMATCH:test_result".to_string()]),
        )
        .await
        .unwrap();
        let wp_id = seed_governance_evidence(&storage, id, "FR-MISMATCH").await;

        let t0 = chrono::Utc::now();
        let revision = SpecRevision {
            id: "spec:mismatch".into(),
            feature_id: id,
            content_hash: "sha256:mismatch".into(),
            parent_revision_id: None,
            accepted_at: t0,
            authority: "test".into(),
        };
        ExecutionRecordPort::create_spec_revision(&storage, &revision)
            .await
            .unwrap();
        let assignment = Assignment {
            id: "assignment:mismatch".into(),
            wp_id,
            spec_revision_id: revision.id,
            created_at: t0,
            supersedes_assignment_id: None,
            status: AssignmentStatus::Active,
        };
        let criteria = snapshot_acceptance_criteria("works");
        ExecutionRecordPort::create_assignment_with_criteria(&storage, &assignment, &criteria)
            .await
            .unwrap();
        let attempt = Attempt {
            id: "attempt:mismatch".into(),
            assignment_id: assignment.id.clone(),
            worker_id: "worker".into(),
            backend: "test".into(),
            job_id: Some("job".into()),
            worktree_path: Some("/tmp/mismatch".into()),
            base_candidate_ref: Some("git:base".into()),
            result_candidate_ref: Some("git:candidate-a".into()),
            status: AttemptStatus::Completed,
            failure_class: None,
            started_at: t0,
            ended_at: Some(t0),
        };
        ExecutionRecordPort::create_attempt(&storage, &attempt)
            .await
            .unwrap();
        let criterion_results = vec![CriterionEvaluation {
            criterion_id: criteria[0].id.clone(),
            result: EvaluationResult::Satisfied,
            evidence_refs: vec!["evidence:test".into()],
            rationale: None,
        }];
        let err = ExecutionRecordPort::create_evaluation_receipt(
            &storage,
            &Evaluation {
                id: "evaluation:mismatch".into(),
                assignment_id: assignment.id,
                attempt_id: Some(attempt.id),
                candidate_ref: "git:candidate-b".into(),
                evaluator_id: "independent".into(),
                evaluator_version: "1".into(),
                result: EvaluationResult::Satisfied,
                evidence_refs: aggregate_evidence_refs(&criterion_results),
                started_at: t0,
                finished_at: t0,
            },
            &criterion_results,
        )
        .await
        .expect_err("mismatched candidate receipt must be rejected");
        assert!(
            err.to_string().contains("completed Attempt candidate"),
            "unexpected error: {err}"
        );

        let feature = StoragePort::get_feature_by_id(&storage, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(feature.state, FeatureState::Implementing);
        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wp.state, WpState::Review);
    })
}

#[test]
fn validate_is_atomic_across_work_packages_when_one_lacks_acceptance() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        let id = StoragePort::create_feature(&storage, &implementing_feature("partial-acceptance"))
            .await
            .unwrap();
        StoragePort::create_governance_contract(
            &storage,
            &contract_for(
                id,
                vec![
                    "FR-A:test_result".to_string(),
                    "FR-B:test_result".to_string(),
                ],
            ),
        )
        .await
        .unwrap();

        let wp_a = seed_governance_evidence(&storage, id, "FR-A").await;
        let wp_b = seed_governance_evidence(&storage, id, "FR-B").await;
        seed_exact_candidate_acceptance(&storage, id, wp_a).await;

        let err = run_validate(args("partial-acceptance"), &storage, &vcs)
            .await
            .expect_err("one accepted WP must not validate the feature");
        assert!(
            err.to_string().contains("exact-candidate work acceptance"),
            "unexpected error: {err}"
        );

        let first = StoragePort::get_work_package(&storage, wp_a)
            .await
            .unwrap()
            .unwrap();
        let second = StoragePort::get_work_package(&storage, wp_b)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            first.state,
            WpState::Review,
            "accepted sibling must not transition early when another WP blocks feature acceptance"
        );
        assert_eq!(second.state, WpState::Review);
        let feature = StoragePort::get_feature_by_id(&storage, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(feature.state, FeatureState::Implementing);
    })
}

#[test]
fn validate_json_format_writes_report_file() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        let id = StoragePort::create_feature(&storage, &implementing_feature("json-feat"))
            .await
            .unwrap();
        StoragePort::create_governance_contract(
            &storage,
            &contract_for(id, vec!["FR-JSON:test_result".to_string()]),
        )
        .await
        .unwrap();
        let wp_id = seed_governance_evidence(&storage, id, "FR-JSON").await;
        seed_exact_candidate_acceptance(&storage, id, wp_id).await;
        let out =
            std::env::temp_dir().join(format!("agileplus-validate-json-{}.md", std::process::id()));
        let mut a = args("json-feat");
        a.format = "json".to_string();
        a.output = Some(out.clone());
        run_validate(a, &storage, &vcs).await.expect("validates");
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.contains("overall_pass") || content.contains("feature_slug"));
        let _ = std::fs::remove_file(out);
    })
}
#[test]
fn validate_errors_when_no_governance_contract() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        StoragePort::create_feature(&storage, &implementing_feature("no-contract"))
            .await
            .unwrap();
        let err = run_validate(args("no-contract"), &storage, &vcs)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("No governance contract"),
            "unexpected error: {err}"
        );
    })
}
