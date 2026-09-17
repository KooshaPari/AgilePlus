//! Integration tests: audit chain, evidence, governance contracts/policies, metrics.

mod common;

use agileplus_domain::domain::{
    audit::{AuditEntry, EvidenceRef},
    governance::{
        Evidence, EvidenceType, GovernanceContract, GovernanceRule, PolicyCheck, PolicyDefinition,
        PolicyDomain, PolicyRule,
    },
    metric::Metric,
};
use agileplus_sqlite::{
    repository::{audit, evidence, governance, metrics},
    SqliteStorageAdapter,
};

fn adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().unwrap()
}

fn audit_entry(feature_id: i64, prev: [u8; 32], hash: [u8; 32], transition: &str) -> AuditEntry {
    AuditEntry {
        id: 0,
        feature_id,
        wp_id: None,
        timestamp: chrono::Utc::now(),
        actor: "tester".into(),
        transition: transition.into(),
        evidence_refs: vec![],
        prev_hash: prev,
        hash,
        event_id: None,
        archived_to: None,
    }
}

fn evidence_row(wp_id: i64, fr_id: &str, ty: EvidenceType) -> Evidence {
    Evidence {
        id: 0,
        wp_id,
        fr_id: fr_id.into(),
        evidence_type: ty,
        artifact_path: "target/report.txt".into(),
        metadata: None,
        created_at: chrono::Utc::now(),
    }
}

fn policy(domain: PolicyDomain, active: bool) -> PolicyRule {
    PolicyRule {
        id: 0,
        domain,
        rule: PolicyDefinition {
            description: "a rule".into(),
            check: PolicyCheck::ManualApproval,
        },
        active,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

// ---------------------------------------------------------------------------
// Audit log
// ---------------------------------------------------------------------------

#[test]
fn audit_first_entry_requires_zero_prev_hash() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    let entry = audit_entry(1, [0u8; 32], [1u8; 32], "created");
    assert!(audit::append_audit_entry(&conn, &entry).unwrap() > 0);
}

#[test]
fn audit_first_entry_with_nonzero_prev_hash_rejected() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    let entry = audit_entry(1, [9u8; 32], [1u8; 32], "created");
    let err = audit::append_audit_entry(&conn, &entry).unwrap_err();
    assert!(matches!(err, agileplus_domain::error::DomainError::Storage(_)));
}

#[test]
fn audit_chain_links_and_latest() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    audit::append_audit_entry(&conn, &audit_entry(1, [0u8; 32], [1u8; 32], "created")).unwrap();
    audit::append_audit_entry(&conn, &audit_entry(1, [1u8; 32], [2u8; 32], "planned")).unwrap();

    let trail = audit::get_audit_trail(&conn, 1).unwrap();
    assert_eq!(trail.len(), 2);
    assert_eq!(trail[0].transition, "created");
    assert_eq!(trail[0].prev_hash, [0u8; 32]);
    assert_eq!(trail[0].hash, [1u8; 32]);
    assert_eq!(trail[1].prev_hash, [1u8; 32], "second entry chains to first");

    let latest = audit::get_latest_audit_entry(&conn, 1).unwrap().unwrap();
    assert_eq!(latest.hash, [2u8; 32]);
    assert_eq!(latest.transition, "planned");
}

#[test]
fn audit_broken_chain_rejected() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    audit::append_audit_entry(&conn, &audit_entry(1, [0u8; 32], [1u8; 32], "created")).unwrap();
    // Wrong prev_hash -> must be rejected.
    let bad = audit_entry(1, [7u8; 32], [2u8; 32], "planned");
    assert!(audit::append_audit_entry(&conn, &bad).is_err());
    // Trail unchanged.
    assert_eq!(audit::get_audit_trail(&conn, 1).unwrap().len(), 1);
}

#[test]
fn audit_trail_empty_for_unknown_feature() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(audit::get_audit_trail(&conn, 42).unwrap().is_empty());
    assert!(audit::get_latest_audit_entry(&conn, 42).unwrap().is_none());
}

#[test]
fn audit_evidence_refs_roundtrip() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    let mut entry = audit_entry(1, [0u8; 32], [3u8; 32], "validated");
    entry.evidence_refs = vec![
        EvidenceRef {
            evidence_id: 10,
            fr_id: "FR-1".into(),
        },
        EvidenceRef {
            evidence_id: 11,
            fr_id: "FR-2".into(),
        },
    ];
    audit::append_audit_entry(&conn, &entry).unwrap();
    let latest = audit::get_latest_audit_entry(&conn, 1).unwrap().unwrap();
    assert_eq!(latest.evidence_refs.len(), 2);
    assert_eq!(latest.evidence_refs[0].evidence_id, 10);
    assert_eq!(latest.evidence_refs[1].fr_id, "FR-2");
}

#[test]
fn audit_wp_id_roundtrips() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    common::seed_work_package(&conn, 5, 1);
    let mut entry = audit_entry(1, [0u8; 32], [4u8; 32], "done");
    entry.wp_id = Some(5);
    audit::append_audit_entry(&conn, &entry).unwrap();
    let latest = audit::get_latest_audit_entry(&conn, 1).unwrap().unwrap();
    assert_eq!(latest.wp_id, Some(5));
}

#[test]
fn audit_actor_and_transition_roundtrip() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    let mut entry = audit_entry(1, [0u8; 32], [5u8; 32], "shipped");
    entry.actor = "agent-xyz".into();
    audit::append_audit_entry(&conn, &entry).unwrap();
    let latest = audit::get_latest_audit_entry(&conn, 1).unwrap().unwrap();
    assert_eq!(latest.actor, "agent-xyz");
    assert_eq!(latest.transition, "shipped");
}

#[test]
fn audit_fk_requires_existing_feature() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let entry = audit_entry(777, [0u8; 32], [1u8; 32], "created");
    assert!(audit::append_audit_entry(&conn, &entry).is_err());
}

// ---------------------------------------------------------------------------
// Evidence
// ---------------------------------------------------------------------------

#[test]
fn evidence_create_and_get_by_wp() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    common::seed_work_package(&conn, 1, 1);
    let id = evidence::create_evidence(&conn, &evidence_row(1, "FR-1", EvidenceType::TestResult))
        .unwrap();
    assert!(id > 0);

    let got = evidence::get_evidence_by_wp(&conn, 1).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].evidence_type, EvidenceType::TestResult);
    assert_eq!(got[0].fr_id, "FR-1");
    assert_eq!(got[0].artifact_path, "target/report.txt");
}

#[test]
fn evidence_get_by_fr_filters() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    common::seed_work_package(&conn, 1, 1);
    evidence::create_evidence(&conn, &evidence_row(1, "FR-A", EvidenceType::CiOutput)).unwrap();
    evidence::create_evidence(&conn, &evidence_row(1, "FR-B", EvidenceType::LintResult)).unwrap();

    let a_items = evidence::get_evidence_by_fr(&conn, "FR-A").unwrap();
    assert_eq!(a_items.len(), 1);
    assert_eq!(a_items[0].evidence_type, EvidenceType::CiOutput);
    assert!(evidence::get_evidence_by_fr(&conn, "FR-Z").unwrap().is_empty());
}

#[test]
fn evidence_all_types_roundtrip() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    common::seed_work_package(&conn, 1, 1);
    for ty in [
        EvidenceType::TestResult,
        EvidenceType::CiOutput,
        EvidenceType::ReviewApproval,
        EvidenceType::SecurityScan,
        EvidenceType::LintResult,
        EvidenceType::ManualAttestation,
    ] {
        evidence::create_evidence(&conn, &evidence_row(1, "FR-X", ty)).unwrap();
    }
    assert_eq!(evidence::get_evidence_by_wp(&conn, 1).unwrap().len(), 6);
}

#[test]
fn evidence_metadata_roundtrips() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    common::seed_work_package(&conn, 1, 1);
    let mut ev = evidence_row(1, "FR-M", EvidenceType::SecurityScan);
    ev.metadata = Some(serde_json::json!({"cve": "none", "count": 3}));
    evidence::create_evidence(&conn, &ev).unwrap();
    let got = evidence::get_evidence_by_wp(&conn, 1).unwrap();
    assert_eq!(got[0].metadata.as_ref().unwrap()["count"], 3);
}

#[test]
fn evidence_empty_for_unknown_wp() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(evidence::get_evidence_by_wp(&conn, 1234).unwrap().is_empty());
}

#[test]
fn evidence_invalid_type_is_rejected_by_check_constraint() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    common::seed_work_package(&conn, 1, 1);
    let err = conn.execute(
        "INSERT INTO evidence (wp_id, fr_id, evidence_type, artifact_path, metadata, created_at)
         VALUES (1, 'FR-Q', 'not_a_type', 'p', NULL, ?1)",
        rusqlite::params![chrono::Utc::now().to_rfc3339()],
    );
    assert!(err.is_err(), "unknown evidence_type must violate the CHECK constraint");
}

#[test]
fn evidence_valid_row_parses_through_wp_query() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    common::seed_work_package(&conn, 1, 1);
    conn.execute(
        "INSERT INTO evidence (wp_id, fr_id, evidence_type, artifact_path, metadata, created_at)
         VALUES (1, 'FR-Q', 'test_result', 'p', NULL, ?1)",
        rusqlite::params![chrono::Utc::now().to_rfc3339()],
    )
    .unwrap();
    assert_eq!(evidence::get_evidence_by_wp(&conn, 1).unwrap().len(), 1);
}

#[test]
fn evidence_fk_requires_existing_wp() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(evidence::create_evidence(&conn, &evidence_row(9999, "FR", EvidenceType::TestResult))
        .is_err());
}

// ---------------------------------------------------------------------------
// Governance contracts
// ---------------------------------------------------------------------------

fn contract(feature_id: i64, version: i32) -> GovernanceContract {
    GovernanceContract {
        id: 0,
        feature_id,
        version,
        rules: vec![GovernanceRule {
            transition: "ship".into(),
            required_evidence: vec!["test_result".into()],
            policy_refs: vec![1],
        }],
        bound_at: chrono::Utc::now(),
    }
}

#[test]
fn governance_contract_create_and_get() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    let id = governance::create_governance_contract(&conn, &contract(1, 1)).unwrap();
    assert!(id > 0);

    let got = governance::get_governance_contract(&conn, 1, 1).unwrap().unwrap();
    assert_eq!(got.version, 1);
    assert_eq!(got.rules.len(), 1);
    assert_eq!(got.rules[0].transition, "ship");
    assert_eq!(got.rules[0].policy_refs, vec![1]);
}

#[test]
fn governance_contract_get_latest_picks_highest_version() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    governance::create_governance_contract(&conn, &contract(1, 1)).unwrap();
    governance::create_governance_contract(&conn, &contract(1, 3)).unwrap();
    governance::create_governance_contract(&conn, &contract(1, 2)).unwrap();

    let latest = governance::get_latest_governance_contract(&conn, 1)
        .unwrap()
        .unwrap();
    assert_eq!(latest.version, 3);
}

#[test]
fn governance_contract_duplicate_version_rejected() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    governance::create_governance_contract(&conn, &contract(1, 1)).unwrap();
    assert!(governance::create_governance_contract(&conn, &contract(1, 1)).is_err());
}

#[test]
fn governance_contract_missing_is_none() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(governance::get_governance_contract(&conn, 1, 1).unwrap().is_none());
    assert!(governance::get_latest_governance_contract(&conn, 1).unwrap().is_none());
}

#[test]
fn governance_contract_fk_requires_feature() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(governance::create_governance_contract(&conn, &contract(555, 1)).is_err());
}

// ---------------------------------------------------------------------------
// Policy rules
// ---------------------------------------------------------------------------

#[test]
fn policy_create_and_list_active() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    governance::create_policy_rule(&conn, &policy(PolicyDomain::Security, true)).unwrap();
    governance::create_policy_rule(&conn, &policy(PolicyDomain::Quality, false)).unwrap();

    let active = governance::list_active_policies(&conn).unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].domain, PolicyDomain::Security);
    assert!(active[0].active);
}

#[test]
fn policy_all_domains_roundtrip() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    for domain in [
        PolicyDomain::Security,
        PolicyDomain::Quality,
        PolicyDomain::Compliance,
        PolicyDomain::Performance,
        PolicyDomain::Custom,
    ] {
        governance::create_policy_rule(&conn, &policy(domain, true)).unwrap();
    }
    let active = governance::list_active_policies(&conn).unwrap();
    assert_eq!(active.len(), 5);
}

#[test]
fn policy_list_active_empty() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(governance::list_active_policies(&conn).unwrap().is_empty());
}

#[test]
fn policy_check_variants_roundtrip() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let checks = vec![
        PolicyCheck::ManualApproval,
        PolicyCheck::Automated,
        PolicyCheck::EvidencePresent {
            evidence_type: EvidenceType::TestResult,
        },
        PolicyCheck::ThresholdMet {
            metric: "coverage".into(),
            min: 85.0,
        },
        PolicyCheck::Custom {
            script: "check.sh".into(),
        },
    ];
    for check in checks {
        let mut p = policy(PolicyDomain::Custom, true);
        p.rule.check = check;
        governance::create_policy_rule(&conn, &p).unwrap();
    }
    assert_eq!(governance::list_active_policies(&conn).unwrap().len(), 5);
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

fn metric(feature_id: Option<i64>, command: &str) -> Metric {
    Metric {
        id: 0,
        feature_id,
        command: command.into(),
        duration_ms: 1234,
        agent_runs: 2,
        review_cycles: 1,
        metadata: None,
        timestamp: chrono::Utc::now(),
    }
}

#[test]
fn metric_record_and_get_by_feature() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    let id = metrics::record_metric(&conn, &metric(Some(1), "cargo test")).unwrap();
    assert!(id > 0);

    let got = metrics::get_metrics_by_feature(&conn, 1).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].command, "cargo test");
    assert_eq!(got[0].duration_ms, 1234);
    assert_eq!(got[0].agent_runs, 2);
    assert_eq!(got[0].review_cycles, 1);
}

#[test]
fn metric_metadata_roundtrips() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    let mut m = metric(Some(1), "lint");
    m.metadata = Some(serde_json::json!({"warnings": 0}));
    metrics::record_metric(&conn, &m).unwrap();
    let got = metrics::get_metrics_by_feature(&conn, 1).unwrap();
    assert_eq!(got[0].metadata.as_ref().unwrap()["warnings"], 0);
}

#[test]
fn metric_with_null_feature_not_returned_for_feature_query() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    metrics::record_metric(&conn, &metric(None, "global")).unwrap();
    assert!(metrics::get_metrics_by_feature(&conn, 1).unwrap().is_empty());
}

#[test]
fn metric_ordering_by_timestamp() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    let mut first = metric(Some(1), "first");
    first.timestamp = chrono::Utc::now() - chrono::Duration::seconds(10);
    let mut second = metric(Some(1), "second");
    second.timestamp = chrono::Utc::now();
    metrics::record_metric(&conn, &second).unwrap();
    metrics::record_metric(&conn, &first).unwrap();

    let got = metrics::get_metrics_by_feature(&conn, 1).unwrap();
    assert_eq!(got[0].command, "first");
    assert_eq!(got[1].command, "second");
}

#[test]
fn metric_feature_scoped_query_isolates_features() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    common::seed_feature_valid(&conn, 1, "f1");
    common::seed_feature_valid(&conn, 2, "f2");
    metrics::record_metric(&conn, &metric(Some(1), "a")).unwrap();
    metrics::record_metric(&conn, &metric(Some(2), "b")).unwrap();
    let got = metrics::get_metrics_by_feature(&conn, 1).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].command, "a");
}
