use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use agileplus_domain::domain::{
    cycle::CycleState,
    module::Module,
    state_machine::FeatureState,
    work_package::{PrState, WpState},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportBundle {
    #[serde(default)]
    pub projects: Vec<ImportProject>,
    #[serde(default)]
    pub modules: Vec<ImportModule>,
    #[serde(default)]
    pub features: Vec<ImportFeature>,
    #[serde(default)]
    pub cycles: Vec<ImportCycle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProject {
    #[serde(default)]
    pub slug: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub features: Vec<ImportFeature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportModule {
    #[serde(default)]
    pub slug: Option<String>,
    pub friendly_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parent_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportFeature {
    #[serde(default)]
    pub slug: Option<String>,
    pub friendly_name: String,
    pub spec_content: String,
    #[serde(default = "default_feature_state")]
    pub state: FeatureState,
    #[serde(default)]
    pub target_branch: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub module_slug: Option<String>,
    #[serde(default)]
    pub project_id: Option<i64>,
    #[serde(default)]
    pub plane_issue_id: Option<String>,
    #[serde(default)]
    pub plane_state_id: Option<String>,
    #[serde(default)]
    pub work_packages: Vec<ImportWorkPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWorkPackage {
    pub title: String,
    #[serde(default)]
    pub acceptance_criteria: Option<String>,
    #[serde(default)]
    pub sequence: Option<i32>,
    #[serde(default)]
    pub file_scope: Vec<String>,
    #[serde(default = "default_wp_state")]
    pub state: WpState,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub pr_url: Option<String>,
    #[serde(default)]
    pub pr_state: Option<PrState>,
    #[serde(default)]
    pub worktree_path: Option<String>,
    #[serde(default)]
    pub plane_sub_issue_id: Option<String>,
    #[serde(default)]
    pub depends_on_sequences: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCycle {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[serde(default = "default_cycle_state")]
    pub state: CycleState,
    #[serde(default)]
    pub module_scope_slug: Option<String>,
    #[serde(default)]
    pub feature_slugs: Vec<String>,
}

fn default_feature_state() -> FeatureState {
    FeatureState::Specified
}

fn default_wp_state() -> WpState {
    WpState::Planned
}

fn default_cycle_state() -> CycleState {
    CycleState::Draft
}

impl ImportModule {
    pub fn slug(&self) -> String {
        self.slug
            .clone()
            .unwrap_or_else(|| Module::slug_from_name(&self.friendly_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn bundle_default_has_empty_vectors() {
        let bundle = ImportBundle::default();
        assert!(bundle.projects.is_empty());
        assert!(bundle.modules.is_empty());
        assert!(bundle.features.is_empty());
        assert!(bundle.cycles.is_empty());
    }

    #[test]
    fn module_slug_prefers_explicit_value() {
        let module = ImportModule {
            slug: Some("custom-slug".into()),
            friendly_name: "Friendly Name".into(),
            description: None,
            parent_slug: None,
        };
        assert_eq!(module.slug(), "custom-slug");
    }

    #[test]
    fn module_slug_derives_from_friendly_name_when_absent() {
        let module = ImportModule {
            slug: None,
            friendly_name: "OAuth Providers".into(),
            description: None,
            parent_slug: None,
        };
        assert_eq!(module.slug(), "oauth-providers");
    }

    #[test]
    fn module_slug_of_empty_name_is_empty() {
        let module = ImportModule {
            slug: None,
            friendly_name: "   ".into(),
            description: None,
            parent_slug: None,
        };
        assert_eq!(module.slug(), "");
    }

    #[test]
    fn deserialize_minimal_feature_applies_defaults() {
        let feature: ImportFeature =
            serde_json::from_str(r#"{"friendly_name":"Auth","spec_content":"spec"}"#).unwrap();

        assert!(feature.slug.is_none());
        assert_eq!(feature.state, FeatureState::Specified);
        assert!(feature.target_branch.is_none());
        assert!(feature.labels.is_empty());
        assert!(feature.module_slug.is_none());
        assert!(feature.work_packages.is_empty());
        assert!(feature.plane_issue_id.is_none());
        assert!(feature.plane_state_id.is_none());
    }

    #[test]
    fn deserialize_feature_requires_friendly_name_and_spec_content() {
        let missing_spec: Result<ImportFeature, _> =
            serde_json::from_str(r#"{"friendly_name":"Auth"}"#);
        assert!(missing_spec.is_err());

        let missing_name: Result<ImportFeature, _> =
            serde_json::from_str(r#"{"spec_content":"spec"}"#);
        assert!(missing_name.is_err());
    }

    #[test]
    fn deserialize_minimal_work_package_applies_defaults() {
        let wp: ImportWorkPackage = serde_json::from_str(r#"{"title":"Do it"}"#).unwrap();

        assert_eq!(wp.state, WpState::Planned);
        assert!(wp.sequence.is_none());
        assert!(wp.acceptance_criteria.is_none());
        assert!(wp.file_scope.is_empty());
        assert!(wp.agent_id.is_none());
        assert!(wp.pr_url.is_none());
        assert!(wp.pr_state.is_none());
        assert!(wp.worktree_path.is_none());
        assert!(wp.plane_sub_issue_id.is_none());
        assert!(wp.depends_on_sequences.is_empty());
    }

    #[test]
    fn deserialize_work_package_requires_title() {
        let result: Result<ImportWorkPackage, _> = serde_json::from_str(r#"{"sequence":1}"#);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_minimal_cycle_applies_defaults() {
        let cycle: ImportCycle = serde_json::from_str(
            r#"{"name":"C1","start_date":"2026-01-01","end_date":"2026-02-01"}"#,
        )
        .unwrap();

        assert_eq!(cycle.state, CycleState::Draft);
        assert!(cycle.description.is_none());
        assert!(cycle.module_scope_slug.is_none());
        assert!(cycle.feature_slugs.is_empty());
        assert_eq!(cycle.start_date, date(2026, 1, 1));
        assert_eq!(cycle.end_date, date(2026, 2, 1));
    }

    #[test]
    fn deserialize_cycle_requires_dates_and_name() {
        let missing_dates: Result<ImportCycle, _> = serde_json::from_str(r#"{"name":"C1"}"#);
        assert!(missing_dates.is_err());

        let missing_name: Result<ImportCycle, _> =
            serde_json::from_str(r#"{"start_date":"2026-01-01","end_date":"2026-02-01"}"#);
        assert!(missing_name.is_err());
    }

    #[test]
    fn deserialize_bundle_from_empty_object_defaults_everything() {
        let bundle: ImportBundle = serde_json::from_str("{}").unwrap();
        assert!(bundle.projects.is_empty());
        assert!(bundle.modules.is_empty());
        assert!(bundle.features.is_empty());
        assert!(bundle.cycles.is_empty());
    }

    #[test]
    fn deserialize_project_requires_only_name() {
        let project: ImportProject = serde_json::from_str(r#"{"name":"Roadmap"}"#).unwrap();
        assert!(project.slug.is_none());
        assert!(project.description.is_none());
        assert!(project.features.is_empty());

        let missing_name: Result<ImportProject, _> = serde_json::from_str(r#"{}"#);
        assert!(missing_name.is_err());
    }

    #[test]
    fn default_state_helpers_return_expected_variants() {
        assert_eq!(default_feature_state(), FeatureState::Specified);
        assert_eq!(default_wp_state(), WpState::Planned);
        assert_eq!(default_cycle_state(), CycleState::Draft);
    }

    #[test]
    fn feature_embedded_work_packages_deserialize() {
        let feature: ImportFeature = serde_json::from_str(
            r#"{
                "friendly_name": "Auth",
                "spec_content": "spec",
                "work_packages": [
                    {"title": "One", "sequence": 1},
                    {"title": "Two", "sequence": 2, "depends_on_sequences": [1]}
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(feature.work_packages.len(), 2);
        assert_eq!(feature.work_packages[1].depends_on_sequences, vec![1]);
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let feature: ImportFeature = serde_json::from_str(
            r#"{"friendly_name":"Auth","spec_content":"spec","future_field":true}"#,
        )
        .unwrap();
        assert_eq!(feature.friendly_name, "Auth");
    }
}
