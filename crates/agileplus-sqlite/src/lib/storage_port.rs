use super::SqliteStorageAdapter;
use crate::repository::{
    audit, cycles, epics, evidence, features, governance, metrics, modules, projects, stories,
    sync_mappings, users, work_packages,
};
use agileplus_domain::{
    domain::{
        audit::AuditEntry,
        cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures},
        epic::{Epic, EpicStatus},
        feature::Feature,
        governance::{Evidence, GovernanceContract, PolicyRule},
        metric::Metric,
        module::{Module, ModuleFeatureTag, ModuleWithFeatures},
        project::Project,
        state_machine::FeatureState,
        story::{Story, StoryStatus},
        sync_mapping::SyncMapping,
        user::{User, UserRole, UserStatus},
        work_package::{WorkPackage, WpDependency, WpState},
    },
    error::DomainError,
    ports::StoragePort,
};

#[async_trait::async_trait]
impl StoragePort for SqliteStorageAdapter {
    // -- Feature CRUD --

    async fn create_feature(&self, feature: &Feature) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        features::create_feature(&conn, feature)
    }

    async fn get_feature_by_slug(&self, slug: &str) -> Result<Option<Feature>, DomainError> {
        let conn = self.lock()?;
        features::get_feature_by_slug(&conn, slug)
    }

    async fn get_feature_by_id(&self, id: i64) -> Result<Option<Feature>, DomainError> {
        let conn = self.lock()?;
        features::get_feature_by_id(&conn, id)
    }

    async fn update_feature_state(&self, id: i64, state: FeatureState) -> Result<(), DomainError> {
        let conn = self.lock()?;
        features::update_feature_state(&conn, id, state)
    }

    async fn update_feature(&self, feature: &Feature) -> Result<(), DomainError> {
        let conn = self.lock()?;
        features::update_feature(&conn, feature)
    }

    async fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> Result<Vec<Feature>, DomainError> {
        let conn = self.lock()?;
        features::list_features_by_state(&conn, state)
    }

    async fn list_all_features(&self) -> Result<Vec<Feature>, DomainError> {
        let conn = self.lock()?;
        features::list_all_features(&conn)
    }

    async fn list_features_by_label(&self, label: &str) -> Result<Vec<Feature>, DomainError> {
        let conn = self.lock()?;
        features::list_features_by_label(&conn, label)
    }

    // -- Work Package CRUD --

    async fn create_work_package(&self, wp: &WorkPackage) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        work_packages::create_work_package(&conn, wp)
    }

    async fn get_work_package(&self, id: i64) -> Result<Option<WorkPackage>, DomainError> {
        let conn = self.lock()?;
        work_packages::get_work_package(&conn, id)
    }

    async fn update_wp_state(&self, id: i64, state: WpState) -> Result<(), DomainError> {
        let conn = self.lock()?;
        work_packages::update_wp_state(&conn, id, state)
    }

    async fn list_wps_by_feature(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        let conn = self.lock()?;
        work_packages::list_wps_by_feature(&conn, feature_id)
    }

    async fn add_wp_dependency(&self, dep: &WpDependency) -> Result<(), DomainError> {
        let conn = self.lock()?;
        work_packages::add_wp_dependency(&conn, dep)
    }

    async fn get_wp_dependencies(&self, wp_id: i64) -> Result<Vec<WpDependency>, DomainError> {
        let conn = self.lock()?;
        work_packages::get_wp_dependencies(&conn, wp_id)
    }

    async fn get_ready_wps(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        let conn = self.lock()?;
        work_packages::get_ready_wps(&conn, feature_id)
    }

    async fn create_work_package_for_story(
        &self,
        story_id: i64,
        wp: &WorkPackage,
    ) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        work_packages::create_work_package_for_story(&conn, story_id, wp)
    }

    async fn list_wps_by_story(&self, story_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        let conn = self.lock()?;
        work_packages::list_wps_by_story(&conn, story_id)
    }

    async fn list_all_work_packages(&self) -> Result<Vec<WorkPackage>, DomainError> {
        let conn = self.lock()?;
        work_packages::list_all_work_packages(&conn)
    }

    async fn get_next_ready_wps(
        &self,
        cycle: Option<i64>,
    ) -> Result<Vec<WorkPackage>, DomainError> {
        let conn = self.lock()?;
        work_packages::get_next_ready_wps(&conn, cycle)
    }

    async fn add_story_to_cycle(&self, cycle_id: i64, story_id: i64) -> Result<(), DomainError> {
        let _ = (cycle_id, story_id);
        Err(DomainError::NotImplemented)
    }

    // -- Audit CRUD --

    async fn append_audit_entry(&self, entry: &AuditEntry) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        audit::append_audit_entry(&conn, entry)
    }

    async fn get_audit_trail(&self, feature_id: i64) -> Result<Vec<AuditEntry>, DomainError> {
        let conn = self.lock()?;
        audit::get_audit_trail(&conn, feature_id)
    }

    async fn get_latest_audit_entry(
        &self,
        feature_id: i64,
    ) -> Result<Option<AuditEntry>, DomainError> {
        let conn = self.lock()?;
        audit::get_latest_audit_entry(&conn, feature_id)
    }

    // -- Evidence + Policy + Metric CRUD --

    async fn create_evidence(&self, ev: &Evidence) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        evidence::create_evidence(&conn, ev)
    }

    async fn get_evidence_by_wp(&self, wp_id: i64) -> Result<Vec<Evidence>, DomainError> {
        let conn = self.lock()?;
        evidence::get_evidence_by_wp(&conn, wp_id)
    }

    async fn get_evidence_by_fr(&self, fr_id: &str) -> Result<Vec<Evidence>, DomainError> {
        let conn = self.lock()?;
        evidence::get_evidence_by_fr(&conn, fr_id)
    }

    async fn create_policy_rule(&self, rule: &PolicyRule) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        governance::create_policy_rule(&conn, rule)
    }

    async fn list_active_policies(&self) -> Result<Vec<PolicyRule>, DomainError> {
        let conn = self.lock()?;
        governance::list_active_policies(&conn)
    }

    async fn record_metric(&self, metric: &Metric) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        metrics::record_metric(&conn, metric)
    }

    async fn get_metrics_by_feature(&self, feature_id: i64) -> Result<Vec<Metric>, DomainError> {
        let conn = self.lock()?;
        metrics::get_metrics_by_feature(&conn, feature_id)
    }

    // -- Governance --

    async fn create_governance_contract(
        &self,
        contract: &GovernanceContract,
    ) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        governance::create_governance_contract(&conn, contract)
    }

    async fn get_governance_contract(
        &self,
        feature_id: i64,
        version: i32,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        let conn = self.lock()?;
        governance::get_governance_contract(&conn, feature_id, version)
    }

    async fn get_latest_governance_contract(
        &self,
        feature_id: i64,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        let conn = self.lock()?;
        governance::get_latest_governance_contract(&conn, feature_id)
    }

    // -- Module CRUD (T007) --

    async fn create_module(&self, module: &Module) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        modules::create_module(&conn, module)
    }

    async fn get_module(&self, id: i64) -> Result<Option<Module>, DomainError> {
        let conn = self.lock()?;
        modules::get_module(&conn, id)
    }

    async fn get_module_by_slug(&self, slug: &str) -> Result<Option<Module>, DomainError> {
        let conn = self.lock()?;
        modules::get_module_by_slug(&conn, slug)
    }

    async fn update_module(
        &self,
        id: i64,
        friendly_name: &str,
        description: Option<&str>,
    ) -> Result<(), DomainError> {
        let conn = self.lock()?;
        modules::update_module(&conn, id, friendly_name, description)
    }

    async fn delete_module(&self, id: i64) -> Result<(), DomainError> {
        let conn = self.lock()?;
        modules::delete_module(&conn, id)
    }

    async fn list_root_modules(&self) -> Result<Vec<Module>, DomainError> {
        let conn = self.lock()?;
        modules::list_root_modules(&conn)
    }

    async fn list_child_modules(&self, parent_id: i64) -> Result<Vec<Module>, DomainError> {
        let conn = self.lock()?;
        modules::list_child_modules(&conn, parent_id)
    }

    async fn get_module_with_features(
        &self,
        id: i64,
    ) -> Result<Option<ModuleWithFeatures>, DomainError> {
        let conn = self.lock()?;
        modules::get_module_with_features(&conn, id)
    }

    // -- Cycle CRUD (T008) --

    async fn create_cycle(&self, cycle: &Cycle) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        cycles::create_cycle(&conn, cycle)
    }

    async fn get_cycle(&self, id: i64) -> Result<Option<Cycle>, DomainError> {
        let conn = self.lock()?;
        cycles::get_cycle(&conn, id)
    }

    async fn update_cycle_state(&self, id: i64, state: CycleState) -> Result<(), DomainError> {
        let conn = self.lock()?;
        cycles::update_cycle_state(&conn, id, state)
    }

    async fn list_cycles_by_state(&self, state: CycleState) -> Result<Vec<Cycle>, DomainError> {
        let conn = self.lock()?;
        cycles::list_cycles_by_state(&conn, state)
    }

    async fn list_cycles_by_module(&self, module_id: i64) -> Result<Vec<Cycle>, DomainError> {
        let conn = self.lock()?;
        cycles::list_cycles_by_module(&conn, module_id)
    }

    async fn list_all_cycles(&self) -> Result<Vec<Cycle>, DomainError> {
        let conn = self.lock()?;
        cycles::list_all_cycles(&conn)
    }

    async fn get_cycle_with_features(
        &self,
        id: i64,
    ) -> Result<Option<CycleWithFeatures>, DomainError> {
        let conn = self.lock()?;
        cycles::get_cycle_with_features(&conn, id)
    }

    // -- Join table ops (T009) --

    async fn tag_feature_to_module(&self, tag: &ModuleFeatureTag) -> Result<(), DomainError> {
        let conn = self.lock()?;
        modules::tag_feature_to_module(&conn, tag)
    }

    async fn untag_feature_from_module(
        &self,
        module_id: i64,
        feature_id: i64,
    ) -> Result<(), DomainError> {
        let conn = self.lock()?;
        modules::untag_feature_from_module(&conn, module_id, feature_id)
    }

    async fn add_feature_to_cycle(&self, entry: &CycleFeature) -> Result<(), DomainError> {
        let conn = self.lock()?;
        cycles::add_feature_to_cycle(&conn, entry)
    }

    async fn remove_feature_from_cycle(
        &self,
        cycle_id: i64,
        feature_id: i64,
    ) -> Result<(), DomainError> {
        let conn = self.lock()?;
        cycles::remove_feature_from_cycle(&conn, cycle_id, feature_id)
    }

    // -- Sync Mapping CRUD (WP06-T033) --

    async fn get_sync_mapping(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Option<SyncMapping>, DomainError> {
        let conn = self.lock()?;
        sync_mappings::get_sync_mapping(&conn, entity_type, entity_id)
    }

    async fn upsert_sync_mapping(&self, mapping: &SyncMapping) -> Result<(), DomainError> {
        let conn = self.lock()?;
        sync_mappings::upsert_sync_mapping(&conn, mapping)
    }

    async fn get_sync_mapping_by_plane_id(
        &self,
        entity_type: &str,
        plane_issue_id: &str,
    ) -> Result<Option<SyncMapping>, DomainError> {
        let conn = self.lock()?;
        sync_mappings::get_sync_mapping_by_plane_id(&conn, entity_type, plane_issue_id)
    }

    async fn delete_sync_mapping(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<(), DomainError> {
        let conn = self.lock()?;
        sync_mappings::delete_sync_mapping(&conn, entity_type, entity_id)
    }

    async fn create_project(&self, project: &Project) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        projects::create_project(&conn, project)
    }

    async fn get_project_by_slug(&self, slug: &str) -> Result<Option<Project>, DomainError> {
        let conn = self.lock()?;
        projects::get_project_by_slug(&conn, slug)
    }

    async fn get_project_by_id(&self, id: i64) -> Result<Option<Project>, DomainError> {
        let conn = self.lock()?;
        projects::get_project_by_id(&conn, id)
    }

    async fn list_all_projects(&self) -> Result<Vec<Project>, DomainError> {
        let conn = self.lock()?;
        projects::list_all_projects(&conn)
    }

    async fn delete_project(&self, id: i64) -> Result<(), DomainError> {
        let conn = self.lock()?;
        projects::delete_project(&conn, id)
    }

    async fn create_user(&self, user: &User) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        users::create_user(&conn, user)
    }

    async fn get_user(&self, id: i64) -> Result<Option<User>, DomainError> {
        let conn = self.lock()?;
        users::get_user_by_id(&conn, id)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let conn = self.lock()?;
        users::get_user_by_email(&conn, email)
    }

    async fn update_user_status(&self, id: i64, status: UserStatus) -> Result<(), DomainError> {
        let conn = self.lock()?;
        users::update_user_status(&conn, id, status)
    }

    async fn update_user_role(&self, id: i64, role: UserRole) -> Result<(), DomainError> {
        let conn = self.lock()?;
        users::update_user_role(&conn, id, role)
    }

    async fn list_all_users(&self) -> Result<Vec<User>, DomainError> {
        let conn = self.lock()?;
        users::list_all_users(&conn)
    }

    async fn delete_user(&self, id: i64) -> Result<(), DomainError> {
        let conn = self.lock()?;
        users::delete_user(&conn, id)
    }

    async fn create_epic(&self, epic: &Epic) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        epics::create_epic(&conn, epic)
    }

    async fn get_epic(&self, id: i64) -> Result<Option<Epic>, DomainError> {
        let conn = self.lock()?;
        epics::get_epic_by_id(&conn, id)
    }

    async fn update_epic_status(&self, id: i64, status: EpicStatus) -> Result<(), DomainError> {
        let conn = self.lock()?;
        epics::update_epic_status(&conn, id, status)
    }

    async fn list_epics_by_project(&self, project_id: i64) -> Result<Vec<Epic>, DomainError> {
        let conn = self.lock()?;
        epics::list_epics_by_project(&conn, project_id)
    }

    async fn delete_epic(&self, id: i64) -> Result<(), DomainError> {
        let conn = self.lock()?;
        epics::delete_epic(&conn, id)
    }

    async fn create_story(&self, story: &Story) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        stories::create_story(&conn, story)
    }

    async fn get_story(&self, id: i64) -> Result<Option<Story>, DomainError> {
        let conn = self.lock()?;
        stories::get_story_by_id(&conn, id)
    }

    async fn update_story_status(&self, id: i64, status: StoryStatus) -> Result<(), DomainError> {
        let conn = self.lock()?;
        stories::update_story_status(&conn, id, status)
    }

    async fn list_stories_by_epic(&self, epic_id: i64) -> Result<Vec<Story>, DomainError> {
        let conn = self.lock()?;
        stories::list_stories_by_epic(&conn, epic_id)
    }

    async fn list_stories_by_project(&self, project_id: i64) -> Result<Vec<Story>, DomainError> {
        let conn = self.lock()?;
        stories::list_stories_by_project(&conn, project_id)
    }

    async fn delete_story(&self, id: i64) -> Result<(), DomainError> {
        let conn = self.lock()?;
        stories::delete_story(&conn, id)
    }

    async fn upsert_story_by_requirement_id(&self, _story: &Story) -> Result<i64, DomainError> {
        Err(DomainError::NotImplemented)
    }
}
