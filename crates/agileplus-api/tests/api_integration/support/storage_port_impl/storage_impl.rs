use std::future::Future;

use agileplus_domain::domain::audit::AuditEntry;
use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures};
use agileplus_domain::domain::epic::{Epic, EpicStatus};
use agileplus_domain::domain::governance::{Evidence, GovernanceContract, PolicyRule};
use agileplus_domain::domain::metric::Metric;
use agileplus_domain::domain::module::{Module, ModuleFeatureTag, ModuleWithFeatures};
use agileplus_domain::domain::project::Project;
use agileplus_domain::domain::story::{Story, StoryStatus};
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_domain::domain::user::{User, UserRole};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::storage::StoragePort;

use super::super::storage::MockStorage;
use super::{audit, cycle, evidence, metrics, module, policy, sync_mapping};

impl StoragePort for MockStorage {
    fn append_audit_entry(
        &self,
        entry: &AuditEntry,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send {
        audit::append_audit_entry(self, entry)
    }

    fn get_audit_trail(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Vec<AuditEntry>, DomainError>> + Send {
        audit::get_audit_trail(self, feature_id)
    }

    fn get_latest_audit_entry(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Option<AuditEntry>, DomainError>> + Send {
        audit::get_latest_audit_entry(self, feature_id)
    }

    fn create_evidence(
        &self,
        e: &Evidence,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send {
        evidence::create_evidence(self, e)
    }

    fn get_evidence_by_wp(
        &self,
        wp_id: i64,
    ) -> impl Future<Output = Result<Vec<Evidence>, DomainError>> + Send {
        evidence::get_evidence_by_wp(self, wp_id)
    }

    fn get_evidence_by_fr(
        &self,
        fr_id: &str,
    ) -> impl Future<Output = Result<Vec<Evidence>, DomainError>> + Send {
        evidence::get_evidence_by_fr(self, fr_id)
    }

    fn create_policy_rule(
        &self,
        r: &PolicyRule,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send {
        policy::create_policy_rule(self, r)
    }

    fn list_active_policies(
        &self,
    ) -> impl Future<Output = Result<Vec<PolicyRule>, DomainError>> + Send {
        policy::list_active_policies(self)
    }

    fn record_metric(&self, m: &Metric) -> impl Future<Output = Result<i64, DomainError>> + Send {
        metrics::record_metric(self, m)
    }

    fn get_metrics_by_feature(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Vec<Metric>, DomainError>> + Send {
        metrics::get_metrics_by_feature(self, feature_id)
    }

    fn create_governance_contract(
        &self,
        _c: &GovernanceContract,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send {
        async move { Ok(1) }
    }

    fn get_governance_contract(
        &self,
        feature_id: i64,
        version: i32,
    ) -> impl Future<Output = Result<Option<GovernanceContract>, DomainError>> + Send {
        let found = self
            .governance
            .lock()
            .expect("governance lock poisoned")
            .iter()
            .find(|c| c.feature_id == feature_id && c.version == version)
            .cloned();
        async move { Ok(found) }
    }

    fn get_latest_governance_contract(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Option<GovernanceContract>, DomainError>> + Send {
        let found = self
            .governance
            .lock()
            .expect("governance lock poisoned")
            .iter()
            .filter(|c| c.feature_id == feature_id)
            .max_by_key(|c| c.version)
            .cloned();
        async move { Ok(found) }
    }

    fn create_module(
        &self,
        module: &Module,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send {
        module::create_module(self, module)
    }

    fn get_module(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Option<Module>, DomainError>> + Send {
        module::get_module(self, id)
    }

    fn get_module_by_slug(
        &self,
        slug: &str,
    ) -> impl Future<Output = Result<Option<Module>, DomainError>> + Send {
        module::get_module_by_slug(self, slug)
    }

    fn update_module(
        &self,
        id: i64,
        friendly_name: &str,
        description: Option<&str>,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        module::update_module(self, id, friendly_name, description)
    }

    fn delete_module(&self, id: i64) -> impl Future<Output = Result<(), DomainError>> + Send {
        module::delete_module(self, id)
    }

    fn list_root_modules(&self) -> impl Future<Output = Result<Vec<Module>, DomainError>> + Send {
        module::list_root_modules(self)
    }

    fn list_child_modules(
        &self,
        parent_id: i64,
    ) -> impl Future<Output = Result<Vec<Module>, DomainError>> + Send {
        module::list_child_modules(self, parent_id)
    }

    fn get_module_with_features(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Option<ModuleWithFeatures>, DomainError>> + Send {
        module::get_module_with_features(self, id)
    }

    fn create_cycle(&self, cycle: &Cycle) -> impl Future<Output = Result<i64, DomainError>> + Send {
        cycle::create_cycle(self, cycle)
    }

    fn get_cycle(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Option<Cycle>, DomainError>> + Send {
        cycle::get_cycle(self, id)
    }

    fn update_cycle_state(
        &self,
        id: i64,
        state: CycleState,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        cycle::update_cycle_state(self, id, state)
    }

    fn list_cycles_by_state(
        &self,
        state: CycleState,
    ) -> impl Future<Output = Result<Vec<Cycle>, DomainError>> + Send {
        cycle::list_cycles_by_state(self, state)
    }

    fn list_cycles_by_module(
        &self,
        module_id: i64,
    ) -> impl Future<Output = Result<Vec<Cycle>, DomainError>> + Send {
        cycle::list_cycles_by_module(self, module_id)
    }

    fn list_all_cycles(&self) -> impl Future<Output = Result<Vec<Cycle>, DomainError>> + Send {
        cycle::list_all_cycles(self)
    }

    fn get_cycle_with_features(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Option<CycleWithFeatures>, DomainError>> + Send {
        cycle::get_cycle_with_features(self, id)
    }

    fn tag_feature_to_module(
        &self,
        tag: &ModuleFeatureTag,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        cycle::tag_feature_to_module(self, tag)
    }

    fn untag_feature_from_module(
        &self,
        module_id: i64,
        feature_id: i64,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        cycle::untag_feature_from_module(self, module_id, feature_id)
    }

    fn add_feature_to_cycle(
        &self,
        entry: &CycleFeature,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        cycle::add_feature_to_cycle(self, entry)
    }

    fn remove_feature_from_cycle(
        &self,
        cycle_id: i64,
        feature_id: i64,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        cycle::remove_feature_from_cycle(self, cycle_id, feature_id)
    }

    fn get_sync_mapping(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> impl Future<Output = Result<Option<SyncMapping>, DomainError>> + Send {
        sync_mapping::get_sync_mapping(self, entity_type, entity_id)
    }

    fn upsert_sync_mapping(
        &self,
        mapping: &SyncMapping,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        sync_mapping::upsert_sync_mapping(self, mapping)
    }

    fn get_sync_mapping_by_plane_id(
        &self,
        entity_type: &str,
        plane_issue_id: &str,
    ) -> impl Future<Output = Result<Option<SyncMapping>, DomainError>> + Send {
        sync_mapping::get_sync_mapping_by_plane_id(self, entity_type, plane_issue_id)
    }

    fn delete_sync_mapping(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        sync_mapping::delete_sync_mapping(self, entity_type, entity_id)
    }

    fn create_project(
        &self,
        project: &Project,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send {
        let mut projects = self.projects.lock().expect("projects lock poisoned");
        let id = (projects.len() as i64) + 1;
        let mut p = project.clone();
        p.id = id;
        projects.push(p);
        async move { Ok(id) }
    }

    fn get_project_by_slug(
        &self,
        slug: &str,
    ) -> impl Future<Output = Result<Option<Project>, DomainError>> + Send {
        let found = self
            .projects
            .lock()
            .expect("projects lock poisoned")
            .iter()
            .find(|p| p.slug == slug)
            .cloned();
        async move { Ok(found) }
    }

    fn list_all_projects(&self) -> impl Future<Output = Result<Vec<Project>, DomainError>> + Send {
        let all = self.projects.lock().expect("projects lock poisoned").clone();
        async move { Ok(all) }
    }

    fn create_epic(&self, epic: &Epic) -> impl Future<Output = Result<i64, DomainError>> + Send {
        let mut epics = self.epics.lock().expect("epics lock poisoned");
        let id = (epics.len() as i64) + 1;
        let mut e = epic.clone();
        e.id = id;
        epics.push(e);
        async move { Ok(id) }
    }

    fn get_epic(&self, id: i64) -> impl Future<Output = Result<Option<Epic>, DomainError>> + Send {
        let found = self.epics.lock().expect("epics lock poisoned").iter().find(|e| e.id == id).cloned();
        async move { Ok(found) }
    }

    fn list_epics_by_project(&self, project_id: i64) -> impl Future<Output = Result<Vec<Epic>, DomainError>> + Send {
        let epics: Vec<Epic> = self.epics.lock().expect("epics lock poisoned").iter().filter(|e| e.project_id == project_id).cloned().collect();
        async move { Ok(epics) }
    }

    fn update_epic_status(&self, id: i64, status: EpicStatus) -> impl Future<Output = Result<(), DomainError>> + Send {
        {
            let mut epics = self.epics.lock().expect("epics lock poisoned");
            if let Some(e) = epics.iter_mut().find(|e| e.id == id) {
                e.status = status;
            }
        }
        async move { Ok(()) }
    }

    fn create_story(&self, story: &Story) -> impl Future<Output = Result<i64, DomainError>> + Send {
        let mut stories = self.stories.lock().expect("stories lock poisoned");
        let id = (stories.len() as i64) + 1;
        let mut s = story.clone();
        s.id = id;
        stories.push(s);
        async move { Ok(id) }
    }

    fn get_story(&self, id: i64) -> impl Future<Output = Result<Option<Story>, DomainError>> + Send {
        let found = self.stories.lock().expect("stories lock poisoned").iter().find(|s| s.id == id).cloned();
        async move { Ok(found) }
    }

    fn list_stories_by_epic(&self, epic_id: i64) -> impl Future<Output = Result<Vec<Story>, DomainError>> + Send {
        let stories: Vec<Story> = self.stories.lock().expect("stories lock poisoned").iter().filter(|s| s.epic_id == epic_id).cloned().collect();
        async move { Ok(stories) }
    }

    fn update_story_status(&self, id: i64, status: StoryStatus) -> impl Future<Output = Result<(), DomainError>> + Send {
        {
            let mut stories = self.stories.lock().expect("stories lock poisoned");
            if let Some(s) = stories.iter_mut().find(|s| s.id == id) {
                s.status = status;
            }
        }
        async move { Ok(()) }
    }

    fn create_user(&self, user: &User) -> impl Future<Output = Result<i64, DomainError>> + Send {
        let mut users = self.users.lock().expect("users lock poisoned");
        let id = (users.len() as i64) + 1;
        let mut u = user.clone();
        u.id = id;
        users.push(u);
        async move { Ok(id) }
    }

    fn get_user(&self, id: i64) -> impl Future<Output = Result<Option<User>, DomainError>> + Send {
        let found = self.users.lock().expect("users lock poisoned").iter().find(|u| u.id == id).cloned();
        async move { Ok(found) }
    }

    fn get_user_by_email(&self, email: &str) -> impl Future<Output = Result<Option<User>, DomainError>> + Send {
        let found = self.users.lock().expect("users lock poisoned").iter().find(|u| u.email == email).cloned();
        async move { Ok(found) }
    }

    fn list_all_users(&self) -> impl Future<Output = Result<Vec<User>, DomainError>> + Send {
        let all = self.users.lock().expect("users lock poisoned").clone();
        async move { Ok(all) }
    }
}