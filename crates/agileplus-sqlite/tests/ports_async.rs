//! Integration tests for the async port implementations:
//! `StoragePort`, `ContentStoragePort`, `EventStore`, and `TriagePort`.

use agileplus_domain::{
    domain::{
        audit::{AuditEntry, EvidenceRef},
        backlog::{BacklogItem, BacklogPriority, BacklogStatus, Intent},
        cycle::{Cycle, CycleFeature, CycleState},
        epic::{Epic, EpicStatus},
        event::Event,
        feature::Feature,
        governance::{
            Evidence, EvidenceType, GovernanceContract, GovernanceRule, PolicyCheck,
            PolicyDefinition, PolicyDomain, PolicyRule,
        },
        metric::Metric,
        module::{Module, ModuleFeatureTag},
        project::Project,
        state_machine::FeatureState,
        story::{Story, StoryStatus},
        sync_mapping::{SyncDirection, SyncMapping},
        user::{User, UserRole, UserStatus},
        work_package::{DependencyType, WpDependency, WpState, WorkPackage},
    },
    ports::{StoragePort, TriageOutcome, TriagePort},
};
use agileplus_events::EventStore;
use agileplus_sqlite::{SqliteStorageAdapter, SqliteTriageAdapter};

fn adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().unwrap()
}

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn feature(slug: &str) -> Feature {
    let now = chrono::Utc::now();
    Feature {
        id: 0,
        slug: slug.into(),
        friendly_name: format!("Feature {slug}"),
        state: FeatureState::Created,
        spec_hash: [0u8; 32],
        target_branch: "main".into(),
        plane_issue_id: None,
        plane_state_id: None,
        labels: vec![],
        module_id: None,
        project_id: None,
        created_at: now,
        updated_at: now,
        created_at_commit: None,
        last_modified_commit: None,
    }
}

fn wp(feature_id: i64, title: &str, seq: i32) -> WorkPackage {
    let now = chrono::Utc::now();
    WorkPackage {
        id: 0,
        feature_id,
        title: title.into(),
        state: WpState::Planned,
        sequence: seq,
        file_scope: vec!["src/lib.rs".into()],
        acceptance_criteria: "tests pass".into(),
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

fn evidence(wp_id: i64) -> Evidence {
    Evidence {
        id: 0,
        wp_id,
        fr_id: "FR-1".into(),
        evidence_type: EvidenceType::TestResult,
        artifact_path: "out.txt".into(),
        metadata: None,
        created_at: chrono::Utc::now(),
    }
}

fn policy() -> PolicyRule {
    PolicyRule {
        id: 0,
        domain: PolicyDomain::Quality,
        rule: PolicyDefinition {
            description: "coverage >= 85".into(),
            check: PolicyCheck::ThresholdMet {
                metric: "coverage".into(),
                min: 85.0,
            },
        },
        active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

fn metric(feature_id: Option<i64>) -> Metric {
    Metric {
        id: 0,
        feature_id,
        command: "cargo test".into(),
        duration_ms: 100,
        agent_runs: 1,
        review_cycles: 0,
        metadata: None,
        timestamp: chrono::Utc::now(),
    }
}

fn contract(feature_id: i64) -> GovernanceContract {
    GovernanceContract {
        id: 0,
        feature_id,
        version: 1,
        rules: vec![GovernanceRule {
            transition: "ship".into(),
            required_evidence: vec!["test_result".into()],
            policy_refs: vec![],
        }],
        bound_at: chrono::Utc::now(),
    }
}

fn audit(feature_id: i64) -> AuditEntry {
    AuditEntry {
        id: 0,
        feature_id,
        wp_id: None,
        timestamp: chrono::Utc::now(),
        actor: "port-tester".into(),
        transition: "created".into(),
        evidence_refs: vec![EvidenceRef {
            evidence_id: 1,
            fr_id: "FR-1".into(),
        }],
        prev_hash: [0u8; 32],
        hash: [1u8; 32],
        event_id: None,
        archived_to: None,
    }
}

fn event(seq: i64) -> Event {
    Event {
        id: 0,
        entity_type: "Feature".into(),
        entity_id: 1,
        event_type: "created".into(),
        payload: serde_json::json!({"seq": seq}),
        actor: "port-tester".into(),
        timestamp: chrono::Utc::now(),
        prev_hash: [0u8; 32],
        hash: [seq as u8; 32],
        sequence: seq,
    }
}

// ---------------------------------------------------------------------------
// StoragePort: feature CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn port_feature_crud_roundtrip() {
    let a = adapter();
    let id = a.create_feature(&feature("port-feat")).await.unwrap();
    assert!(id > 0);

    let by_id = a.get_feature_by_id(id).await.unwrap().unwrap();
    assert_eq!(by_id.slug, "port-feat");
    let by_slug = a.get_feature_by_slug("port-feat").await.unwrap().unwrap();
    assert_eq!(by_slug.id, id);

    a.update_feature_state(id, FeatureState::Planned).await.unwrap();
    assert_eq!(
        a.get_feature_by_id(id).await.unwrap().unwrap().state,
        FeatureState::Planned
    );

    let mut updated = by_id.clone();
    updated.friendly_name = "Renamed".into();
    updated.labels = vec!["x".into()];
    a.update_feature(&updated).await.unwrap();
    let got = a.get_feature_by_id(id).await.unwrap().unwrap();
    assert_eq!(got.friendly_name, "Renamed");
    assert_eq!(got.labels, vec!["x"]);
}

#[tokio::test]
async fn port_feature_missing_returns_none() {
    let a = adapter();
    assert!(a.get_feature_by_id(42).await.unwrap().is_none());
    assert!(a.get_feature_by_slug("nope").await.unwrap().is_none());
}

#[tokio::test]
async fn port_list_features_by_state_and_all() {
    let a = adapter();
    a.create_feature(&feature("a")).await.unwrap();
    let mut planned = feature("b");
    planned.state = FeatureState::Planned;
    a.create_feature(&planned).await.unwrap();

    assert_eq!(a.list_all_features().await.unwrap().len(), 2);
    assert_eq!(
        a.list_features_by_state(FeatureState::Created).await.unwrap().len(),
        1
    );
    assert_eq!(
        a.list_features_by_state(FeatureState::Planned).await.unwrap().len(),
        1
    );
}

#[tokio::test]
async fn port_list_features_by_label() {
    let a = adapter();
    let mut tagged = feature("tagged");
    tagged.labels = vec!["security".into()];
    a.create_feature(&tagged).await.unwrap();
    a.create_feature(&feature("plain")).await.unwrap();
    let got = a.list_features_by_label("security").await.unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].slug, "tagged");
}

// ---------------------------------------------------------------------------
// StoragePort: work packages
// ---------------------------------------------------------------------------

#[tokio::test]
async fn port_work_package_crud_roundtrip() {
    let a = adapter();
    let fid = a.create_feature(&feature("wp-feat")).await.unwrap();
    let id = a.create_work_package(&wp(fid, "WP01", 1)).await.unwrap();
    assert!(id > 0);

    let got = a.get_work_package(id).await.unwrap().unwrap();
    assert_eq!(got.title, "WP01");
    assert_eq!(got.file_scope, vec!["src/lib.rs"]);

    a.update_wp_state(id, WpState::Doing).await.unwrap();
    assert_eq!(a.get_work_package(id).await.unwrap().unwrap().state, WpState::Doing);
}

#[tokio::test]
async fn port_work_package_missing_none() {
    let a = adapter();
    assert!(a.get_work_package(1234).await.unwrap().is_none());
}

#[tokio::test]
async fn port_list_wps_by_feature() {
    let a = adapter();
    let fid = a.create_feature(&feature("listwps")).await.unwrap();
    a.create_work_package(&wp(fid, "one", 1)).await.unwrap();
    a.create_work_package(&wp(fid, "two", 2)).await.unwrap();
    assert_eq!(a.list_wps_by_feature(fid).await.unwrap().len(), 2);
    assert!(a.list_wps_by_feature(999).await.unwrap().is_empty());
    assert_eq!(a.list_all_work_packages().await.unwrap().len(), 2);
}

#[tokio::test]
async fn port_wp_dependencies_and_ready() {
    let a = adapter();
    let fid = a.create_feature(&feature("dep-feat")).await.unwrap();
    let wp1 = a.create_work_package(&wp(fid, "dep", 1)).await.unwrap();
    let wp2 = a.create_work_package(&wp(fid, "blocked", 2)).await.unwrap();

    a.add_wp_dependency(&WpDependency {
        wp_id: wp2,
        depends_on: wp1,
        dep_type: DependencyType::Explicit,
    })
    .await
    .unwrap();

    let deps = a.get_wp_dependencies(wp2).await.unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].depends_on, wp1);

    // wp1 is ready; wp2 is blocked by wp1 (planned != done).
    let ready = a.get_ready_wps(fid).await.unwrap();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, wp1);

    // After completing wp1, wp2 becomes ready.
    a.update_wp_state(wp1, WpState::Done).await.unwrap();
    let ready = a.get_ready_wps(fid).await.unwrap();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, wp2);

    assert_eq!(a.get_next_ready_wps(None).await.unwrap().len(), 1);
}

#[tokio::test]
async fn port_wp_dependency_idempotent() {
    let a = adapter();
    let fid = a.create_feature(&feature("depidem")).await.unwrap();
    let wp1 = a.create_work_package(&wp(fid, "a", 1)).await.unwrap();
    let wp2 = a.create_work_package(&wp(fid, "b", 2)).await.unwrap();
    let dep = WpDependency {
        wp_id: wp2,
        depends_on: wp1,
        dep_type: DependencyType::Data,
    };
    a.add_wp_dependency(&dep).await.unwrap();
    a.add_wp_dependency(&dep).await.unwrap();
    assert_eq!(a.get_wp_dependencies(wp2).await.unwrap().len(), 1);
}

#[tokio::test]
async fn port_create_work_package_for_story_and_list() {
    let a = adapter();
    let pid = a.create_project(&Project::new("P", "p").unwrap()).await.unwrap();
    let eid = a.create_epic(&Epic::new(pid, "E").unwrap()).await.unwrap();
    let sid = a
        .create_story(&Story::new(eid, pid, "S", Some(1)).unwrap())
        .await
        .unwrap();
    let fid = a.create_feature(&feature("story-wp")).await.unwrap();

    let wp_id = a
        .create_work_package_for_story(sid, &wp(fid, "story wp", 1))
        .await
        .unwrap();
    let linked = a.list_wps_by_story(sid).await.unwrap();
    assert_eq!(linked.len(), 1);
    assert_eq!(linked[0].id, wp_id);
    assert!(a.list_wps_by_story(9999).await.unwrap().is_empty());
}

#[tokio::test]
async fn port_add_story_to_cycle_not_implemented() {
    let a = adapter();
    let err = a.add_story_to_cycle(1, 1).await.unwrap_err();
    assert!(matches!(err, agileplus_domain::error::DomainError::NotImplemented));
}

#[tokio::test]
async fn port_upsert_story_by_requirement_id_not_implemented() {
    let a = adapter();
    let err = a
        .upsert_story_by_requirement_id(&Story::new(1, 1, "S", None).unwrap())
        .await
        .unwrap_err();
    assert!(matches!(err, agileplus_domain::error::DomainError::NotImplemented));
}

// ---------------------------------------------------------------------------
// StoragePort: audit / evidence / policy / metrics
// ---------------------------------------------------------------------------

#[tokio::test]
async fn port_audit_append_and_read() {
    let a = adapter();
    let fid = a.create_feature(&feature("audit-feat")).await.unwrap();
    a.append_audit_entry(&audit(fid)).await.unwrap();
    let trail = a.get_audit_trail(fid).await.unwrap();
    assert_eq!(trail.len(), 1);
    assert_eq!(trail[0].evidence_refs.len(), 1);
    assert!(a.get_latest_audit_entry(fid).await.unwrap().is_some());
    assert!(a.get_audit_trail(9999).await.unwrap().is_empty());
}

#[tokio::test]
async fn port_evidence_create_and_query() {
    let a = adapter();
    let fid = a.create_feature(&feature("ev-feat")).await.unwrap();
    let wpid = a.create_work_package(&wp(fid, "w", 1)).await.unwrap();
    a.create_evidence(&evidence(wpid)).await.unwrap();
    assert_eq!(a.get_evidence_by_wp(wpid).await.unwrap().len(), 1);
    assert_eq!(a.get_evidence_by_fr("FR-1").await.unwrap().len(), 1);
    assert!(a.get_evidence_by_fr("FR-NONE").await.unwrap().is_empty());
}

#[tokio::test]
async fn port_policy_create_and_list() {
    let a = adapter();
    a.create_policy_rule(&policy()).await.unwrap();
    let active = a.list_active_policies().await.unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].domain, PolicyDomain::Quality);
}

#[tokio::test]
async fn port_metrics_record_and_query() {
    let a = adapter();
    let fid = a.create_feature(&feature("metric-feat")).await.unwrap();
    a.record_metric(&metric(Some(fid))).await.unwrap();
    let got = a.get_metrics_by_feature(fid).await.unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].command, "cargo test");
}

// ---------------------------------------------------------------------------
// StoragePort: governance contracts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn port_governance_contract_roundtrip() {
    let a = adapter();
    let fid = a.create_feature(&feature("gov-feat")).await.unwrap();
    a.create_governance_contract(&contract(fid)).await.unwrap();
    let got = a.get_governance_contract(fid, 1).await.unwrap().unwrap();
    assert_eq!(got.rules.len(), 1);
    assert!(a.get_latest_governance_contract(fid).await.unwrap().is_some());
    assert!(a.get_latest_governance_contract(9999).await.unwrap().is_none());
}

// ---------------------------------------------------------------------------
// StoragePort: modules
// ---------------------------------------------------------------------------

#[tokio::test]
async fn port_module_crud_roundtrip() {
    let a = adapter();
    let id = a.create_module(&Module::new("Auth", None)).await.unwrap();
    assert_eq!(a.get_module(id).await.unwrap().unwrap().slug, "auth");
    assert_eq!(
        a.get_module_by_slug("auth").await.unwrap().unwrap().id,
        id
    );
    a.update_module(id, "Authentication", Some("desc")).await.unwrap();
    let got = a.get_module(id).await.unwrap().unwrap();
    assert_eq!(got.slug, "authentication");
    assert_eq!(got.description.as_deref(), Some("desc"));
    assert_eq!(a.list_root_modules().await.unwrap().len(), 1);
    a.delete_module(id).await.unwrap();
    assert!(a.get_module(id).await.unwrap().is_none());
}

#[tokio::test]
async fn port_module_hierarchy_and_view() {
    let a = adapter();
    let root = a.create_module(&Module::new("Root", None)).await.unwrap();
    let child = a.create_module(&Module::new("Child", Some(root))).await.unwrap();
    assert_eq!(a.list_child_modules(root).await.unwrap().len(), 1);
    assert_eq!(a.list_child_modules(root).await.unwrap()[0].id, child);

    let view = a.get_module_with_features(root).await.unwrap().unwrap();
    assert_eq!(view.child_modules.len(), 1);
    assert!(a.get_module_with_features(999).await.unwrap().is_none());
}

#[tokio::test]
async fn port_module_tagging() {
    let a = adapter();
    let mid = a.create_module(&Module::new("M", None)).await.unwrap();
    let fid = a.create_feature(&feature("tagme")).await.unwrap();
    a.tag_feature_to_module(&ModuleFeatureTag::new(mid, fid))
        .await
        .unwrap();
    let view = a.get_module_with_features(mid).await.unwrap().unwrap();
    assert_eq!(view.tagged_features.len(), 1);

    a.untag_feature_from_module(mid, fid).await.unwrap();
    let view = a.get_module_with_features(mid).await.unwrap().unwrap();
    assert!(view.tagged_features.is_empty());
}

// ---------------------------------------------------------------------------
// StoragePort: cycles
// ---------------------------------------------------------------------------

#[tokio::test]
async fn port_cycle_crud_and_joins() {
    let a = adapter();
    let cid = a
        .create_cycle(&Cycle::new("Sprint", date(2026, 1, 1), date(2026, 2, 1), None).unwrap())
        .await
        .unwrap();
    assert_eq!(a.get_cycle(cid).await.unwrap().unwrap().state, CycleState::Draft);

    a.update_cycle_state(cid, CycleState::Active).await.unwrap();
    assert_eq!(a.list_cycles_by_state(CycleState::Active).await.unwrap().len(), 1);
    assert_eq!(a.list_all_cycles().await.unwrap().len(), 1);
    assert!(a.list_cycles_by_module(1).await.unwrap().is_empty());
    assert!(a.get_cycle(1234).await.unwrap().is_none());

    let fid = a.create_feature(&feature("cycle-feat")).await.unwrap();
    a.add_feature_to_cycle(&CycleFeature::new(cid, fid)).await.unwrap();
    let view = a.get_cycle_with_features(cid).await.unwrap().unwrap();
    assert_eq!(view.features.len(), 1);
    assert!(a.get_cycle_with_features(999).await.unwrap().is_none());

    a.remove_feature_from_cycle(cid, fid).await.unwrap();
    let view = a.get_cycle_with_features(cid).await.unwrap().unwrap();
    assert!(view.features.is_empty());
}

// ---------------------------------------------------------------------------
// StoragePort: sync mappings
// ---------------------------------------------------------------------------

#[tokio::test]
async fn port_sync_mapping_roundtrip() {
    let a = adapter();
    let mut m = SyncMapping::new("feature", 1, "plane-1", "hash");
    m.sync_direction = SyncDirection::Push;
    a.upsert_sync_mapping(&m).await.unwrap();

    let got = a.get_sync_mapping("feature", 1).await.unwrap().unwrap();
    assert_eq!(got.plane_issue_id, "plane-1");
    assert_eq!(got.sync_direction, SyncDirection::Push);

    let by_plane = a
        .get_sync_mapping_by_plane_id("feature", "plane-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(by_plane.entity_id, 1);

    a.delete_sync_mapping("feature", 1).await.unwrap();
    assert!(a.get_sync_mapping("feature", 1).await.unwrap().is_none());
}

// ---------------------------------------------------------------------------
// StoragePort: projects
// ---------------------------------------------------------------------------

#[tokio::test]
async fn port_project_crud_roundtrip() {
    let a = adapter();
    let p = Project::new("Port Project", "port-project").unwrap();
    let id = a.create_project(&p).await.unwrap();
    assert_eq!(
        a.get_project_by_slug("port-project").await.unwrap().unwrap().id,
        id
    );
    assert_eq!(a.get_project_by_id(id).await.unwrap().unwrap().name, "Port Project");
    assert_eq!(a.list_all_projects().await.unwrap().len(), 1);
    assert!(a.get_project_by_id(999).await.unwrap().is_none());

    a.delete_project(id).await.unwrap();
    assert!(a.get_project_by_id(id).await.unwrap().is_none());
}

// ---------------------------------------------------------------------------
// StoragePort: users
// ---------------------------------------------------------------------------

#[tokio::test]
async fn port_user_crud_roundtrip() {
    let a = adapter();
    let u = User::new("Port User", "port@example.com", UserRole::Admin).unwrap();
    let id = a.create_user(&u).await.unwrap();
    assert_eq!(a.get_user(id).await.unwrap().unwrap().display_name, "Port User");
    assert_eq!(
        a.get_user_by_email("port@example.com")
            .await
            .unwrap()
            .unwrap()
            .id,
        id
    );

    a.update_user_status(id, UserStatus::Suspended).await.unwrap();
    a.update_user_role(id, UserRole::Viewer).await.unwrap();
    let got = a.get_user(id).await.unwrap().unwrap();
    assert_eq!(got.status, UserStatus::Suspended);
    assert_eq!(got.role, UserRole::Viewer);

    assert_eq!(a.list_all_users().await.unwrap().len(), 1);
    a.delete_user(id).await.unwrap();
    assert!(a.get_user(id).await.unwrap().is_none());
}

// ---------------------------------------------------------------------------
// StoragePort: epics / stories
// ---------------------------------------------------------------------------

#[tokio::test]
async fn port_epic_and_story_roundtrip() {
    let a = adapter();
    let pid = a.create_project(&Project::new("P", "p-epic").unwrap()).await.unwrap();
    let eid = a.create_epic(&Epic::new(pid, "Epic").unwrap()).await.unwrap();
    assert_eq!(a.get_epic(eid).await.unwrap().unwrap().title, "Epic");
    a.update_epic_status(eid, EpicStatus::Active).await.unwrap();
    assert_eq!(a.list_epics_by_project(pid).await.unwrap().len(), 1);

    let sid = a
        .create_story(&Story::new(eid, pid, "Story", Some(2)).unwrap())
        .await
        .unwrap();
    assert_eq!(a.get_story(sid).await.unwrap().unwrap().points, Some(2));
    a.update_story_status(sid, StoryStatus::Done).await.unwrap();
    assert_eq!(a.list_stories_by_epic(eid).await.unwrap().len(), 1);
    assert_eq!(a.list_stories_by_project(pid).await.unwrap().len(), 1);

    a.delete_story(sid).await.unwrap();
    a.delete_epic(eid).await.unwrap();
    assert!(a.get_story(sid).await.unwrap().is_none());
    assert!(a.get_epic(eid).await.unwrap().is_none());
}

// ---------------------------------------------------------------------------
// EventStore port
// ---------------------------------------------------------------------------

#[tokio::test]
async fn event_store_append_and_query() {
    let a = adapter();
    let id = a.append(&event(1)).await.unwrap();
    assert!(id > 0);
    let got = a.get_events("Feature", 1).await.unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].sequence, 1);
    assert_eq!(got[0].payload["seq"], 1);
}

#[tokio::test]
async fn event_store_since_and_latest() {
    let a = adapter();
    a.append(&event(1)).await.unwrap();
    a.append(&event(2)).await.unwrap();
    a.append(&event(3)).await.unwrap();

    let since = a.get_events_since("Feature", 1, 1).await.unwrap();
    assert_eq!(since.len(), 2);
    assert_eq!(a.get_latest_sequence("Feature", 1).await.unwrap(), 3);
    assert_eq!(a.get_latest_sequence("Feature", 99).await.unwrap(), 0);
}

#[tokio::test]
async fn event_store_range() {
    let a = adapter();
    let mut e = event(1);
    e.timestamp = chrono::Utc::now();
    a.append(&e).await.unwrap();

    let from = chrono::Utc::now() - chrono::Duration::hours(1);
    let to = chrono::Utc::now() + chrono::Duration::hours(1);
    assert_eq!(a.get_events_by_range("Feature", 1, from, to).await.unwrap().len(), 1);
}

#[tokio::test]
async fn event_store_duplicate_sequence_errors() {
    let a = adapter();
    a.append(&event(1)).await.unwrap();
    assert!(a.append(&event(1)).await.is_err());
}

// ---------------------------------------------------------------------------
// TriagePort
// ---------------------------------------------------------------------------

/// Seed a backlog item through the ContentStoragePort without bringing that
/// trait into scope (which would make `adapter.method()` calls ambiguous with
/// `StoragePort`).
async fn seed_ticket(t: &SqliteTriageAdapter, title: &str, intent: Intent) -> i64 {
    agileplus_domain::ports::ContentStoragePort::create_backlog_item(
        t.storage(),
        &BacklogItem::from_triage(title.into(), "d".into(), intent, "gh".into()),
    )
    .await
    .unwrap()
}

async fn ticket_status(t: &SqliteTriageAdapter, id: i64) -> BacklogStatus {
    agileplus_domain::ports::ContentStoragePort::get_backlog_item(t.storage(), id)
        .await
        .unwrap()
        .unwrap()
        .status
}

#[tokio::test]
async fn triage_no_ticket_available() {
    let t = SqliteTriageAdapter::in_memory().unwrap();
    let err = t.next_ticket().await.unwrap_err();
    assert!(matches!(err, agileplus_domain::ports::TriageError::NoTicketAvailable));
}

#[tokio::test]
async fn triage_next_ticket_returns_priority_item() {
    let t = SqliteTriageAdapter::in_memory().unwrap();
    seed_ticket(&t, "urgent", Intent::Bug).await;
    seed_ticket(&t, "low", Intent::Idea).await;

    let ticket = t.next_ticket().await.unwrap();
    assert_eq!(ticket.title, "urgent");
    assert_eq!(ticket.priority, BacklogPriority::High);
}

#[tokio::test]
async fn triage_record_outcome_accepted() {
    let t = SqliteTriageAdapter::in_memory().unwrap();
    let id = seed_ticket(&t, "x", Intent::Bug).await;
    t.record_outcome(&id.to_string(), TriageOutcome::Accepted)
        .await
        .unwrap();
    assert_eq!(ticket_status(&t, id).await, BacklogStatus::Triaged);
}

#[tokio::test]
async fn triage_record_outcome_dismissed() {
    let t = SqliteTriageAdapter::in_memory().unwrap();
    let id = seed_ticket(&t, "y", Intent::Idea).await;
    t.record_outcome(&id.to_string(), TriageOutcome::Dismissed)
        .await
        .unwrap();
    assert_eq!(ticket_status(&t, id).await, BacklogStatus::Dismissed);
}

#[tokio::test]
async fn triage_record_outcome_invalid_id() {
    let t = SqliteTriageAdapter::in_memory().unwrap();
    let err = t.record_outcome("not-a-number", TriageOutcome::Accepted).await.unwrap_err();
    assert!(matches!(
        err,
        agileplus_domain::ports::TriageError::InvalidTicketId(_)
    ));
}

#[tokio::test]
async fn triage_record_outcome_missing_ticket() {
    let t = SqliteTriageAdapter::in_memory().unwrap();
    let err = t.record_outcome("4242", TriageOutcome::Accepted).await.unwrap_err();
    assert!(matches!(
        err,
        agileplus_domain::ports::TriageError::TicketNotFound(_)
    ));
}
