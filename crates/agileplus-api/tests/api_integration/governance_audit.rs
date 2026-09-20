// SPDX-License-Identifier: MIT OR Apache-2.0
//! Governance-validation and audit-chain verification behaviour.
//!
//! `POST /api/v1/features/:slug/validate` walks each governance rule and checks
//! whether its required evidence exists. The existing suite only exercises a
//! contract with *zero* rules, so every rule-satisfaction branch was dead. These
//! tests drive rules that are vacuously satisfied, rules whose evidence lookup
//! comes back empty, and the two audit-chain failure modes (`hash mismatch`,
//! `chain break`) plus the empty-trail case.
//!
//! Traceability: WP15-T086

use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::governance::{
    Evidence, EvidenceType, GovernanceContract, GovernanceRule,
};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use chrono::{DateTime, Utc};

use crate::support::{MockStorage, TEST_API_KEY, setup_test_server_with_storage};

const KEY: &str = "X-API-Key";

fn seeded_feature() -> Feature {
    let now = Utc::now();
    Feature {
        id: 1,
        slug: "governed-feature".to_string(),
        friendly_name: "Governed Feature".to_string(),
        state: FeatureState::Implementing,
        spec_hash: [0u8; 32],
        target_branch: "main".to_string(),
        plane_issue_id: None,
        plane_state_id: None,
        labels: vec![],
        module_id: None,
        project_id: None,
        created_at_commit: None,
        last_modified_commit: None,
        created_at: now,
        updated_at: now,
    }
}

fn seeded_work_package() -> WorkPackage {
    let now = Utc::now();
    WorkPackage {
        id: 1,
        feature_id: 1,
        title: "WP01".to_string(),
        state: WpState::Doing,
        sequence: 1,
        file_scope: vec![],
        acceptance_criteria: "evidence attached".to_string(),
        agent_id: None,
        pr_url: None,
        pr_state: None,
        worktree_path: None,
        plane_sub_issue_id: None,
        base_commit: None,
        head_commit: None,
        created_at: now,
        updated_at: now,
    }
}

fn storage_with_feature() -> MockStorage {
    let storage = MockStorage::default();
    storage
        .features
        .lock()
        .expect("features lock poisoned")
        .push(seeded_feature());
    storage
}

/// Feature + work package + a two-rule contract: one rule with no evidence
/// requirement, one requiring evidence that the mock never returns.
fn seeded_partial_governance_storage() -> MockStorage {
    let storage = storage_with_feature();
    storage
        .work_packages
        .lock()
        .expect("work_packages lock poisoned")
        .push(seeded_work_package());
    storage
        .governance
        .lock()
        .expect("governance lock poisoned")
        .push(GovernanceContract {
            id: 1,
            feature_id: 1,
            version: 2,
            rules: vec![
                GovernanceRule {
                    transition: "specify".to_string(),
                    required_evidence: vec![],
                    policy_refs: vec![],
                },
                GovernanceRule {
                    transition: "validate".to_string(),
                    required_evidence: vec!["FR-999".to_string()],
                    policy_refs: vec![],
                },
            ],
            bound_at: Utc::now(),
        });
    storage
}

/// Contract whose rules all have empty `required_evidence`, i.e. every rule is
/// satisfied without consulting the evidence store.
fn seeded_vacuous_governance_storage() -> MockStorage {
    let storage = storage_with_feature();
    storage
        .governance
        .lock()
        .expect("governance lock poisoned")
        .push(GovernanceContract {
            id: 1,
            feature_id: 1,
            version: 3,
            rules: vec![
                GovernanceRule {
                    transition: "specify".to_string(),
                    required_evidence: vec![],
                    policy_refs: vec![],
                },
                GovernanceRule {
                    transition: "validate".to_string(),
                    required_evidence: vec![],
                    policy_refs: vec![7],
                },
            ],
            bound_at: Utc::now(),
        });
    storage
}

// ── Governance validation ────────────────────────────────────────────────────

#[tokio::test]
async fn validate_reports_unsatisfied_rule_as_non_compliant() {
    let server = setup_test_server_with_storage(seeded_partial_governance_storage()).await;
    let resp = server
        .post("/api/v1/features/governed-feature/validate")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["governance_version"], 2);
    assert_eq!(body["total_rules"], 2);
    assert_eq!(
        body["satisfied_rules"], 1,
        "the rule with no evidence requirement is satisfied; the one demanding \
         missing evidence is not, got: {body}"
    );
    assert_eq!(body["compliant"], false);
}

#[tokio::test]
async fn validate_rule_without_required_evidence_is_satisfied() {
    let server = setup_test_server_with_storage(seeded_vacuous_governance_storage()).await;
    let resp = server
        .post("/api/v1/features/governed-feature/validate")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["governance_version"], 3);
    assert_eq!(body["total_rules"], 2);
    assert_eq!(body["satisfied_rules"], 2);
    assert_eq!(body["compliant"], true);
    assert_eq!(body["feature_slug"], "governed-feature");
}

/// A feature that exists but has no contract is a different 404 from an unknown
/// feature: the handler must reach the contract lookup and report it.
#[tokio::test]
async fn governance_get_without_contract_is_404() {
    let server = setup_test_server_with_storage(storage_with_feature()).await;
    let resp = server
        .get("/api/v1/features/governed-feature/governance")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
    let body: serde_json::Value = resp.json();
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("No governance contract"),
        "got: {body}"
    );
}

#[tokio::test]
async fn validate_without_contract_is_404() {
    let server = setup_test_server_with_storage(storage_with_feature()).await;
    let resp = server
        .post("/api/v1/features/governed-feature/validate")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
    let body: serde_json::Value = resp.json();
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("No governance contract"),
        "got: {body}"
    );
}

// ── Audit chain verification ─────────────────────────────────────────────────

/// Two well-formed entries whose timestamps/actors differ, so a caller can
/// tamper with either the stored hash or the back-link.
fn chained_entries() -> Vec<AuditEntry> {
    let genesis = AuditEntry {
        id: 1,
        feature_id: 1,
        wp_id: None,
        timestamp: DateTime::from_timestamp(1_700_000_000, 0).expect("valid timestamp"),
        actor: "system".to_string(),
        transition: "created".to_string(),
        evidence_refs: vec![],
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    let genesis = AuditEntry {
        hash: hash_entry(&genesis),
        ..genesis
    };

    let second = AuditEntry {
        id: 2,
        feature_id: 1,
        wp_id: Some(1),
        timestamp: DateTime::from_timestamp(1_700_000_100, 0).expect("valid timestamp"),
        actor: "agent".to_string(),
        transition: "specified".to_string(),
        evidence_refs: vec![],
        prev_hash: genesis.hash,
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    let second = AuditEntry {
        hash: hash_entry(&second),
        ..second
    };

    vec![genesis, second]
}

async fn server_with_audit(audit: Vec<AuditEntry>) -> axum_test::TestServer {
    let storage = storage_with_feature();
    storage
        .audit
        .lock()
        .expect("audit lock poisoned")
        .extend(audit);
    setup_test_server_with_storage(storage).await
}

/// Control: the untampered chain built by [`chained_entries`] verifies.
#[tokio::test]
async fn audit_verify_accepts_intact_chain() {
    let server = server_with_audit(chained_entries()).await;
    let resp = server
        .post("/api/v1/features/governed-feature/audit/verify")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["chain_valid"], true);
    assert_eq!(body["entries_verified"], 2);
}

#[tokio::test]
async fn audit_verify_reports_hash_mismatch_for_tampered_entry() {
    let mut entries = chained_entries();
    entries[0].hash = [0xff; 32]; // stored hash no longer matches the content

    let server = server_with_audit(entries).await;
    let resp = server
        .post("/api/v1/features/governed-feature/audit/verify")
        .add_header(KEY, TEST_API_KEY)
        .await;
    // Verification failure is a successful verification *report*, not an HTTP error.
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["chain_valid"], false);
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("hash mismatch"),
        "got: {body}"
    );
}

#[tokio::test]
async fn audit_verify_reports_chain_break_for_missing_back_link() {
    let mut entries = chained_entries();
    // Content still hashes correctly, but the link to the previous entry is wrong.
    entries[1].prev_hash = [0xab; 32];
    entries[1].hash = hash_entry(&entries[1]);

    let server = server_with_audit(entries).await;
    let resp = server
        .post("/api/v1/features/governed-feature/audit/verify")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["chain_valid"], false);
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("chain break"),
        "got: {body}"
    );
}

/// A feature with no audit entries at all: the verify endpoint still answers 200
/// but must not claim the (nonexistent) chain is valid.
#[tokio::test]
async fn audit_verify_reports_empty_trail_as_invalid() {
    let empty: Vec<AuditEntry> = Vec::new();
    let server = server_with_audit(empty).await;
    let resp = server
        .post("/api/v1/features/governed-feature/audit/verify")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["chain_valid"], false);
    assert_eq!(body["feature_slug"], "governed-feature");
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("empty audit chain"),
        "got: {body}"
    );
}

#[tokio::test]
async fn audit_trail_for_feature_without_entries_is_empty() {
    let empty: Vec<AuditEntry> = Vec::new();
    let server = server_with_audit(empty).await;
    let resp = server
        .get("/api/v1/features/governed-feature/audit")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let entries: Vec<serde_json::Value> = resp.json();
    assert!(entries.is_empty(), "got: {entries:?}");
}

// ── Evidence lookup: the by-FR and by-work-package fallback paths ────────────
//
// `POST /features/:slug/validate` first asks the storage port for evidence by
// FR id, then falls back to a per-work-package lookup when that comes back
// empty. Only the "missing everywhere" case was exercised before; these tests
// drive both success branches: a match through the by-FR lookup, and a match on
// the evidence *type label* through the work-package fallback.

fn evidence(id: i64, fr_id: &str, evidence_type: EvidenceType) -> Evidence {
    Evidence {
        id,
        wp_id: 1,
        fr_id: fr_id.to_string(),
        evidence_type,
        artifact_path: "artifacts/evidence.txt".to_string(),
        metadata: None,
        created_at: Utc::now(),
    }
}

/// Feature + work package + a single-rule contract demanding `required`.
fn storage_requiring_evidence(required: &str) -> MockStorage {
    let storage = storage_with_feature();
    storage
        .work_packages
        .lock()
        .expect("work_packages lock poisoned")
        .push(seeded_work_package());
    storage
        .governance
        .lock()
        .expect("governance lock poisoned")
        .push(GovernanceContract {
            id: 1,
            feature_id: 1,
            version: 4,
            rules: vec![GovernanceRule {
                transition: "validate".to_string(),
                required_evidence: vec![required.to_string()],
                policy_refs: vec![],
            }],
            bound_at: Utc::now(),
        });
    storage
}

/// The evidence is addressed by FR id, so the first lookup finds it directly.
#[tokio::test]
async fn validate_finds_evidence_through_the_fr_lookup() {
    let storage = storage_requiring_evidence("FR-1");
    storage
        .evidence
        .lock()
        .expect("evidence lock poisoned")
        .push(evidence(1, "FR-1", EvidenceType::TestResult));

    let server = setup_test_server_with_storage(storage).await;
    let resp = server
        .post("/api/v1/features/governed-feature/validate")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["total_rules"], 1);
    assert_eq!(
        body["satisfied_rules"], 1,
        "evidence carrying the required FR id satisfies the rule, got: {body}"
    );
    assert_eq!(body["compliant"], true);
}

/// The requirement is an evidence *type label* (`lint_result`), which no FR id
/// matches: the evaluator must fall back to the work-package lookup and match
/// on `evidence_type`.
#[tokio::test]
async fn validate_falls_back_to_work_package_evidence_by_type_label() {
    let storage = storage_requiring_evidence("lint_result");
    storage
        .evidence
        .lock()
        .expect("evidence lock poisoned")
        // Deliberately not addressed by the required label as an FR id.
        .push(evidence(1, "FR-999", EvidenceType::LintResult));

    let server = setup_test_server_with_storage(storage).await;
    let resp = server
        .post("/api/v1/features/governed-feature/validate")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(
        body["satisfied_rules"], 1,
        "a work-package evidence artifact whose type matches the label satisfies \
         the rule even though no FR id matched, got: {body}"
    );
    assert_eq!(body["compliant"], true);
}

/// Evidence attached to a different work package must not satisfy the rule.
#[tokio::test]
async fn validate_ignores_evidence_from_unrelated_work_package() {
    let storage = storage_requiring_evidence("lint_result");
    storage
        .evidence
        .lock()
        .expect("evidence lock poisoned")
        .push(Evidence {
            wp_id: 99,
            ..evidence(1, "FR-999", EvidenceType::LintResult)
        });

    let server = setup_test_server_with_storage(storage).await;
    let resp = server
        .post("/api/v1/features/governed-feature/validate")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["satisfied_rules"], 0);
    assert_eq!(body["compliant"], false);
}
