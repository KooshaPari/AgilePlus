//! Integration tests for `agileplus validate` (commands/validate.rs).
//!
//! Exercises the full `run_validate` flow against an in-memory SQLite
//! storage adapter and a temp-dir git adapter: state enforcement,
//! evidence evaluation, policy evaluation, report formatting, and the
//! Implementing -> Validated transition with audit entry.

use agileplus_cli::commands::validate::{ValidateArgs, run_validate};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::governance::{
    GovernanceContract, GovernanceRule,
};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WorkPackage;
use agileplus_domain::ports::StoragePort;
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

fn args(feature: &str) -> ValidateArgs {
    ValidateArgs {
        feature: feature.to_string(),
        format: "markdown".to_string(),
        skip_policies: true,
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
        StoragePort::create_feature(&storage, &wrong_state_feature("planned-feat")).await.unwrap();
        let err = run_validate(args("planned-feat"), &storage, &vcs)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("Expected 'Implementing'"),
            "unexpected error: {err}"
        );
        // State must NOT have transitioned.
        let f = StoragePort::get_feature_by_slug(&storage, "planned-feat").await.unwrap().unwrap();
        assert_eq!(f.state, FeatureState::Planned);
    })
}
#[test]
fn validate_force_overrides_wrong_state_and_records_exception() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        let id = StoragePort::create_feature(&storage, &wrong_state_feature("forced-feat")).await.unwrap();
        StoragePort::create_governance_contract(&storage, &contract_for(id, vec![])).await
            .unwrap();
        let mut a = args("forced-feat");
        a.force = true;
        run_validate(a, &storage, &vcs).await.expect("force validates");
        let f = StoragePort::get_feature_by_slug(&storage, "forced-feat").await.unwrap().unwrap();
        assert_eq!(f.state, FeatureState::Validated);
        // Governance exception appended as audit entry.
        let trail = StoragePort::get_audit_trail(&storage, id).await.unwrap();
        assert!(
            trail.iter().any(|e| e.transition.contains("Implementing -> Validated")),
            "expected audit entry for transition"
        );
    })
}
#[test]
fn validate_fails_when_required_evidence_missing() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        let id = StoragePort::create_feature(&storage, &implementing_feature("missing-ev")).await.unwrap();
        StoragePort::create_governance_contract(&storage, &contract_for(id, vec!["FR-001:test_result".to_string()])).await
            .unwrap();
        let err = run_validate(args("missing-ev"), &storage, &vcs)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("Validation FAILED"),
            "unexpected error: {err}"
        );
        // Failed validation must NOT transition the feature.
        let f = StoragePort::get_feature_by_slug(&storage, "missing-ev").await.unwrap().unwrap();
        assert_eq!(f.state, FeatureState::Implementing);
    })
}
#[test]
fn validate_passes_with_evidence_and_transitions() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        // Seed feature with contract, WP, and evidence inline.
        let id = StoragePort::create_feature(&storage, &implementing_feature("happy-feat")).await.unwrap();
        StoragePort::create_governance_contract(&storage, &contract_for(id, vec!["FR-001:test_result".to_string()])).await.unwrap();
        let mut wp = WorkPackage::new(id, "WP one", 1, "works");
        wp.id = 0;
        let wp_id = StoragePort::create_work_package(&storage, &wp).await.unwrap();
        let ev = agileplus_domain::domain::governance::Evidence {
            id: 0,
            wp_id,
            fr_id: "FR-001".to_string(),
            evidence_type: agileplus_domain::domain::governance::EvidenceType::TestResult,
            artifact_path: "target/test.log".to_string(),
            metadata: None,
            created_at: chrono::Utc::now(),
        };
        StoragePort::create_evidence(&storage, &ev).await.unwrap();
        run_validate(args("happy-feat"), &storage, &vcs)
            .await
            .expect("validates with evidence");
        let f = StoragePort::get_feature_by_slug(&storage, "happy-feat").await.unwrap().unwrap();
        assert_eq!(f.state, FeatureState::Validated);
        // Audit entry appended with hash chain.
        let id = f.id;
        let trail = StoragePort::get_audit_trail(&storage, id).await.unwrap();
        assert!(!trail.is_empty());
        assert!(trail.iter().all(|e| e.hash != [0u8; 32]), "hash must be computed");
    })
}
#[test]
fn validate_json_format_writes_report_file() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = GitVcsAdapter::new(std::env::temp_dir());
        let id = StoragePort::create_feature(&storage, &implementing_feature("json-feat")).await.unwrap();
        StoragePort::create_governance_contract(&storage, &contract_for(id, vec![])).await
            .unwrap();
        let out = std::env::temp_dir().join(format!("agileplus-validate-json-{}.md", std::process::id()));
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
        StoragePort::create_feature(&storage, &implementing_feature("no-contract")).await.unwrap();
        let err = run_validate(args("no-contract"), &storage, &vcs)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("No governance contract"),
            "unexpected error: {err}"
        );
    })
}
