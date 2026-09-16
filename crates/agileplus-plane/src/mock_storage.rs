//! Minimal mock StoragePort for testing outbound sync functions.

use std::collections::HashMap;
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
