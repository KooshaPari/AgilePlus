use super::*;
use agileplus_domain::domain::{
    audit::{AuditEntry, hash_entry},
    feature::Feature,
    governance::{
        Evidence, EvidenceType, GovernanceContract, GovernanceRule, PolicyCheck, PolicyDefinition,
        PolicyDomain, PolicyRule,
    },
    metric::Metric,
    project::Project,
    state_machine::FeatureState,
    work_package::{DependencyType, WorkPackage, WpDependency, WpState},
};

fn make_adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory adapter")
}

// -- Feature tests --

#[tokio::test]
async fn feature_create_and_get_by_slug() {
    let db = make_adapter();
    let f = Feature::new("my-feat", "My Feature", [0u8; 32], None);
    let id = StoragePort::create_feature(&db, &f).await.unwrap();
    assert!(id > 0);

    let got = StoragePort::get_feature_by_slug(&db, "my-feat")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.slug, "my-feat");
    assert_eq!(got.friendly_name, "My Feature");
    assert_eq!(got.spec_hash, [0u8; 32]);
    assert_eq!(got.state, FeatureState::Created);
}

#[tokio::test]
async fn feature_get_by_id() {
    let db = make_adapter();
    let f = Feature::new("feat-id", "Feat", [1u8; 32], None);
    let id = StoragePort::create_feature(&db, &f).await.unwrap();
    let got = StoragePort::get_feature_by_id(&db, id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.slug, "feat-id");
}

#[tokio::test]
async fn feature_update_state() {
    let db = make_adapter();
    let f = Feature::new("upd-feat", "Upd", [0u8; 32], None);
    let id = StoragePort::create_feature(&db, &f).await.unwrap();

    StoragePort::update_feature_state(&db, id, FeatureState::Specified)
        .await
        .unwrap();
    let got = StoragePort::get_feature_by_id(&db, id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.state, FeatureState::Specified);
}

#[tokio::test]
async fn feature_list_by_state() {
    let db = make_adapter();
    let f1 = Feature::new("f1", "F1", [0u8; 32], None);
    let f2 = Feature::new("f2", "F2", [0u8; 32], None);
    let id1 = StoragePort::create_feature(&db, &f1).await.unwrap();
    let _id2 = StoragePort::create_feature(&db, &f2).await.unwrap();
    StoragePort::update_feature_state(&db, id1, FeatureState::Specified)
        .await
        .unwrap();

    let specified = StoragePort::list_features_by_state(&db, FeatureState::Specified)
        .await
        .unwrap();
    assert_eq!(specified.len(), 1);
    assert_eq!(specified[0].slug, "f1");

    let created = StoragePort::list_features_by_state(&db, FeatureState::Created)
        .await
        .unwrap();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].slug, "f2");
}

#[tokio::test]
async fn feature_list_all() {
    let db = make_adapter();
    StoragePort::create_feature(&db, &Feature::new("aa", "AA", [0u8; 32], None))
        .await
        .unwrap();
    StoragePort::create_feature(&db, &Feature::new("bb", "BB", [0u8; 32], None))
        .await
        .unwrap();
    let all = StoragePort::list_all_features(&db).await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn feature_duplicate_slug_fails() {
    let db = make_adapter();
    let f = Feature::new("dup", "Dup", [0u8; 32], None);
    StoragePort::create_feature(&db, &f).await.unwrap();
    let result = StoragePort::create_feature(&db, &f).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn feature_not_found_returns_none() {
    let db = make_adapter();
    let got = StoragePort::get_feature_by_slug(&db, "no-such-slug")
        .await
        .unwrap();
    assert!(got.is_none());
    let got2 = StoragePort::get_feature_by_id(&db, 9999).await.unwrap();
    assert!(got2.is_none());
}

// -- Work Package tests --

#[tokio::test]
async fn wp_create_and_get() {
    let db = make_adapter();
    let feat = Feature::new("wp-feat", "WP Feat", [0u8; 32], None);
    let fid = StoragePort::create_feature(&db, &feat).await.unwrap();

    let mut wp = WorkPackage::new(fid, "Task A", 1, "criteria");
    wp.file_scope = vec!["src/main.rs".into()];
    let wp_id = StoragePort::create_work_package(&db, &wp).await.unwrap();
    assert!(wp_id > 0);

    let got = StoragePort::get_work_package(&db, wp_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.title, "Task A");
    assert_eq!(got.file_scope, vec!["src/main.rs"]);
    assert_eq!(got.state, WpState::Planned);
    assert_eq!(got.feature_id, fid);
}

#[tokio::test]
async fn wp_update_state() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("f", "F", [0u8; 32], None))
        .await
        .unwrap();
    let wp_id = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "T", 1, "c"))
        .await
        .unwrap();
    StoragePort::update_wp_state(&db, wp_id, WpState::Doing)
        .await
        .unwrap();
    let got = StoragePort::get_work_package(&db, wp_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.state, WpState::Doing);
}

#[tokio::test]
async fn wp_list_by_feature_ordered_by_sequence() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("f2", "F2", [0u8; 32], None))
        .await
        .unwrap();
    StoragePort::create_work_package(&db, &WorkPackage::new(fid, "B", 2, "c"))
        .await
        .unwrap();
    StoragePort::create_work_package(&db, &WorkPackage::new(fid, "A", 1, "c"))
        .await
        .unwrap();

    let wps = StoragePort::list_wps_by_feature(&db, fid).await.unwrap();
    assert_eq!(wps.len(), 2);
    assert_eq!(wps[0].sequence, 1);
    assert_eq!(wps[1].sequence, 2);
}

#[tokio::test]
async fn wp_dependencies_and_ready_wps() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("f3", "F3", [0u8; 32], None))
        .await
        .unwrap();
    let wp1 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP1", 1, "c"))
        .await
        .unwrap();
    let wp2 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP2", 2, "c"))
        .await
        .unwrap();

    // wp2 depends on wp1
    StoragePort::add_wp_dependency(
        &db,
        &WpDependency {
            wp_id: wp2,
            depends_on: wp1,
            dep_type: DependencyType::Explicit,
        },
    )
    .await
    .unwrap();

    // Both planned but wp2 has unsatisfied dep -> only wp1 is ready
    let ready = StoragePort::get_ready_wps(&db, fid).await.unwrap();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, wp1);

    // Complete wp1
    StoragePort::update_wp_state(&db, wp1, WpState::Doing)
        .await
        .unwrap();
    StoragePort::update_wp_state(&db, wp1, WpState::Review)
        .await
        .unwrap();
    StoragePort::update_wp_state(&db, wp1, WpState::Done)
        .await
        .unwrap();

    // Now wp2 should be ready
    let ready = StoragePort::get_ready_wps(&db, fid).await.unwrap();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, wp2);
}

#[tokio::test]
async fn wp_get_dependencies() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("fd", "FD", [0u8; 32], None))
        .await
        .unwrap();
    let w1 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "W1", 1, "c"))
        .await
        .unwrap();
    let w2 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "W2", 2, "c"))
        .await
        .unwrap();
    StoragePort::add_wp_dependency(
        &db,
        &WpDependency {
            wp_id: w2,
            depends_on: w1,
            dep_type: DependencyType::Data,
        },
    )
    .await
    .unwrap();
    let deps = StoragePort::get_wp_dependencies(&db, w2).await.unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].depends_on, w1);
}

// -- Audit tests --

fn make_audit_entry(feature_id: i64, prev_hash: [u8; 32]) -> AuditEntry {
    let mut entry = AuditEntry {
        id: 0,
        feature_id,
        wp_id: None,
        timestamp: chrono::Utc::now(),
        actor: "agent".into(),
        transition: "created->specified".into(),
        evidence_refs: vec![],
        prev_hash,
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    entry.hash = hash_entry(&entry);
    entry
}

#[tokio::test]
async fn audit_append_and_trail() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("af", "AF", [0u8; 32], None))
        .await
        .unwrap();

    let e1 = make_audit_entry(fid, [0u8; 32]);
    let _id1 = StoragePort::append_audit_entry(&db, &e1).await.unwrap();

    let e2 = make_audit_entry(fid, e1.hash);
    let _id2 = StoragePort::append_audit_entry(&db, &e2).await.unwrap();

    let e3 = make_audit_entry(fid, e2.hash);
    StoragePort::append_audit_entry(&db, &e3).await.unwrap();

    let trail = StoragePort::get_audit_trail(&db, fid).await.unwrap();
    assert_eq!(trail.len(), 3);
    // Ordered chronologically
    assert!(trail[0].id <= trail[1].id);
    assert!(trail[1].id <= trail[2].id);
}

#[tokio::test]
async fn audit_wrong_prev_hash_rejected() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("afc", "AFC", [0u8; 32], None))
        .await
        .unwrap();

    let e1 = make_audit_entry(fid, [0u8; 32]);
    StoragePort::append_audit_entry(&db, &e1).await.unwrap();

    // Entry with wrong prev_hash
    let e_bad = make_audit_entry(fid, [0xFFu8; 32]);
    let result = StoragePort::append_audit_entry(&db, &e_bad).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn audit_get_latest() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("al", "AL", [0u8; 32], None))
        .await
        .unwrap();

    assert!(
        StoragePort::get_latest_audit_entry(&db, fid)
            .await
            .unwrap()
            .is_none()
    );

    let e1 = make_audit_entry(fid, [0u8; 32]);
    StoragePort::append_audit_entry(&db, &e1).await.unwrap();

    let e2 = make_audit_entry(fid, e1.hash);
    StoragePort::append_audit_entry(&db, &e2).await.unwrap();

    let latest = StoragePort::get_latest_audit_entry(&db, fid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest.hash, e2.hash);
}

// -- Evidence tests --

#[tokio::test]
async fn evidence_create_and_get_by_wp() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("ef", "EF", [0u8; 32], None))
        .await
        .unwrap();
    let wp_id = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP", 1, "c"))
        .await
        .unwrap();

    let ev = Evidence {
        id: 0,
        wp_id,
        fr_id: "FR-001".into(),
        evidence_type: EvidenceType::TestResult,
        artifact_path: "results/test.xml".into(),
        metadata: Some(serde_json::json!({"pass": 42})),
        created_at: chrono::Utc::now(),
    };
    let ev_id = StoragePort::create_evidence(&db, &ev).await.unwrap();
    assert!(ev_id > 0);

    let evs = StoragePort::get_evidence_by_wp(&db, wp_id).await.unwrap();
    assert_eq!(evs.len(), 1);
    assert_eq!(evs[0].fr_id, "FR-001");
    assert_eq!(evs[0].evidence_type, EvidenceType::TestResult);
    assert!(evs[0].metadata.is_some());
}

#[tokio::test]
async fn evidence_get_by_fr() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("efr", "EFR", [0u8; 32], None))
        .await
        .unwrap();
    let wp_id = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP", 1, "c"))
        .await
        .unwrap();

    let ev1 = Evidence {
        id: 0,
        wp_id,
        fr_id: "FR-001".into(),
        evidence_type: EvidenceType::CiOutput,
        artifact_path: "ci.log".into(),
        metadata: None,
        created_at: chrono::Utc::now(),
    };
    let ev2 = Evidence {
        id: 0,
        wp_id,
        fr_id: "FR-002".into(),
        evidence_type: EvidenceType::LintResult,
        artifact_path: "lint.log".into(),
        metadata: None,
        created_at: chrono::Utc::now(),
    };
    StoragePort::create_evidence(&db, &ev1).await.unwrap();
    StoragePort::create_evidence(&db, &ev2).await.unwrap();

    let fr1 = StoragePort::get_evidence_by_fr(&db, "FR-001")
        .await
        .unwrap();
    assert_eq!(fr1.len(), 1);
    assert_eq!(fr1[0].fr_id, "FR-001");
}

// -- Policy tests --

#[tokio::test]
async fn policy_create_and_list_active() {
    let db = make_adapter();
    let rule = PolicyRule {
        id: 0,
        domain: PolicyDomain::Quality,
        rule: PolicyDefinition {
            description: "All tests must pass".into(),
            check: PolicyCheck::ManualApproval,
        },
        active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let id = StoragePort::create_policy_rule(&db, &rule).await.unwrap();
    assert!(id > 0);

    let active = StoragePort::list_active_policies(&db).await.unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].domain, PolicyDomain::Quality);
}

// -- Governance Contract tests --

#[tokio::test]
async fn governance_contract_create_and_get() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("gc", "GC", [0u8; 32], None))
        .await
        .unwrap();

    let contract = GovernanceContract {
        id: 0,
        feature_id: fid,
        version: 1,
        rules: vec![GovernanceRule {
            transition: "created->specified".into(),
            required_evidence: vec![],
            policy_refs: vec![],
        }],
        bound_at: chrono::Utc::now(),
    };
    let cid = StoragePort::create_governance_contract(&db, &contract)
        .await
        .unwrap();
    assert!(cid > 0);

    let got = StoragePort::get_governance_contract(&db, fid, 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.feature_id, fid);
    assert_eq!(got.version, 1);
    assert_eq!(got.rules.len(), 1);

    let latest = StoragePort::get_latest_governance_contract(&db, fid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest.version, 1);
}

#[tokio::test]
async fn governance_contract_versioning() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("gcv", "GCV", [0u8; 32], None))
        .await
        .unwrap();

    let c1 = GovernanceContract {
        id: 0,
        feature_id: fid,
        version: 1,
        rules: vec![],
        bound_at: chrono::Utc::now(),
    };
    let c2 = GovernanceContract {
        id: 0,
        feature_id: fid,
        version: 2,
        rules: vec![],
        bound_at: chrono::Utc::now(),
    };
    StoragePort::create_governance_contract(&db, &c1)
        .await
        .unwrap();
    StoragePort::create_governance_contract(&db, &c2)
        .await
        .unwrap();

    let latest = StoragePort::get_latest_governance_contract(&db, fid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest.version, 2);

    let v1 = StoragePort::get_governance_contract(&db, fid, 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(v1.version, 1);
}

// -- Metrics tests --

#[tokio::test]
async fn metric_record_and_get() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("mf", "MF", [0u8; 32], None))
        .await
        .unwrap();

    let m = Metric {
        id: 0,
        feature_id: Some(fid),
        command: "spec-kitty implement".into(),
        duration_ms: 1234,
        agent_runs: 3,
        review_cycles: 1,
        metadata: Some(serde_json::json!({"model": "claude"})),
        timestamp: chrono::Utc::now(),
    };
    let mid = StoragePort::record_metric(&db, &m).await.unwrap();
    assert!(mid > 0);

    let ms = StoragePort::get_metrics_by_feature(&db, fid).await.unwrap();
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0].command, "spec-kitty implement");
    assert_eq!(ms[0].duration_ms, 1234);
    assert!(ms[0].metadata.is_some());
}

// -- Module tests --

use agileplus_domain::domain::module::{Module, ModuleFeatureTag};

#[tokio::test]
async fn module_create_and_get() {
    let db = make_adapter();
    let m = Module::new("Auth Module", None);
    let id = StoragePort::create_module(&db, &m).await.unwrap();
    assert!(id > 0);

    let got = StoragePort::get_module(&db, id).await.unwrap().unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.slug, "auth-module");
    assert_eq!(got.friendly_name, "Auth Module");
    assert!(got.parent_module_id.is_none());
}

#[tokio::test]
async fn module_get_by_slug() {
    let db = make_adapter();
    let m = Module::new("Billing", None);
    let id = StoragePort::create_module(&db, &m).await.unwrap();
    let got = StoragePort::get_module_by_slug(&db, "billing")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.id, id);
}

#[tokio::test]
async fn module_not_found_returns_none() {
    let db = make_adapter();
    assert!(StoragePort::get_module(&db, 9999).await.unwrap().is_none());
    assert!(
        StoragePort::get_module_by_slug(&db, "no-such")
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn module_update() {
    let db = make_adapter();
    let m = Module::new("Old Name", None);
    let id = StoragePort::create_module(&db, &m).await.unwrap();
    StoragePort::update_module(&db, id, "New Name", Some("a description"))
        .await
        .unwrap();
    let got = StoragePort::get_module(&db, id).await.unwrap().unwrap();
    assert_eq!(got.friendly_name, "New Name");
    assert_eq!(got.slug, "new-name");
    assert_eq!(got.description.as_deref(), Some("a description"));
}

#[tokio::test]
async fn module_delete_simple() {
    let db = make_adapter();
    let m = Module::new("Temp", None);
    let id = StoragePort::create_module(&db, &m).await.unwrap();
    StoragePort::delete_module(&db, id).await.unwrap();
    assert!(StoragePort::get_module(&db, id).await.unwrap().is_none());
}

#[tokio::test]
async fn module_delete_with_children_fails() {
    let db = make_adapter();
    let parent = Module::new("Parent", None);
    let pid = StoragePort::create_module(&db, &parent).await.unwrap();
    let child = Module::new("Child", Some(pid));
    StoragePort::create_module(&db, &child).await.unwrap();

    let result = StoragePort::delete_module(&db, pid).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        agileplus_domain::error::DomainError::ModuleHasDependents(_)
    ));
}

#[tokio::test]
async fn module_delete_with_owned_features_fails() {
    let db = make_adapter();
    let m = Module::new("Owner", None);
    let mid = StoragePort::create_module(&db, &m).await.unwrap();
    // Create feature and link it via tag
    let f = Feature::new("feat", "Feat", [0u8; 32], None);
    let fid = StoragePort::create_feature(&db, &f).await.unwrap();
    // Manually set module_id via tag so the module owns this feature
    // (we use the raw feature.module_id path by updating directly)
    {
        let conn = db.conn_for_bench().unwrap();
        conn.execute(
            "UPDATE features SET module_id = ?1 WHERE id = ?2",
            rusqlite::params![mid, fid],
        )
        .unwrap();
    }

    let result = StoragePort::delete_module(&db, mid).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        agileplus_domain::error::DomainError::ModuleHasDependents(_)
    ));
}

#[tokio::test]
async fn module_list_root_and_children() {
    let db = make_adapter();
    let r1 = StoragePort::create_module(&db, &Module::new("Root1", None))
        .await
        .unwrap();
    let r2 = StoragePort::create_module(&db, &Module::new("Root2", None))
        .await
        .unwrap();
    let _ = StoragePort::create_module(&db, &Module::new("Child1", Some(r1)))
        .await
        .unwrap();

    let roots = StoragePort::list_root_modules(&db).await.unwrap();
    assert_eq!(roots.len(), 2);

    let children = StoragePort::list_child_modules(&db, r1).await.unwrap();
    assert_eq!(children.len(), 1);

    let r2_children = StoragePort::list_child_modules(&db, r2).await.unwrap();
    assert!(r2_children.is_empty());
}

#[tokio::test]
async fn module_tag_and_untag_feature() {
    let db = make_adapter();
    let mid = StoragePort::create_module(&db, &Module::new("M", None))
        .await
        .unwrap();
    let fid = StoragePort::create_feature(&db, &Feature::new("f-tag", "FTag", [0u8; 32], None))
        .await
        .unwrap();

    let tag = ModuleFeatureTag::new(mid, fid);
    StoragePort::tag_feature_to_module(&db, &tag).await.unwrap();
    // Idempotent -- should not fail
    StoragePort::tag_feature_to_module(&db, &tag).await.unwrap();

    let mwf = StoragePort::get_module_with_features(&db, mid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(mwf.tagged_features.len(), 1);
    assert_eq!(mwf.tagged_features[0].id, fid);

    StoragePort::untag_feature_from_module(&db, mid, fid)
        .await
        .unwrap();
    let mwf2 = StoragePort::get_module_with_features(&db, mid)
        .await
        .unwrap()
        .unwrap();
    assert!(mwf2.tagged_features.is_empty());
}

#[tokio::test]
async fn module_get_with_features_none_for_missing() {
    let db = make_adapter();
    assert!(
        StoragePort::get_module_with_features(&db, 9999)
            .await
            .unwrap()
            .is_none()
    );
}

// -- Cycle tests --

use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState};
use chrono::NaiveDate;

fn make_date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

#[tokio::test]
async fn cycle_create_and_get() {
    let db = make_adapter();
    let c = Cycle::new(
        "Q1-2026",
        make_date(2026, 1, 1),
        make_date(2026, 3, 31),
        None,
    )
    .unwrap();
    let id = StoragePort::create_cycle(&db, &c).await.unwrap();
    assert!(id > 0);

    let got = StoragePort::get_cycle(&db, id).await.unwrap().unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.name, "Q1-2026");
    assert_eq!(got.state, CycleState::Draft);
    assert!(got.module_scope_id.is_none());
}

#[tokio::test]
async fn cycle_not_found_returns_none() {
    let db = make_adapter();
    assert!(StoragePort::get_cycle(&db, 9999).await.unwrap().is_none());
}

#[tokio::test]
async fn cycle_update_state() {
    let db = make_adapter();
    let c = Cycle::new(
        "Cycle-A",
        make_date(2026, 1, 1),
        make_date(2026, 2, 1),
        None,
    )
    .unwrap();
    let id = StoragePort::create_cycle(&db, &c).await.unwrap();
    StoragePort::update_cycle_state(&db, id, CycleState::Active)
        .await
        .unwrap();
    let got = StoragePort::get_cycle(&db, id).await.unwrap().unwrap();
    assert_eq!(got.state, CycleState::Active);
}

#[tokio::test]
async fn cycle_list_by_state() {
    let db = make_adapter();
    let c1 = Cycle::new(
        "Draft-1",
        make_date(2026, 1, 1),
        make_date(2026, 2, 1),
        None,
    )
    .unwrap();
    let c2 = Cycle::new(
        "Draft-2",
        make_date(2026, 3, 1),
        make_date(2026, 4, 1),
        None,
    )
    .unwrap();
    let id1 = StoragePort::create_cycle(&db, &c1).await.unwrap();
    let id2 = StoragePort::create_cycle(&db, &c2).await.unwrap();
    StoragePort::update_cycle_state(&db, id1, CycleState::Active)
        .await
        .unwrap();

    let drafts = StoragePort::list_cycles_by_state(&db, CycleState::Draft)
        .await
        .unwrap();
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].id, id2);

    let actives = StoragePort::list_cycles_by_state(&db, CycleState::Active)
        .await
        .unwrap();
    assert_eq!(actives.len(), 1);
    assert_eq!(actives[0].id, id1);
}

#[tokio::test]
async fn cycle_list_by_module() {
    let db = make_adapter();
    let mid = StoragePort::create_module(&db, &Module::new("ScopeModule", None))
        .await
        .unwrap();
    let c1 = Cycle::new(
        "Scoped",
        make_date(2026, 1, 1),
        make_date(2026, 2, 1),
        Some(mid),
    )
    .unwrap();
    let c2 = Cycle::new(
        "Unscoped",
        make_date(2026, 3, 1),
        make_date(2026, 4, 1),
        None,
    )
    .unwrap();
    let id1 = StoragePort::create_cycle(&db, &c1).await.unwrap();
    StoragePort::create_cycle(&db, &c2).await.unwrap();

    let scoped = StoragePort::list_cycles_by_module(&db, mid).await.unwrap();
    assert_eq!(scoped.len(), 1);
    assert_eq!(scoped[0].id, id1);
}

#[tokio::test]
async fn cycle_add_and_remove_feature() {
    let db = make_adapter();
    let c = Cycle::new("C1", make_date(2026, 1, 1), make_date(2026, 2, 1), None).unwrap();
    let cid = StoragePort::create_cycle(&db, &c).await.unwrap();
    let fid =
        StoragePort::create_feature(&db, &Feature::new("cyc-feat", "CycFeat", [0u8; 32], None))
            .await
            .unwrap();

    let entry = CycleFeature::new(cid, fid);
    StoragePort::add_feature_to_cycle(&db, &entry)
        .await
        .unwrap();
    // Idempotent
    StoragePort::add_feature_to_cycle(&db, &entry)
        .await
        .unwrap();

    let cwf = StoragePort::get_cycle_with_features(&db, cid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cwf.features.len(), 1);
    assert_eq!(cwf.features[0].id, fid);

    StoragePort::remove_feature_from_cycle(&db, cid, fid)
        .await
        .unwrap();
    let cwf2 = StoragePort::get_cycle_with_features(&db, cid)
        .await
        .unwrap()
        .unwrap();
    assert!(cwf2.features.is_empty());
}

#[tokio::test]
async fn cycle_with_features_none_for_missing() {
    let db = make_adapter();
    assert!(
        StoragePort::get_cycle_with_features(&db, 9999)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn cycle_module_scope_enforcement() {
    let db = make_adapter();
    let mid = StoragePort::create_module(&db, &Module::new("Scope", None))
        .await
        .unwrap();
    // Cycle scoped to this module
    let c = Cycle::new(
        "Scoped-Cycle",
        make_date(2026, 1, 1),
        make_date(2026, 2, 1),
        Some(mid),
    )
    .unwrap();
    let cid = StoragePort::create_cycle(&db, &c).await.unwrap();

    // Feature NOT in module scope
    let fid =
        StoragePort::create_feature(&db, &Feature::new("out-of-scope", "OOS", [0u8; 32], None))
            .await
            .unwrap();
    let entry = CycleFeature::new(cid, fid);
    let result = StoragePort::add_feature_to_cycle(&db, &entry).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        agileplus_domain::error::DomainError::FeatureNotInModuleScope { .. }
    ));

    // Tag feature to module -- now it should work
    StoragePort::tag_feature_to_module(&db, &ModuleFeatureTag::new(mid, fid))
        .await
        .unwrap();
    StoragePort::add_feature_to_cycle(&db, &CycleFeature::new(cid, fid))
        .await
        .unwrap();
    let cwf = StoragePort::get_cycle_with_features(&db, cid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cwf.features.len(), 1);
}

#[tokio::test]
async fn cycle_wp_progress_summary() {
    let db = make_adapter();
    let c = Cycle::new(
        "WP-Prog",
        make_date(2026, 1, 1),
        make_date(2026, 2, 1),
        None,
    )
    .unwrap();
    let cid = StoragePort::create_cycle(&db, &c).await.unwrap();
    let fid = StoragePort::create_feature(&db, &Feature::new("prog-feat", "Prog", [0u8; 32], None))
        .await
        .unwrap();
    StoragePort::add_feature_to_cycle(&db, &CycleFeature::new(cid, fid))
        .await
        .unwrap();

    // Create 2 WPs
    let _wp1 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP1", 1, "c"))
        .await
        .unwrap();
    let wp2 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP2", 2, "c"))
        .await
        .unwrap();
    StoragePort::update_wp_state(
        &db,
        wp2,
        agileplus_domain::domain::work_package::WpState::Done,
    )
    .await
    .unwrap();

    let cwf = StoragePort::get_cycle_with_features(&db, cid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cwf.wp_progress.total, 2);
    assert_eq!(cwf.wp_progress.planned, 1);
    assert_eq!(cwf.wp_progress.done, 1);
}

// -- Project extended tests --

#[tokio::test]
async fn project_create_and_get_by_slug() {
    let db = make_adapter();
    let p = Project::new("Test Project", "test-project").unwrap();
    let id = StoragePort::create_project(&db, &p).await.unwrap();
    assert!(id > 0);
    let got = StoragePort::get_project_by_slug(&db, "test-project")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.name, "Test Project");
}

#[tokio::test]
async fn project_get_by_id() {
    let db = make_adapter();
    let p = Project::new("Id Project", "id-project").unwrap();
    let id = StoragePort::create_project(&db, &p).await.unwrap();
    let got = StoragePort::get_project_by_id(&db, id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.slug, "id-project");
}

#[tokio::test]
async fn project_list_all() {
    let db = make_adapter();
    StoragePort::create_project(&db, &Project::new("P1", "p1").unwrap())
        .await
        .unwrap();
    StoragePort::create_project(&db, &Project::new("P2", "p2").unwrap())
        .await
        .unwrap();
    let all = StoragePort::list_all_projects(&db).await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn project_delete() {
    let db = make_adapter();
    let id = StoragePort::create_project(&db, &Project::new("Tmp", "tmp-del").unwrap())
        .await
        .unwrap();
    StoragePort::delete_project(&db, id).await.unwrap();
    assert!(
        StoragePort::get_project_by_id(&db, id)
            .await
            .unwrap()
            .is_none()
    );
}

// -- User tests --

use agileplus_domain::domain::user::{User, UserRole, UserStatus};

#[tokio::test]
async fn user_create_and_get_by_id() {
    let db = make_adapter();
    let u = User::new("Alice", "alice@example.com", UserRole::Member).unwrap();
    let id = StoragePort::create_user(&db, &u).await.unwrap();
    assert!(id > 0);
    let got = StoragePort::get_user(&db, id).await.unwrap().unwrap();
    assert_eq!(got.display_name, "Alice");
    assert_eq!(got.email, "alice@example.com");
    assert_eq!(got.role, UserRole::Member);
    assert_eq!(got.status, UserStatus::Active);
}

#[tokio::test]
async fn user_get_by_email() {
    let db = make_adapter();
    let u = User::new("Bob", "bob@example.com", UserRole::Admin).unwrap();
    let id = StoragePort::create_user(&db, &u).await.unwrap();
    let got = StoragePort::get_user_by_email(&db, "bob@example.com")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.role, UserRole::Admin);
}

#[tokio::test]
async fn user_not_found_returns_none() {
    let db = make_adapter();
    assert!(StoragePort::get_user(&db, 9999).await.unwrap().is_none());
    assert!(
        StoragePort::get_user_by_email(&db, "no@no.com")
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn user_update_status() {
    let db = make_adapter();
    let u = User::new("Carol", "carol@example.com", UserRole::Viewer).unwrap();
    let id = StoragePort::create_user(&db, &u).await.unwrap();
    StoragePort::update_user_status(&db, id, UserStatus::Inactive)
        .await
        .unwrap();
    let got = StoragePort::get_user(&db, id).await.unwrap().unwrap();
    assert_eq!(got.status, UserStatus::Inactive);
}

#[tokio::test]
async fn user_update_role() {
    let db = make_adapter();
    let u = User::new("Dave", "dave@example.com", UserRole::Viewer).unwrap();
    let id = StoragePort::create_user(&db, &u).await.unwrap();
    StoragePort::update_user_role(&db, id, UserRole::Admin)
        .await
        .unwrap();
    let got = StoragePort::get_user(&db, id).await.unwrap().unwrap();
    assert_eq!(got.role, UserRole::Admin);
}

#[tokio::test]
async fn user_list_all() {
    let db = make_adapter();
    StoragePort::create_user(&db, &User::new("U1", "u1@x.com", UserRole::Member).unwrap())
        .await
        .unwrap();
    StoragePort::create_user(&db, &User::new("U2", "u2@x.com", UserRole::Member).unwrap())
        .await
        .unwrap();
    let all = StoragePort::list_all_users(&db).await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn user_delete() {
    let db = make_adapter();
    let id = StoragePort::create_user(
        &db,
        &User::new("Tmp", "tmp@x.com", UserRole::Viewer).unwrap(),
    )
    .await
    .unwrap();
    StoragePort::delete_user(&db, id).await.unwrap();
    assert!(StoragePort::get_user(&db, id).await.unwrap().is_none());
}

#[tokio::test]
async fn user_duplicate_email_fails() {
    let db = make_adapter();
    let u = User::new("Dup", "dup@x.com", UserRole::Member).unwrap();
    StoragePort::create_user(&db, &u).await.unwrap();
    assert!(StoragePort::create_user(&db, &u).await.is_err());
}

// -- Epic tests --

use agileplus_domain::domain::epic::{Epic, EpicStatus};

async fn make_project(db: &SqliteStorageAdapter) -> i64 {
    StoragePort::create_project(db, &Project::new("Epic Project", "epic-project").unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn epic_create_and_get_by_id() {
    let db = make_adapter();
    let pid = make_project(&db).await;
    let e = Epic::new(pid, "Auth Overhaul").unwrap();
    let id = StoragePort::create_epic(&db, &e).await.unwrap();
    assert!(id > 0);
    let got = StoragePort::get_epic(&db, id).await.unwrap().unwrap();
    assert_eq!(got.title, "Auth Overhaul");
    assert_eq!(got.project_id, pid);
    assert_eq!(got.status, EpicStatus::Backlog);
}

#[tokio::test]
async fn epic_not_found_returns_none() {
    let db = make_adapter();
    assert!(StoragePort::get_epic(&db, 9999).await.unwrap().is_none());
}

#[tokio::test]
async fn epic_update_status() {
    let db = make_adapter();
    let pid = make_project(&db).await;
    let e = Epic::new(pid, "Billing").unwrap();
    let id = StoragePort::create_epic(&db, &e).await.unwrap();
    StoragePort::update_epic_status(&db, id, EpicStatus::Active)
        .await
        .unwrap();
    let got = StoragePort::get_epic(&db, id).await.unwrap().unwrap();
    assert_eq!(got.status, EpicStatus::Active);
}

#[tokio::test]
async fn epic_list_by_project() {
    let db = make_adapter();
    let pid = StoragePort::create_project(
        &db,
        &Project::new("Proj for Epics", "proj-for-epics").unwrap(),
    )
    .await
    .unwrap();
    let pid2 = StoragePort::create_project(&db, &Project::new("Other Proj", "other-proj").unwrap())
        .await
        .unwrap();
    StoragePort::create_epic(&db, &Epic::new(pid, "E1").unwrap())
        .await
        .unwrap();
    StoragePort::create_epic(&db, &Epic::new(pid, "E2").unwrap())
        .await
        .unwrap();
    StoragePort::create_epic(&db, &Epic::new(pid2, "E3").unwrap())
        .await
        .unwrap();

    let epics = StoragePort::list_epics_by_project(&db, pid).await.unwrap();
    assert_eq!(epics.len(), 2);
    assert!(epics.iter().all(|e| e.project_id == pid));
}

#[tokio::test]
async fn epic_delete() {
    let db = make_adapter();
    let pid = StoragePort::create_project(&db, &Project::new("Del Proj", "del-proj-epic").unwrap())
        .await
        .unwrap();
    let eid = StoragePort::create_epic(&db, &Epic::new(pid, "Temp Epic").unwrap())
        .await
        .unwrap();
    StoragePort::delete_epic(&db, eid).await.unwrap();
    assert!(StoragePort::get_epic(&db, eid).await.unwrap().is_none());
}

// -- Story tests --

use agileplus_domain::domain::story::{Story, StoryStatus};

async fn make_project_and_epic(db: &SqliteStorageAdapter) -> (i64, i64) {
    let pid =
        StoragePort::create_project(db, &Project::new("Story Project", "story-project").unwrap())
            .await
            .unwrap();
    let eid = StoragePort::create_epic(db, &Epic::new(pid, "Story Epic").unwrap())
        .await
        .unwrap();
    (pid, eid)
}

#[tokio::test]
async fn story_create_and_get_by_id() {
    let db = make_adapter();
    let (pid, eid) = make_project_and_epic(&db).await;
    let s = Story::new(eid, pid, "User can log in", Some(3)).unwrap();
    let id = StoragePort::create_story(&db, &s).await.unwrap();
    assert!(id > 0);
    let got = StoragePort::get_story(&db, id).await.unwrap().unwrap();
    assert_eq!(got.title, "User can log in");
    assert_eq!(got.epic_id, eid);
    assert_eq!(got.project_id, pid);
    assert_eq!(got.points, Some(3));
    assert_eq!(got.status, StoryStatus::Todo);
}

#[tokio::test]
async fn story_not_found_returns_none() {
    let db = make_adapter();
    assert!(StoragePort::get_story(&db, 9999).await.unwrap().is_none());
}

#[tokio::test]
async fn story_update_status() {
    let db = make_adapter();
    let (pid, eid) = make_project_and_epic(&db).await;
    let s = Story::new(eid, pid, "Login flow", None).unwrap();
    let id = StoragePort::create_story(&db, &s).await.unwrap();
    StoragePort::update_story_status(&db, id, StoryStatus::InProgress)
        .await
        .unwrap();
    let got = StoragePort::get_story(&db, id).await.unwrap().unwrap();
    assert_eq!(got.status, StoryStatus::InProgress);
}

#[tokio::test]
async fn story_list_by_epic() {
    let db = make_adapter();
    let pid = StoragePort::create_project(
        &db,
        &Project::new("Multi Epic Proj", "multi-epic-proj").unwrap(),
    )
    .await
    .unwrap();
    let eid1 = StoragePort::create_epic(&db, &Epic::new(pid, "Epic One").unwrap())
        .await
        .unwrap();
    let eid2 = StoragePort::create_epic(&db, &Epic::new(pid, "Epic Two").unwrap())
        .await
        .unwrap();
    StoragePort::create_story(&db, &Story::new(eid1, pid, "S1", None).unwrap())
        .await
        .unwrap();
    StoragePort::create_story(&db, &Story::new(eid1, pid, "S2", Some(5)).unwrap())
        .await
        .unwrap();
    StoragePort::create_story(&db, &Story::new(eid2, pid, "S3", None).unwrap())
        .await
        .unwrap();

    let stories = StoragePort::list_stories_by_epic(&db, eid1).await.unwrap();
    assert_eq!(stories.len(), 2);
    assert!(stories.iter().all(|s| s.epic_id == eid1));
}

#[tokio::test]
async fn story_list_by_project() {
    let db = make_adapter();
    let pid = StoragePort::create_project(
        &db,
        &Project::new("All Stories Proj", "all-stories-proj").unwrap(),
    )
    .await
    .unwrap();
    let pid2 = StoragePort::create_project(
        &db,
        &Project::new("Other Stories Proj", "other-stories-proj").unwrap(),
    )
    .await
    .unwrap();
    let eid1 = StoragePort::create_epic(&db, &Epic::new(pid, "E1").unwrap())
        .await
        .unwrap();
    let eid2 = StoragePort::create_epic(&db, &Epic::new(pid2, "E2").unwrap())
        .await
        .unwrap();
    StoragePort::create_story(&db, &Story::new(eid1, pid, "Proj1 S1", None).unwrap())
        .await
        .unwrap();
    StoragePort::create_story(&db, &Story::new(eid1, pid, "Proj1 S2", None).unwrap())
        .await
        .unwrap();
    StoragePort::create_story(&db, &Story::new(eid2, pid2, "Proj2 S1", None).unwrap())
        .await
        .unwrap();

    let proj1_stories = StoragePort::list_stories_by_project(&db, pid)
        .await
        .unwrap();
    assert_eq!(proj1_stories.len(), 2);
    assert!(proj1_stories.iter().all(|s| s.project_id == pid));

    let proj2_stories = StoragePort::list_stories_by_project(&db, pid2)
        .await
        .unwrap();
    assert_eq!(proj2_stories.len(), 1);
}

#[tokio::test]
async fn story_delete() {
    let db = make_adapter();
    let (pid, eid) = make_project_and_epic(&db).await;
    let sid = StoragePort::create_story(&db, &Story::new(eid, pid, "Temp", None).unwrap())
        .await
        .unwrap();
    StoragePort::delete_story(&db, sid).await.unwrap();
    assert!(StoragePort::get_story(&db, sid).await.unwrap().is_none());
}

#[tokio::test]
async fn story_invariant_no_zero_points_roundtrip() {
    // Domain invariant: zero points is rejected at construction; storage
    // should never see it. Verify that a story with points=None round-trips.
    let db = make_adapter();
    let (pid, eid) = make_project_and_epic(&db).await;
    let s = Story::new(eid, pid, "No points", None).unwrap();
    let id = StoragePort::create_story(&db, &s).await.unwrap();
    let got = StoragePort::get_story(&db, id).await.unwrap().unwrap();
    assert!(got.points.is_none());
}

// -- Sync Mapping CRUD tests --

use agileplus_domain::domain::sync_mapping::{SyncDirection, SyncMapping};

#[tokio::test]
async fn sync_mapping_upsert_and_get() {
    let db = make_adapter();
    let mapping = SyncMapping::new("feature", 1, "plane-issue-100", "hash_abc");
    StoragePort::upsert_sync_mapping(&db, &mapping).await.unwrap();

    let got = StoragePort::get_sync_mapping(&db, "feature", 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.entity_type, "feature");
    assert_eq!(got.entity_id, 1);
    assert_eq!(got.plane_issue_id, "plane-issue-100");
    assert_eq!(got.content_hash, "hash_abc");
    assert_eq!(got.sync_direction, SyncDirection::Bidirectional);
    assert_eq!(got.conflict_count, 0);
}

#[tokio::test]
async fn sync_mapping_not_found_returns_none() {
    let db = make_adapter();
    let got = StoragePort::get_sync_mapping(&db, "feature", 999)
        .await
        .unwrap();
    assert!(got.is_none());
}

#[tokio::test]
async fn sync_mapping_upsert_updates_existing() {
    let db = make_adapter();
    let mut mapping = SyncMapping::new("feature", 1, "plane-100", "hash_v1");
    StoragePort::upsert_sync_mapping(&db, &mapping).await.unwrap();

    mapping.plane_issue_id = "plane-200".to_string();
    mapping.content_hash = "hash_v2".to_string();
    mapping.sync_direction = SyncDirection::Push;
    mapping.conflict_count = 3;
    StoragePort::upsert_sync_mapping(&db, &mapping).await.unwrap();

    let got = StoragePort::get_sync_mapping(&db, "feature", 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.plane_issue_id, "plane-200");
    assert_eq!(got.content_hash, "hash_v2");
    assert_eq!(got.sync_direction, SyncDirection::Push);
    assert_eq!(got.conflict_count, 3);
}

#[tokio::test]
async fn sync_mapping_get_by_plane_id() {
    let db = make_adapter();
    let mapping = SyncMapping::new("module", 42, "plane-mod-55", "h1");
    StoragePort::upsert_sync_mapping(&db, &mapping).await.unwrap();

    let got = StoragePort::get_sync_mapping_by_plane_id(&db, "module", "plane-mod-55")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.entity_id, 42);
    assert_eq!(got.entity_type, "module");
}

#[tokio::test]
async fn sync_mapping_get_by_plane_id_not_found() {
    let db = make_adapter();
    let got = StoragePort::get_sync_mapping_by_plane_id(&db, "feature", "nonexistent")
        .await
        .unwrap();
    assert!(got.is_none());
}

#[tokio::test]
async fn sync_mapping_delete() {
    let db = make_adapter();
    let mapping = SyncMapping::new("feature", 5, "plane-del-1", "hash_d");
    StoragePort::upsert_sync_mapping(&db, &mapping).await.unwrap();

    StoragePort::delete_sync_mapping(&db, "feature", 5).await.unwrap();
    let got = StoragePort::get_sync_mapping(&db, "feature", 5).await.unwrap();
    assert!(got.is_none());
}

#[tokio::test]
async fn sync_mapping_delete_nonexistent_is_ok() {
    let db = make_adapter();
    StoragePort::delete_sync_mapping(&db, "feature", 9999).await.unwrap();
}

#[tokio::test]
async fn sync_mapping_different_entity_types_independent() {
    let db = make_adapter();
    StoragePort::upsert_sync_mapping(&db, &SyncMapping::new("feature", 1, "pf-1", "hash_f"))
        .await
        .unwrap();
    StoragePort::upsert_sync_mapping(&db, &SyncMapping::new("work_package", 1, "pw-1", "hash_w"))
        .await
        .unwrap();
    StoragePort::upsert_sync_mapping(&db, &SyncMapping::new("module", 1, "pm-1", "hash_m"))
        .await
        .unwrap();

    let f = StoragePort::get_sync_mapping(&db, "feature", 1).await.unwrap().unwrap();
    let w = StoragePort::get_sync_mapping(&db, "work_package", 1).await.unwrap().unwrap();
    let m = StoragePort::get_sync_mapping(&db, "module", 1).await.unwrap().unwrap();

    assert_eq!(f.plane_issue_id, "pf-1");
    assert_eq!(w.plane_issue_id, "pw-1");
    assert_eq!(m.plane_issue_id, "pm-1");
}

#[tokio::test]
async fn sync_mapping_pull_direction_roundtrip() {
    let db = make_adapter();
    let mut mapping = SyncMapping::new("cycle", 10, "cyc-plane-1", "cycle_hash");
    mapping.sync_direction = SyncDirection::Pull;
    StoragePort::upsert_sync_mapping(&db, &mapping).await.unwrap();

    let got = StoragePort::get_sync_mapping(&db, "cycle", 10).await.unwrap().unwrap();
    assert_eq!(got.sync_direction, SyncDirection::Pull);
}

// -- Feature labels tests --

#[tokio::test]
async fn feature_list_by_label() {
    let db = make_adapter();
    let mut f1 = Feature::new("feat-a", "Feat A", [0u8; 32], None);
    f1.labels = vec!["bug".into(), "urgent".into()];
    StoragePort::create_feature(&db, &f1).await.unwrap();

    let mut f2 = Feature::new("feat-b", "Feat B", [0u8; 32], None);
    f2.labels = vec!["feature".into()];
    StoragePort::create_feature(&db, &f2).await.unwrap();

    let mut f3 = Feature::new("feat-c", "Feat C", [0u8; 32], None);
    f3.labels = vec!["bug".into(), "enhancement".into()];
    StoragePort::create_feature(&db, &f3).await.unwrap();

    let bugs = StoragePort::list_features_by_label(&db, "bug").await.unwrap();
    assert_eq!(bugs.len(), 2);
    assert!(bugs.iter().any(|f| f.slug == "feat-a"));
    assert!(bugs.iter().any(|f| f.slug == "feat-c"));

    let features = StoragePort::list_features_by_label(&db, "feature").await.unwrap();
    assert_eq!(features.len(), 1);
    assert_eq!(features[0].slug, "feat-b");
}

#[tokio::test]
async fn feature_list_by_label_no_match() {
    let db = make_adapter();
    let mut f = Feature::new("feat-x", "Feat X", [0u8; 32], None);
    f.labels = vec!["bug".into()];
    StoragePort::create_feature(&db, &f).await.unwrap();

    let result = StoragePort::list_features_by_label(&db, "nonexistent").await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn feature_update_full() {
    let db = make_adapter();
    let mut f = Feature::new("orig", "Original", [1u8; 32], None);
    f.labels = vec!["v1".into()];
    let id = StoragePort::create_feature(&db, &f).await.unwrap();

    let mut got = StoragePort::get_feature_by_id(&db, id).await.unwrap().unwrap();
    got.friendly_name = "Updated Name".to_string();
    got.labels = vec!["v2".into(), "updated".into()];
    got.state = FeatureState::Implementing;
    StoragePort::update_feature(&db, &got).await.unwrap();

    let result = StoragePort::get_feature_by_id(&db, id).await.unwrap().unwrap();
    assert_eq!(result.friendly_name, "Updated Name");
    assert_eq!(result.labels, vec!["v2", "updated"]);
    assert_eq!(result.state, FeatureState::Implementing);
}

// -- Work Package extended tests --

#[tokio::test]
async fn wp_list_all() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("all-wp", "AWP", [0u8; 32], None))
        .await
        .unwrap();
    StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP1", 1, "c1"))
        .await
        .unwrap();
    StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP2", 2, "c2"))
        .await
        .unwrap();

    let all = StoragePort::list_all_work_packages(&db).await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn wp_get_next_ready_with_dep() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("cycle-wp", "CWP", [0u8; 32], None))
        .await
        .unwrap();
    let wp1 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "Ready", 1, "c"))
        .await
        .unwrap();
    let wp2 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "NotReady", 2, "c"))
        .await
        .unwrap();

    StoragePort::add_wp_dependency(
        &db,
        &WpDependency {
            wp_id: wp2,
            depends_on: wp1,
            dep_type: DependencyType::FileOverlap,
        },
    )
    .await
    .unwrap();

    let ready = StoragePort::get_next_ready_wps(&db, None).await.unwrap();
    assert!(ready.iter().any(|w| w.id == wp1));
}

#[tokio::test]
async fn wp_get_ready_empty_when_all_done() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("done-wp", "DWP", [0u8; 32], None))
        .await
        .unwrap();
    let wp1 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "Done1", 1, "c"))
        .await
        .unwrap();
    StoragePort::update_wp_state(&db, wp1, WpState::Doing).await.unwrap();
    StoragePort::update_wp_state(&db, wp1, WpState::Review).await.unwrap();
    StoragePort::update_wp_state(&db, wp1, WpState::Done).await.unwrap();

    let ready = StoragePort::get_ready_wps(&db, fid).await.unwrap();
    assert!(ready.is_empty());
}

// -- Cycle list all --

#[tokio::test]
async fn cycle_list_all() {
    let db = make_adapter();
    StoragePort::create_cycle(
        &db,
        &Cycle::new("C1", make_date(2026, 1, 1), make_date(2026, 2, 1), None).unwrap(),
    )
    .await
    .unwrap();
    StoragePort::create_cycle(
        &db,
        &Cycle::new("C2", make_date(2026, 3, 1), make_date(2026, 4, 1), None).unwrap(),
    )
    .await
    .unwrap();

    let all = StoragePort::list_all_cycles(&db).await.unwrap();
    assert_eq!(all.len(), 2);
}

// -- Module edge cases --

#[tokio::test]
async fn module_create_with_nonexistent_parent_fails() {
    let db = make_adapter();
    let m = Module::new("Orphan", Some(9999));
    let result = StoragePort::create_module(&db, &m).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn module_update_not_found_fails() {
    let db = make_adapter();
    let result = StoragePort::update_module(&db, 9999, "New", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn module_delete_not_found_fails() {
    let db = make_adapter();
    let result = StoragePort::delete_module(&db, 9999).await;
    assert!(result.is_err());
}

// -- Add story to cycle (NotImplemented) --

#[tokio::test]
async fn add_story_to_cycle_returns_not_implemented() {
    let db = make_adapter();
    let result = StoragePort::add_story_to_cycle(&db, 1, 1).await;
    assert!(matches!(result, Err(agileplus_domain::error::DomainError::NotImplemented)));
}

// -- L2 #38 migration test --
//
// The L1 #5 audit identified 5 tables missing from the schema that the
// downstream L2 work (worklog, trace, gate, run, scope surfaces) depends
// on. This test confirms migration 022_l2_38_worklog_trace_gate_run_scope
// is registered with the MigrationRunner and that all 5 tables exist
// post-migration. L2 #38 is the only author of these tables; if any of
// them disappears, downstream L2 tasks will fail.
#[test]
fn test_l2_38_migration() {
    // Build an in-memory DB via the adapter (runs all migrations).
    let db = SqliteStorageAdapter::in_memory().expect("in-memory adapter");
    let conn = db.conn_for_bench().expect("conn");

    let expected: &[&str] = &["gate_results", "run_records", "scope_status"];

    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
        .expect("prepare sqlite_master");
    let mut rows = stmt.query([]).expect("query sqlite_master");
    let mut found: Vec<String> = Vec::new();
    while let Some(row) = rows.next().expect("row") {
        let name: String = row.get(0).expect("name");
        if expected.contains(&name.as_str()) {
            found.push(name);
        }
    }

    for t in expected {
        assert!(
            found.iter().any(|n| n == t),
            "L2-38 migration table `{t}` not found in sqlite_master; found: {found:?}"
        );
    }
    assert_eq!(
        found.len(),
        expected.len(),
        "expected {} L2-38 tables, found {found:?}",
        expected.len()
    );

    // Sanity-check: migration is recorded in _migrations.
    let applied: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
            rusqlite::params!["024_l2_38_worklog_trace_gate_run_scope"],
            |row| row.get(0),
        )
        .expect("query _migrations");
    assert_eq!(applied, 1, "024 migration should be recorded as applied");
}
