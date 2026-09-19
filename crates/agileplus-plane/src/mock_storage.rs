//! Minimal mock StoragePort for testing outbound sync functions.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use async_trait::async_trait;
use agileplus_domain::domain::audit::AuditEntry;
use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures};
use agileplus_domain::domain::epic::{Epic, EpicStatus};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::governance::{Evidence, GovernanceContract, PolicyRule};
use agileplus_domain::domain::metric::Metric;
use agileplus_domain::domain::module::{Module, ModuleFeatureTag, ModuleWithFeatures};
use agileplus_domain::domain::project::Project;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::story::{Story, StoryStatus};
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_domain::domain::user::User;
use agileplus_domain::domain::work_package::{WorkPackage, WpDependency, WpState};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::StoragePort;

#[derive(Default)]
pub struct MockStoragePort {
    pub sync_mappings: Mutex<HashMap<(String, i64), SyncMapping>>,
    pub modules: Mutex<Vec<Module>>,
    pub cycles: Mutex<Vec<Cycle>>,
    /// When set, `list_root_modules` fails. Lets tests drive the daemon's
    /// tick-failure path without a bespoke `StoragePort` implementation.
    pub fail_list_modules: AtomicBool,
    /// When set, `upsert_sync_mapping` fails. Lets tests drive the outbound
    /// push functions' mapping-write error paths.
    pub fail_upsert_mapping: AtomicBool,
}

impl MockStoragePort {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_sync_mapping(mut self, entity_type: &str, entity_id: i64, plane_id: &str) -> Self {
        self.sync_mappings
            .get_mut()
            .unwrap()
            .insert(
                (entity_type.to_string(), entity_id),
                SyncMapping::new(entity_type, entity_id, plane_id, ""),
            );
        self
    }

    /// Make every subsequent `list_root_modules` call fail.
    pub fn failing_list_modules(self) -> Self {
        self.fail_list_modules.store(true, Ordering::SeqCst);
        self
    }

    /// Make every subsequent `upsert_sync_mapping` call fail.
    pub fn failing_upsert_mapping(self) -> Self {
        self.fail_upsert_mapping.store(true, Ordering::SeqCst);
        self
    }
}

#[async_trait]
impl StoragePort for MockStoragePort {
    // -- Feature CRUD (stubs) --
    async fn create_feature(&self, _: &Feature) -> Result<i64, DomainError> { Ok(1) }
    async fn get_feature_by_slug(&self, _: &str) -> Result<Option<Feature>, DomainError> { Ok(None) }
    async fn get_feature_by_id(&self, _: i64) -> Result<Option<Feature>, DomainError> { Ok(None) }
    async fn update_feature_state(&self, _: i64, _: FeatureState) -> Result<(), DomainError> { Ok(()) }
    async fn list_features_by_state(&self, _: FeatureState) -> Result<Vec<Feature>, DomainError> { Ok(vec![]) }
    async fn list_all_features(&self) -> Result<Vec<Feature>, DomainError> { Ok(vec![]) }

    // -- Work Package CRUD (stubs) --
    async fn create_work_package(&self, _: &WorkPackage) -> Result<i64, DomainError> { Ok(1) }
    async fn get_work_package(&self, _: i64) -> Result<Option<WorkPackage>, DomainError> { Ok(None) }
    async fn update_wp_state(&self, _: i64, _: WpState) -> Result<(), DomainError> { Ok(()) }
    async fn list_wps_by_feature(&self, _: i64) -> Result<Vec<WorkPackage>, DomainError> { Ok(vec![]) }
    async fn add_wp_dependency(&self, _: &WpDependency) -> Result<(), DomainError> { Ok(()) }
    async fn get_wp_dependencies(&self, _: i64) -> Result<Vec<WpDependency>, DomainError> { Ok(vec![]) }
    async fn get_ready_wps(&self, _: i64) -> Result<Vec<WorkPackage>, DomainError> { Ok(vec![]) }

    // -- Audit (stubs) --
    async fn append_audit_entry(&self, _: &AuditEntry) -> Result<i64, DomainError> { Ok(1) }
    async fn get_audit_trail(&self, _: i64) -> Result<Vec<AuditEntry>, DomainError> { Ok(vec![]) }
    async fn get_latest_audit_entry(&self, _: i64) -> Result<Option<AuditEntry>, DomainError> { Ok(None) }

    // -- Evidence (stubs) --
    async fn create_evidence(&self, _: &Evidence) -> Result<i64, DomainError> { Ok(1) }
    async fn get_evidence_by_wp(&self, _: i64) -> Result<Vec<Evidence>, DomainError> { Ok(vec![]) }
    async fn get_evidence_by_fr(&self, _: &str) -> Result<Vec<Evidence>, DomainError> { Ok(vec![]) }

    // -- Policy (stubs) --
    async fn create_policy_rule(&self, _: &PolicyRule) -> Result<i64, DomainError> { Ok(1) }
    async fn list_active_policies(&self) -> Result<Vec<PolicyRule>, DomainError> { Ok(vec![]) }
    async fn record_metric(&self, _: &Metric) -> Result<i64, DomainError> { Ok(1) }
    async fn get_metrics_by_feature(&self, _: i64) -> Result<Vec<Metric>, DomainError> { Ok(vec![]) }

    // -- Governance (stubs) --
    async fn create_governance_contract(&self, _: &GovernanceContract) -> Result<i64, DomainError> { Ok(1) }
    async fn get_governance_contract(&self, _: i64, _: i32) -> Result<Option<GovernanceContract>, DomainError> { Ok(None) }
    async fn get_latest_governance_contract(&self, _: i64) -> Result<Option<GovernanceContract>, DomainError> { Ok(None) }

    // -- Module CRUD --
    async fn create_module(&self, m: &Module) -> Result<i64, DomainError> {
        let mut modules = self.modules.lock().unwrap();
        let id = modules.len() as i64 + 1;
        modules.push(Module { id, ..m.clone() });
        Ok(id)
    }
    async fn get_module(&self, id: i64) -> Result<Option<Module>, DomainError> {
        Ok(self.modules.lock().unwrap().iter().find(|m| m.id == id).cloned())
    }
    async fn get_module_by_slug(&self, _: &str) -> Result<Option<Module>, DomainError> { Ok(None) }
    async fn update_module(&self, _: i64, _: &str, _: Option<&str>) -> Result<(), DomainError> { Ok(()) }
    async fn delete_module(&self, _: i64) -> Result<(), DomainError> { Ok(()) }
    async fn list_root_modules(&self) -> Result<Vec<Module>, DomainError> {
        if self.fail_list_modules.load(Ordering::SeqCst) {
            return Err(DomainError::Storage(
                "mock: list_root_modules failed".to_string(),
            ));
        }
        Ok(self.modules.lock().unwrap().clone())
    }
    async fn list_child_modules(&self, _: i64) -> Result<Vec<Module>, DomainError> { Ok(vec![]) }
    async fn get_module_with_features(&self, _: i64) -> Result<Option<ModuleWithFeatures>, DomainError> { Ok(None) }
    async fn tag_feature_to_module(&self, _: &ModuleFeatureTag) -> Result<(), DomainError> { Ok(()) }
    async fn untag_feature_from_module(&self, _: i64, _: i64) -> Result<(), DomainError> { Ok(()) }

    // -- Cycle CRUD --
    async fn create_cycle(&self, c: &Cycle) -> Result<i64, DomainError> {
        let mut cycles = self.cycles.lock().unwrap();
        let id = cycles.len() as i64 + 1;
        cycles.push(Cycle { id, ..c.clone() });
        Ok(id)
    }
    async fn get_cycle(&self, id: i64) -> Result<Option<Cycle>, DomainError> {
        Ok(self.cycles.lock().unwrap().iter().find(|c| c.id == id).cloned())
    }
    async fn update_cycle_state(&self, _: i64, _: CycleState) -> Result<(), DomainError> { Ok(()) }
    async fn list_cycles_by_state(&self, _: CycleState) -> Result<Vec<Cycle>, DomainError> { Ok(vec![]) }
    async fn list_cycles_by_module(&self, _: i64) -> Result<Vec<Cycle>, DomainError> { Ok(vec![]) }
    async fn list_all_cycles(&self) -> Result<Vec<Cycle>, DomainError> {
        Ok(self.cycles.lock().unwrap().clone())
    }
    async fn get_cycle_with_features(&self, _: i64) -> Result<Option<CycleWithFeatures>, DomainError> { Ok(None) }
    async fn add_feature_to_cycle(&self, _: &CycleFeature) -> Result<(), DomainError> { Ok(()) }
    async fn remove_feature_from_cycle(&self, _: i64, _: i64) -> Result<(), DomainError> { Ok(()) }

    // -- Sync Mapping (real implementation backed by HashMap) --
    async fn get_sync_mapping(&self, entity_type: &str, entity_id: i64) -> Result<Option<SyncMapping>, DomainError> {
        let mappings = self.sync_mappings.lock().unwrap();
        Ok(mappings.get(&(entity_type.to_string(), entity_id)).cloned())
    }
    async fn upsert_sync_mapping(&self, mapping: &SyncMapping) -> Result<(), DomainError> {
        if self.fail_upsert_mapping.load(Ordering::SeqCst) {
            return Err(DomainError::Storage(
                "mock: upsert_sync_mapping failed".to_string(),
            ));
        }
        let mut mappings = self.sync_mappings.lock().unwrap();
        mappings.insert(
            (mapping.entity_type.clone(), mapping.entity_id),
            mapping.clone(),
        );
        Ok(())
    }
    async fn get_sync_mapping_by_plane_id(
        &self,
        entity_type: &str,
        plane_issue_id: &str,
    ) -> Result<Option<SyncMapping>, DomainError> {
        let mappings = self.sync_mappings.lock().unwrap();
        Ok(mappings
            .values()
            .find(|m| m.entity_type == entity_type && m.plane_issue_id == plane_issue_id)
            .cloned())
    }
    async fn delete_sync_mapping(&self, entity_type: &str, entity_id: i64) -> Result<(), DomainError> {
        let mut mappings = self.sync_mappings.lock().unwrap();
        mappings.remove(&(entity_type.to_string(), entity_id));
        Ok(())
    }

    // -- Project CRUD (stubs) --
    async fn create_project(&self, _: &Project) -> Result<i64, DomainError> { Ok(1) }
    async fn get_project_by_slug(&self, _: &str) -> Result<Option<Project>, DomainError> { Ok(None) }
    async fn list_all_projects(&self) -> Result<Vec<Project>, DomainError> { Ok(vec![]) }

    // -- Epic CRUD (stubs) --
    async fn create_epic(&self, _: &Epic) -> Result<i64, DomainError> { Ok(1) }
    async fn get_epic(&self, _: i64) -> Result<Option<Epic>, DomainError> { Ok(None) }
    async fn list_epics_by_project(&self, _: i64) -> Result<Vec<Epic>, DomainError> { Ok(vec![]) }
    async fn update_epic_status(&self, _: i64, _: EpicStatus) -> Result<(), DomainError> { Ok(()) }

    // -- Story CRUD (stubs) --
    async fn create_story(&self, _: &Story) -> Result<i64, DomainError> { Ok(1) }
    async fn get_story(&self, _: i64) -> Result<Option<Story>, DomainError> { Ok(None) }
    async fn list_stories_by_epic(&self, _: i64) -> Result<Vec<Story>, DomainError> { Ok(vec![]) }
    async fn update_story_status(&self, _: i64, _: StoryStatus) -> Result<(), DomainError> { Ok(()) }

    // -- User CRUD (stubs) --
    async fn create_user(&self, _: &User) -> Result<i64, DomainError> { Ok(1) }
    async fn get_user(&self, _: i64) -> Result<Option<User>, DomainError> { Ok(None) }
    async fn get_user_by_email(&self, _: &str) -> Result<Option<User>, DomainError> { Ok(None) }
    async fn list_all_users(&self) -> Result<Vec<User>, DomainError> { Ok(vec![]) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn module(name: &str) -> Module {
        Module::new(name, None)
    }

    fn cycle(name: &str) -> Cycle {
        Cycle {
            id: 0,
            name: name.to_string(),
            description: Some("desc".to_string()),
            state: CycleState::Active,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
            module_scope_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn create_module_assigns_incrementing_ids() {
        let store = MockStoragePort::new();
        let a = store.create_module(&module("A")).await.unwrap();
        let b = store.create_module(&module("B")).await.unwrap();
        assert_eq!((a, b), (1, 2));
    }

    #[tokio::test]
    async fn get_module_returns_stored_module() {
        let store = MockStoragePort::new();
        let id = store.create_module(&module("Auth")).await.unwrap();
        let got = store.get_module(id).await.unwrap().unwrap();
        assert_eq!(got.friendly_name, "Auth");
    }

    #[tokio::test]
    async fn get_module_missing_returns_none() {
        let store = MockStoragePort::new();
        assert!(store.get_module(42).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn update_and_delete_module_are_noops_but_ok() {
        let store = MockStoragePort::new();
        store.update_module(1, "x", Some("d")).await.unwrap();
        store.delete_module(1).await.unwrap();
    }

    #[tokio::test]
    async fn list_root_modules_returns_all() {
        let store = MockStoragePort::new();
        store.create_module(&module("A")).await.unwrap();
        store.create_module(&module("B")).await.unwrap();
        assert_eq!(store.list_root_modules().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn create_cycle_assigns_incrementing_ids() {
        let store = MockStoragePort::new();
        let a = store.create_cycle(&cycle("S1")).await.unwrap();
        let b = store.create_cycle(&cycle("S2")).await.unwrap();
        assert_eq!((a, b), (1, 2));
    }

    #[tokio::test]
    async fn get_cycle_returns_stored_cycle() {
        let store = MockStoragePort::new();
        let id = store.create_cycle(&cycle("Sprint")).await.unwrap();
        let got = store.get_cycle(id).await.unwrap().unwrap();
        assert_eq!(got.name, "Sprint");
    }

    #[tokio::test]
    async fn get_cycle_missing_returns_none() {
        let store = MockStoragePort::new();
        assert!(store.get_cycle(7).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn list_all_cycles_returns_all() {
        let store = MockStoragePort::new();
        store.create_cycle(&cycle("A")).await.unwrap();
        store.create_cycle(&cycle("B")).await.unwrap();
        assert_eq!(store.list_all_cycles().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn with_sync_mapping_preseed_is_readable() {
        let store = MockStoragePort::new().with_sync_mapping("module", 1, "plane-1");
        let got = store.get_sync_mapping("module", 1).await.unwrap().unwrap();
        assert_eq!(got.plane_issue_id, "plane-1");
    }

    #[tokio::test]
    async fn upsert_sync_mapping_overwrites() {
        let store = MockStoragePort::new();
        store
            .upsert_sync_mapping(&SyncMapping::new("feature", 5, "p-1", "h1"))
            .await
            .unwrap();
        store
            .upsert_sync_mapping(&SyncMapping::new("feature", 5, "p-2", "h2"))
            .await
            .unwrap();
        let got = store.get_sync_mapping("feature", 5).await.unwrap().unwrap();
        assert_eq!(got.plane_issue_id, "p-2");
    }

    #[tokio::test]
    async fn get_sync_mapping_by_plane_id_finds_match() {
        let store = MockStoragePort::new();
        store
            .upsert_sync_mapping(&SyncMapping::new("cycle", 9, "plane-xyz", "h"))
            .await
            .unwrap();
        let found = store
            .get_sync_mapping_by_plane_id("cycle", "plane-xyz")
            .await
            .unwrap();
        assert_eq!(found.unwrap().entity_id, 9);
    }

    #[tokio::test]
    async fn get_sync_mapping_by_plane_id_respects_type() {
        let store = MockStoragePort::new();
        store
            .upsert_sync_mapping(&SyncMapping::new("cycle", 9, "plane-xyz", "h"))
            .await
            .unwrap();
        assert!(store
            .get_sync_mapping_by_plane_id("module", "plane-xyz")
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn delete_sync_mapping_removes_entry() {
        let store = MockStoragePort::new().with_sync_mapping("module", 1, "p");
        store.delete_sync_mapping("module", 1).await.unwrap();
        assert!(store.get_sync_mapping("module", 1).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn stub_crud_methods_return_defaults() {
        let store = MockStoragePort::new();
        assert_eq!(store.create_feature(&Feature::new("s", "n", [0u8; 32], None)).await.unwrap(), 1);
        assert!(store.list_all_features().await.unwrap().is_empty());
        assert!(store.list_all_projects().await.unwrap().is_empty());
        assert!(store.list_all_users().await.unwrap().is_empty());
    }

    /// The write-side stubs that no other test calls.
    ///
    /// They only exist to satisfy `StoragePort` without a database, but they
    /// are part of the port the daemon and outbound tests build on: if one of
    /// them started returning `Err` (or stopped being reachable at all) the
    /// dependent suites would fail for an unrelated-looking reason.
    #[tokio::test]
    async fn write_only_stubs_report_success() {
        use agileplus_domain::domain::governance::{
            EvidenceType, GovernanceRule, PolicyCheck, PolicyDefinition, PolicyDomain,
        };
        use agileplus_domain::domain::user::UserRole;
        use agileplus_domain::domain::work_package::DependencyType;

        let store = MockStoragePort::new();
        let now = chrono::Utc::now();

        assert_eq!(
            store
                .create_work_package(&WorkPackage::new(1, "WP01", 1, "criteria"))
                .await
                .unwrap(),
            1
        );
        store
            .add_wp_dependency(&WpDependency {
                wp_id: 1,
                depends_on: 2,
                dep_type: DependencyType::Explicit,
            })
            .await
            .unwrap();
        assert_eq!(
            store
                .create_evidence(&Evidence {
                    id: 0,
                    wp_id: 1,
                    fr_id: "FR-051".to_string(),
                    evidence_type: EvidenceType::TestResult,
                    artifact_path: "target/test.log".to_string(),
                    metadata: None,
                    created_at: now,
                })
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            store
                .create_policy_rule(&PolicyRule {
                    id: 0,
                    domain: PolicyDomain::Quality,
                    rule: PolicyDefinition {
                        description: "coverage floor".to_string(),
                        check: PolicyCheck::ThresholdMet {
                            metric: "line_coverage".to_string(),
                            min: 0.85,
                        },
                    },
                    active: true,
                    created_at: now,
                    updated_at: now,
                })
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            store
                .record_metric(&Metric {
                    id: 0,
                    feature_id: Some(1),
                    command: "cargo test".to_string(),
                    duration_ms: 10,
                    agent_runs: 1,
                    review_cycles: 0,
                    metadata: None,
                    timestamp: now,
                })
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            store
                .create_governance_contract(&GovernanceContract {
                    id: 0,
                    feature_id: 1,
                    version: 1,
                    rules: vec![GovernanceRule {
                        transition: "created->specified".to_string(),
                        required_evidence: vec!["test_result".to_string()],
                        policy_refs: vec![1],
                    }],
                    bound_at: now,
                })
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            store
                .create_project(&Project::new("Alpha", "alpha").unwrap())
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            store
                .create_epic(&Epic::new(1, "Epic").unwrap())
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            store
                .create_story(&Story::new(1, 1, "Story", Some(1)).unwrap())
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            store
                .create_user(&User::new("Ada", "ada@example.com", UserRole::Admin).unwrap())
                .await
                .unwrap(),
            1
        );
    }

    // -- stub contract for the read/query half of StoragePort --
    //
    // The outbound and daemon tests depend on these neutral defaults, so they
    // are pinned here rather than discovered through an unrelated failure.

    #[tokio::test]
    async fn feature_queries_return_neutral_defaults() {
        use agileplus_domain::domain::state_machine::FeatureState;

        let store = MockStoragePort::new();
        assert!(store.get_feature_by_slug("nope").await.unwrap().is_none());
        assert!(store.get_feature_by_id(1).await.unwrap().is_none());
        assert!(store.list_features_by_state(FeatureState::Created).await.unwrap().is_empty());
        store.update_feature_state(1, FeatureState::Created).await.unwrap();
    }

    #[tokio::test]
    async fn work_package_queries_return_neutral_defaults() {
        use agileplus_domain::domain::work_package::WpState;

        let store = MockStoragePort::new();
        assert!(store.get_work_package(1).await.unwrap().is_none());
        assert!(store.list_wps_by_feature(1).await.unwrap().is_empty());
        assert!(store.get_wp_dependencies(1).await.unwrap().is_empty());
        assert!(store.get_ready_wps(1).await.unwrap().is_empty());
        store.update_wp_state(1, WpState::Planned).await.unwrap();
    }

    #[tokio::test]
    async fn audit_evidence_and_policy_queries_return_neutral_defaults() {
        let entry = AuditEntry {
            id: 0,
            feature_id: 1,
            wp_id: None,
            timestamp: chrono::Utc::now(),
            actor: "test".to_string(),
            transition: "created".to_string(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };

        let store = MockStoragePort::new();
        assert_eq!(store.append_audit_entry(&entry).await.unwrap(), 1);
        assert!(store.get_audit_trail(1).await.unwrap().is_empty());
        assert!(store.get_latest_audit_entry(1).await.unwrap().is_none());
        assert_eq!(store.get_evidence_by_wp(1).await.unwrap().len(), 0);
        assert_eq!(store.get_evidence_by_fr("FR-1").await.unwrap().len(), 0);
        assert!(store.list_active_policies().await.unwrap().is_empty());
        assert!(store.get_metrics_by_feature(1).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn governance_queries_return_none() {
        let store = MockStoragePort::new();
        assert!(store.get_governance_contract(1, 1).await.unwrap().is_none());
        assert!(store.get_latest_governance_contract(1).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn module_child_queries_return_neutral_defaults() {
        let store = MockStoragePort::new();
        assert!(store.get_module_by_slug("nope").await.unwrap().is_none());
        assert!(store.list_child_modules(1).await.unwrap().is_empty());
        assert!(store.get_module_with_features(1).await.unwrap().is_none());
        store.tag_feature_to_module(&ModuleFeatureTag::new(1, 2)).await.unwrap();
        store.untag_feature_from_module(1, 2).await.unwrap();
    }

    #[tokio::test]
    async fn cycle_queries_return_neutral_defaults() {
        let store = MockStoragePort::new();
        assert!(store.get_cycle_with_features(1).await.unwrap().is_none());
        assert!(store.list_cycles_by_module(1).await.unwrap().is_empty());
        assert!(store.list_cycles_by_state(CycleState::Active).await.unwrap().is_empty());
        store.update_cycle_state(1, CycleState::Review).await.unwrap();
        store.add_feature_to_cycle(&CycleFeature::new(1, 2)).await.unwrap();
        store.remove_feature_from_cycle(1, 2).await.unwrap();
    }

    #[tokio::test]
    async fn project_epic_story_and_user_queries_return_neutral_defaults() {
        use agileplus_domain::domain::epic::EpicStatus;
        use agileplus_domain::domain::story::StoryStatus;

        let store = MockStoragePort::new();
        assert!(store.get_project_by_slug("nope").await.unwrap().is_none());
        assert!(store.get_epic(1).await.unwrap().is_none());
        assert!(store.list_epics_by_project(1).await.unwrap().is_empty());
        store.update_epic_status(1, EpicStatus::Active).await.unwrap();
        assert!(store.get_story(1).await.unwrap().is_none());
        assert!(store.list_stories_by_epic(1).await.unwrap().is_empty());
        store.update_story_status(1, StoryStatus::Todo).await.unwrap();
        assert!(store.get_user(1).await.unwrap().is_none());
        assert!(store.get_user_by_email("nobody@example.com").await.unwrap().is_none());
    }
}
