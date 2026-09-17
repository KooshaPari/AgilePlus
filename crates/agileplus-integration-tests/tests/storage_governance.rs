//! StoragePort contract: governance contracts, policy rules, projects, and
//! Plane.so sync mappings.
//!
//! Every test uses an isolated in-memory SQLite adapter through the public
//! `StoragePort` trait — no file I/O and no network required.
//!
//! Traceability: WP19-T107/T108 (governance + sync surface)

use agileplus_domain::{
    domain::{
        feature::Feature,
        governance::{
            GovernanceContract, GovernanceRule, PolicyCheck, PolicyDefinition, PolicyDomain,
            PolicyRule,
        },
        project::Project,
        sync_mapping::{SyncDirection, SyncMapping},
    },
    error::DomainError,
    ports::StoragePort,
};
use agileplus_sqlite::SqliteStorageAdapter;
use chrono::Utc;

fn storage() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory adapter should initialise")
}

async fn seed_feature(storage: &SqliteStorageAdapter, slug: &str) -> i64 {
    storage
        .create_feature(&Feature::new(slug, slug, [0u8; 32], None))
        .await
        .expect("create_feature should succeed")
}

fn contract(feature_id: i64, version: i32) -> GovernanceContract {
    GovernanceContract {
        id: 0,
        feature_id,
        version,
        rules: vec![GovernanceRule {
            transition: "implementing->validated".to_string(),
            required_evidence: vec!["test_result".to_string()],
            policy_refs: vec![1],
        }],
        bound_at: Utc::now(),
    }
}

fn policy(domain: PolicyDomain, active: bool) -> PolicyRule {
    PolicyRule {
        id: 0,
        domain,
        rule: PolicyDefinition {
            description: "unit tests must pass".to_string(),
            check: PolicyCheck::Automated,
        },
        active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

// --- Governance contracts -------------------------------------------------

#[tokio::test]
async fn governance_contract_create_and_get_by_version() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "contract-feature").await;

    let id = storage
        .create_governance_contract(&contract(feature_id, 1))
        .await
        .expect("create_governance_contract should succeed");
    assert!(id > 0);

    let stored = storage
        .get_governance_contract(feature_id, 1)
        .await
        .expect("get_governance_contract should succeed")
        .expect("contract should exist");
    assert_eq!(stored.feature_id, feature_id);
    assert_eq!(stored.version, 1);
}

#[tokio::test]
async fn governance_contract_rules_roundtrip() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "contract-rules").await;
    storage
        .create_governance_contract(&contract(feature_id, 1))
        .await
        .expect("create should succeed");

    let stored = storage
        .get_governance_contract(feature_id, 1)
        .await
        .expect("get should succeed")
        .expect("contract should exist");
    assert_eq!(stored.rules.len(), 1);
    assert_eq!(stored.rules[0].transition, "implementing->validated");
    assert_eq!(stored.rules[0].required_evidence, vec!["test_result"]);
    assert_eq!(stored.rules[0].policy_refs, vec![1]);
}

#[tokio::test]
async fn governance_contract_latest_returns_highest_version() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "contract-latest").await;
    storage
        .create_governance_contract(&contract(feature_id, 1))
        .await
        .expect("create should succeed");
    storage
        .create_governance_contract(&contract(feature_id, 3))
        .await
        .expect("create should succeed");
    storage
        .create_governance_contract(&contract(feature_id, 2))
        .await
        .expect("create should succeed");

    let latest = storage
        .get_latest_governance_contract(feature_id)
        .await
        .expect("get_latest_governance_contract should succeed")
        .expect("contract should exist");
    assert_eq!(latest.version, 3);
}

#[tokio::test]
async fn governance_contract_missing_returns_none() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "contract-missing").await;

    let by_version = storage
        .get_governance_contract(feature_id, 9)
        .await
        .expect("get should succeed");
    assert!(by_version.is_none());

    let latest = storage
        .get_latest_governance_contract(feature_id)
        .await
        .expect("get latest should succeed");
    assert!(latest.is_none());
}

#[tokio::test]
async fn governance_contract_duplicate_version_is_rejected() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "contract-dup").await;
    storage
        .create_governance_contract(&contract(feature_id, 1))
        .await
        .expect("create should succeed");

    let result = storage
        .create_governance_contract(&contract(feature_id, 1))
        .await;
    assert!(
        matches!(result, Err(DomainError::Storage(_))),
        "(feature_id, version) is unique"
    );
}

// --- Policy rules ---------------------------------------------------------

#[tokio::test]
async fn policy_rule_create_and_list_active() {
    let storage = storage();
    let id = storage
        .create_policy_rule(&policy(PolicyDomain::Quality, true))
        .await
        .expect("create_policy_rule should succeed");
    assert!(id > 0);

    let active = storage
        .list_active_policies()
        .await
        .expect("list_active_policies should succeed");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].domain, PolicyDomain::Quality);
    assert_eq!(active[0].rule.description, "unit tests must pass");
}

#[tokio::test]
async fn policy_rule_inactive_is_excluded_from_active_list() {
    let storage = storage();
    storage
        .create_policy_rule(&policy(PolicyDomain::Security, true))
        .await
        .expect("create should succeed");
    storage
        .create_policy_rule(&policy(PolicyDomain::Compliance, false))
        .await
        .expect("create should succeed");

    let active = storage
        .list_active_policies()
        .await
        .expect("list_active_policies should succeed");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].domain, PolicyDomain::Security);
}

#[tokio::test]
async fn policy_rule_domains_roundtrip() {
    let storage = storage();
    for domain in [
        PolicyDomain::Security,
        PolicyDomain::Quality,
        PolicyDomain::Compliance,
        PolicyDomain::Performance,
        PolicyDomain::Custom,
    ] {
        storage
            .create_policy_rule(&policy(domain, true))
            .await
            .expect("create_policy_rule should succeed");
    }

    let active = storage
        .list_active_policies()
        .await
        .expect("list_active_policies should succeed");
    assert_eq!(active.len(), 5);
    for domain in [
        PolicyDomain::Security,
        PolicyDomain::Quality,
        PolicyDomain::Compliance,
        PolicyDomain::Performance,
        PolicyDomain::Custom,
    ] {
        assert!(
            active.iter().any(|rule| rule.domain == domain),
            "missing policy domain {domain:?}"
        );
    }
}

#[tokio::test]
async fn policy_rule_policy_check_variants_roundtrip() {
    let storage = storage();
    let checks = [
        PolicyCheck::ManualApproval,
        PolicyCheck::Automated,
        PolicyCheck::Custom {
            script: "check.sh".to_string(),
        },
    ];
    for (index, check) in checks.into_iter().enumerate() {
        let mut rule = policy(PolicyDomain::Custom, true);
        rule.rule.check = check;
        rule.rule.description = format!("check-{index}");
        storage
            .create_policy_rule(&rule)
            .await
            .expect("create_policy_rule should succeed");
    }

    let active = storage
        .list_active_policies()
        .await
        .expect("list_active_policies should succeed");
    assert_eq!(active.len(), 3);
    assert!(
        active
            .iter()
            .any(|rule| matches!(rule.rule.check, PolicyCheck::ManualApproval))
    );
    assert!(
        active
            .iter()
            .any(|rule| matches!(rule.rule.check, PolicyCheck::Automated))
    );
}

#[tokio::test]
async fn policy_rule_matches_reference_by_id_and_prefix() {
    let storage = storage();
    let id = storage
        .create_policy_rule(&policy(PolicyDomain::Quality, true))
        .await
        .expect("create_policy_rule should succeed");

    let rule = &storage
        .list_active_policies()
        .await
        .expect("list_active_policies should succeed")[0];
    assert_eq!(rule.id, id);
    assert!(rule.matches_reference(&id.to_string()));
    assert!(rule.matches_reference(&format!("policy:{id}")));
    assert!(!rule.matches_reference("policy:9999"));
}

// --- Projects -------------------------------------------------------------

#[tokio::test]
async fn project_create_and_get_by_slug() {
    let storage = storage();
    let project = Project::new("AgilePlus", "agileplus").expect("valid project");

    let id = storage
        .create_project(&project)
        .await
        .expect("create_project should succeed");
    assert!(id > 0);

    let stored = storage
        .get_project_by_slug("agileplus")
        .await
        .expect("get_project_by_slug should succeed")
        .expect("project should exist");
    assert_eq!(stored.name, "AgilePlus");
    assert_eq!(stored.slug, "agileplus");
}

#[tokio::test]
async fn project_get_by_slug_missing_returns_none() {
    let storage = storage();
    let result = storage
        .get_project_by_slug("ghost")
        .await
        .expect("lookup should succeed");
    assert!(result.is_none());
}

#[tokio::test]
async fn project_duplicate_slug_is_rejected() {
    let storage = storage();
    let project = Project::new("First", "shared-slug").expect("valid project");
    storage
        .create_project(&project)
        .await
        .expect("create_project should succeed");

    let result = storage
        .create_project(&Project::new("Second", "shared-slug").expect("valid project"))
        .await;
    assert!(
        matches!(result, Err(DomainError::Storage(_))),
        "project slug is unique"
    );
}

#[tokio::test]
async fn project_description_roundtrips() {
    let storage = storage();
    let mut project = Project::new("Described", "described").expect("valid project");
    project.description = Some("a described project".to_string());

    storage
        .create_project(&project)
        .await
        .expect("create_project should succeed");

    let stored = storage
        .get_project_by_slug("described")
        .await
        .expect("lookup should succeed")
        .expect("project should exist");
    assert_eq!(stored.description.as_deref(), Some("a described project"));
}

// --- Sync mappings --------------------------------------------------------

#[tokio::test]
async fn sync_mapping_upsert_and_get() {
    let storage = storage();
    let mapping = SyncMapping::new("feature", 1, "plane-100", "hash-a");

    storage
        .upsert_sync_mapping(&mapping)
        .await
        .expect("upsert_sync_mapping should succeed");

    let stored = storage
        .get_sync_mapping("feature", 1)
        .await
        .expect("get_sync_mapping should succeed")
        .expect("mapping should exist");
    assert_eq!(stored.plane_issue_id, "plane-100");
    assert_eq!(stored.content_hash, "hash-a");
}

#[tokio::test]
async fn sync_mapping_upsert_updates_existing_row() {
    let storage = storage();
    storage
        .upsert_sync_mapping(&SyncMapping::new("feature", 1, "plane-100", "hash-a"))
        .await
        .expect("upsert should succeed");

    let mut updated = SyncMapping::new("feature", 1, "plane-100", "hash-b");
    updated.sync_direction = SyncDirection::Push;
    updated.increment_conflict();
    storage
        .upsert_sync_mapping(&updated)
        .await
        .expect("upsert should succeed");

    let stored = storage
        .get_sync_mapping("feature", 1)
        .await
        .expect("get should succeed")
        .expect("mapping should exist");
    assert_eq!(stored.content_hash, "hash-b");
    assert_eq!(stored.sync_direction, SyncDirection::Push);
    assert_eq!(stored.conflict_count, 1);
}

#[tokio::test]
async fn sync_mapping_get_missing_returns_none() {
    let storage = storage();
    let result = storage
        .get_sync_mapping("feature", 404)
        .await
        .expect("lookup should succeed");
    assert!(result.is_none());
}

#[tokio::test]
async fn sync_mapping_get_by_plane_id() {
    let storage = storage();
    storage
        .upsert_sync_mapping(&SyncMapping::new("wp", 7, "plane-777", "hash"))
        .await
        .expect("upsert should succeed");

    let stored = storage
        .get_sync_mapping_by_plane_id("wp", "plane-777")
        .await
        .expect("get_sync_mapping_by_plane_id should succeed")
        .expect("mapping should exist");
    assert_eq!(stored.entity_id, 7);
}

#[tokio::test]
async fn sync_mapping_delete_removes_row() {
    let storage = storage();
    storage
        .upsert_sync_mapping(&SyncMapping::new("feature", 5, "plane-500", "hash"))
        .await
        .expect("upsert should succeed");

    storage
        .delete_sync_mapping("feature", 5)
        .await
        .expect("delete_sync_mapping should succeed");

    let stored = storage
        .get_sync_mapping("feature", 5)
        .await
        .expect("get should succeed");
    assert!(stored.is_none());
}

#[tokio::test]
async fn sync_mapping_directions_roundtrip() {
    let storage = storage();
    for (index, direction) in [
        SyncDirection::Push,
        SyncDirection::Pull,
        SyncDirection::Bidirectional,
    ]
    .into_iter()
    .enumerate()
    {
        let mut mapping = SyncMapping::new("feature", index as i64, format!("plane-{index}"), "h");
        mapping.sync_direction = direction;
        storage
            .upsert_sync_mapping(&mapping)
            .await
            .expect("upsert should succeed");

        let stored = storage
            .get_sync_mapping("feature", index as i64)
            .await
            .expect("get should succeed")
            .expect("mapping should exist");
        assert_eq!(stored.sync_direction, direction);
    }
}
