//! Integration tests for agileplus-import manifest types.
//!
//! Verifies serde roundtrips, default field values, and the ImportModule::slug() helper.

use agileplus_domain::domain::{
    cycle::CycleState,
    state_machine::FeatureState,
    work_package::{PrState, WpState},
};
use agileplus_import::{ImportBundle, ImportCycle, ImportFeature, ImportModule, ImportProject, ImportWorkPackage};

// ---------------------------------------------------------------------------
// ImportBundle
// ---------------------------------------------------------------------------

#[test]
fn import_bundle_default_is_empty() {
    let bundle = ImportBundle::default();
    assert!(bundle.projects.is_empty());
    assert!(bundle.modules.is_empty());
    assert!(bundle.features.is_empty());
    assert!(bundle.cycles.is_empty());
}

#[test]
fn import_bundle_json_roundtrip() {
    let bundle = ImportBundle {
        projects: vec![ImportProject {
            slug: Some("my-proj".into()),
            name: "My Project".into(),
            description: Some("A project".into()),
            features: vec![],
        }],
        modules: vec![ImportModule {
            slug: Some("auth".into()),
            friendly_name: "Auth".into(),
            description: Some("Authentication".into()),
            parent_slug: None,
        }],
        features: vec![ImportFeature {
            slug: Some("login".into()),
            friendly_name: "Login".into(),
            spec_content: "# Login\n\nUser logs in.".into(),
            state: FeatureState::Specified,
            target_branch: Some("feature/login".into()),
            labels: vec!["auth".into(), "mvp".into()],
            module_slug: Some("auth".into()),
            project_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            work_packages: vec![],
        }],
        cycles: vec![ImportCycle {
            name: "Sprint 1".into(),
            description: Some("First sprint".into()),
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
            state: CycleState::Draft,
            module_scope_slug: Some("auth".into()),
            feature_slugs: vec!["login".into()],
        }],
    };

    let json = serde_json::to_string(&bundle).unwrap();
    let restored: ImportBundle = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.projects.len(), 1);
    assert_eq!(restored.projects[0].name, "My Project");
    assert_eq!(restored.modules.len(), 1);
    assert_eq!(restored.modules[0].friendly_name, "Auth");
    assert_eq!(restored.features.len(), 1);
    assert_eq!(restored.features[0].friendly_name, "Login");
    assert_eq!(restored.features[0].labels, vec!["auth", "mvp"]);
    assert_eq!(restored.cycles.len(), 1);
    assert_eq!(restored.cycles[0].name, "Sprint 1");
}

#[test]
fn import_bundle_yaml_roundtrip() {
    let bundle = ImportBundle {
        projects: vec![ImportProject {
            slug: None,
            name: "Proj".into(),
            description: None,
            features: vec![],
        }],
        modules: vec![],
        features: vec![],
        cycles: vec![],
    };

    let yaml = serde_yaml::to_string(&bundle).unwrap();
    let restored: ImportBundle = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(restored.projects.len(), 1);
    assert_eq!(restored.projects[0].name, "Proj");
}

#[test]
fn import_bundle_json_empty_object() {
    let bundle: ImportBundle = serde_json::from_str("{}").unwrap();
    assert!(bundle.projects.is_empty());
    assert!(bundle.modules.is_empty());
    assert!(bundle.features.is_empty());
    assert!(bundle.cycles.is_empty());
}

// ---------------------------------------------------------------------------
// ImportProject
// ---------------------------------------------------------------------------

#[test]
fn import_project_serde_defaults() {
    let json = r#"{"name": "Test Project"}"#;
    let project: ImportProject = serde_json::from_str(json).unwrap();
    assert_eq!(project.name, "Test Project");
    assert!(project.slug.is_none());
    assert!(project.description.is_none());
    assert!(project.features.is_empty());
}

#[test]
fn import_project_nested_features() {
    let json = r#"{
        "name": "Proj",
        "slug": "proj",
        "features": [
            {"friendly_name": "Feat A", "spec_content": "content a"},
            {"friendly_name": "Feat B", "spec_content": "content b"}
        ]
    }"#;
    let project: ImportProject = serde_json::from_str(json).unwrap();
    assert_eq!(project.features.len(), 2);
    assert_eq!(project.features[0].friendly_name, "Feat A");
    assert_eq!(project.features[1].spec_content, "content b");
}

// ---------------------------------------------------------------------------
// ImportModule
// ---------------------------------------------------------------------------

#[test]
fn import_module_serde_defaults() {
    let json = r#"{"friendly_name": "My Module"}"#;
    let module: ImportModule = serde_json::from_str(json).unwrap();
    assert_eq!(module.friendly_name, "My Module");
    assert!(module.slug.is_none());
    assert!(module.description.is_none());
    assert!(module.parent_slug.is_none());
}

#[test]
fn import_module_slug_explicit() {
    let module = ImportModule {
        slug: Some("explicit-slug".into()),
        friendly_name: "Friendly Name".into(),
        description: None,
        parent_slug: None,
    };
    assert_eq!(module.slug(), "explicit-slug");
}

#[test]
fn import_module_slug_derived_from_name() {
    let module = ImportModule {
        slug: None,
        friendly_name: "OAuth Providers".into(),
        description: None,
        parent_slug: None,
    };
    // Module::slug_from_name converts non-alphanumeric to '-' and collapses
    assert_eq!(module.slug(), "oauth-providers");
}

#[test]
fn import_module_slug_special_characters() {
    let module = ImportModule {
        slug: None,
        friendly_name: "  Hello   World!  ".into(),
        description: None,
        parent_slug: None,
    };
    assert_eq!(module.slug(), "hello-world");
}

// ---------------------------------------------------------------------------
// ImportFeature
// ---------------------------------------------------------------------------

#[test]
fn import_feature_default_state_is_specified() {
    let json = r#"{"friendly_name": "Feat", "spec_content": "text"}"#;
    let feature: ImportFeature = serde_json::from_str(json).unwrap();
    assert_eq!(feature.state, FeatureState::Specified);
}

#[test]
fn import_feature_serde_defaults() {
    let json = r#"{"friendly_name": "Feat", "spec_content": "text"}"#;
    let feature: ImportFeature = serde_json::from_str(json).unwrap();
    assert!(feature.slug.is_none());
    assert_eq!(feature.state, FeatureState::Specified);
    assert!(feature.target_branch.is_none());
    assert!(feature.labels.is_empty());
    assert!(feature.module_slug.is_none());
    assert!(feature.project_id.is_none());
    assert!(feature.plane_issue_id.is_none());
    assert!(feature.plane_state_id.is_none());
    assert!(feature.work_packages.is_empty());
}

#[test]
fn import_feature_explicit_state() {
    let json = r#"{
        "friendly_name": "Feat",
        "spec_content": "text",
        "state": "implementing"
    }"#;
    let feature: ImportFeature = serde_json::from_str(json).unwrap();
    assert_eq!(feature.state, FeatureState::Implementing);
}

#[test]
fn import_feature_with_work_packages() {
    let json = r#"{
        "friendly_name": "Feat",
        "spec_content": "spec",
        "work_packages": [
            {"title": "WP 1", "sequence": 1},
            {"title": "WP 2", "sequence": 2}
        ]
    }"#;
    let feature: ImportFeature = serde_json::from_str(json).unwrap();
    assert_eq!(feature.work_packages.len(), 2);
    assert_eq!(feature.work_packages[0].title, "WP 1");
    assert_eq!(feature.work_packages[1].title, "WP 2");
}

#[test]
fn import_feature_labels_roundtrip() {
    let feature = ImportFeature {
        slug: None,
        friendly_name: "Feat".into(),
        spec_content: "text".into(),
        state: FeatureState::Created,
        target_branch: None,
        labels: vec!["bug".into(), "p0".into()],
        module_slug: None,
        project_id: None,
        plane_issue_id: None,
        plane_state_id: None,
        work_packages: vec![],
    };
    let json = serde_json::to_string(&feature).unwrap();
    let restored: ImportFeature = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.labels, vec!["bug", "p0"]);
    assert_eq!(restored.state, FeatureState::Created);
}

// ---------------------------------------------------------------------------
// ImportWorkPackage
// ---------------------------------------------------------------------------

#[test]
fn import_work_package_default_state_is_planned() {
    let json = r#"{"title": "Do thing"}"#;
    let wp: ImportWorkPackage = serde_json::from_str(json).unwrap();
    assert_eq!(wp.title, "Do thing");
    assert_eq!(wp.state, WpState::Planned);
}

#[test]
fn import_work_package_serde_defaults() {
    let json = r#"{"title": "Task"}"#;
    let wp: ImportWorkPackage = serde_json::from_str(json).unwrap();
    assert!(wp.acceptance_criteria.is_none());
    assert!(wp.sequence.is_none());
    assert!(wp.file_scope.is_empty());
    assert!(wp.agent_id.is_none());
    assert!(wp.pr_url.is_none());
    assert!(wp.pr_state.is_none());
    assert!(wp.worktree_path.is_none());
    assert!(wp.plane_sub_issue_id.is_none());
    assert!(wp.depends_on_sequences.is_empty());
}

#[test]
fn import_work_package_explicit_state() {
    let json = r#"{"title": "Task", "state": "doing"}"#;
    let wp: ImportWorkPackage = serde_json::from_str(json).unwrap();
    assert_eq!(wp.state, WpState::Doing);
}

#[test]
fn import_work_package_with_pr_state() {
    let json = r#"{
        "title": "Task",
        "state": "review",
        "pr_url": "https://github.com/org/repo/pull/1",
        "pr_state": "approved"
    }"#;
    let wp: ImportWorkPackage = serde_json::from_str(json).unwrap();
    assert_eq!(wp.state, WpState::Review);
    assert_eq!(wp.pr_url.as_deref(), Some("https://github.com/org/repo/pull/1"));
    assert_eq!(wp.pr_state, Some(PrState::Approved));
}

#[test]
fn import_work_package_file_scope_and_dependencies() {
    let json = r#"{
        "title": "Task",
        "file_scope": ["src/main.rs", "src/lib.rs"],
        "depends_on_sequences": [1, 3]
    }"#;
    let wp: ImportWorkPackage = serde_json::from_str(json).unwrap();
    assert_eq!(wp.file_scope, vec!["src/main.rs", "src/lib.rs"]);
    assert_eq!(wp.depends_on_sequences, vec![1, 3]);
}

#[test]
fn import_work_package_all_pr_states() {
    let states = [
        (r#""open""#, PrState::Open),
        (r#""review""#, PrState::Review),
        (r#""changes_requested""#, PrState::ChangesRequested),
        (r#""approved""#, PrState::Approved),
        (r#""merged""#, PrState::Merged),
    ];
    for (state_str, expected) in &states {
        let json = format!(r#"{{"title": "T", "pr_state": {}}}"#, state_str);
        let wp: ImportWorkPackage = serde_json::from_str(&json).unwrap();
        assert_eq!(wp.pr_state, Some(*expected));
    }
}

#[test]
fn import_work_package_all_wp_states() {
    let states = [
        (r#""planned""#, WpState::Planned),
        (r#""doing""#, WpState::Doing),
        (r#""review""#, WpState::Review),
        (r#""done""#, WpState::Done),
        (r#""blocked""#, WpState::Blocked),
    ];
    for (state_str, expected) in &states {
        let json = format!(r#"{{"title": "T", "state": {}}}"#, state_str);
        let wp: ImportWorkPackage = serde_json::from_str(&json).unwrap();
        assert_eq!(wp.state, *expected);
    }
}

// ---------------------------------------------------------------------------
// ImportCycle
// ---------------------------------------------------------------------------

#[test]
fn import_cycle_default_state_is_draft() {
    let json = r#"{
        "name": "Sprint 1",
        "start_date": "2026-01-01",
        "end_date": "2026-01-14"
    }"#;
    let cycle: ImportCycle = serde_json::from_str(json).unwrap();
    assert_eq!(cycle.name, "Sprint 1");
    assert_eq!(cycle.state, CycleState::Draft);
}

#[test]
fn import_cycle_serde_defaults() {
    let json = r#"{
        "name": "Q1",
        "start_date": "2026-01-01",
        "end_date": "2026-03-31"
    }"#;
    let cycle: ImportCycle = serde_json::from_str(json).unwrap();
    assert!(cycle.description.is_none());
    assert_eq!(cycle.state, CycleState::Draft);
    assert!(cycle.module_scope_slug.is_none());
    assert!(cycle.feature_slugs.is_empty());
}

#[test]
fn import_cycle_explicit_state() {
    let json = r#"{
        "name": "Sprint 1",
        "start_date": "2026-01-01",
        "end_date": "2026-01-14",
        "state": "Active"
    }"#;
    let cycle: ImportCycle = serde_json::from_str(json).unwrap();
    // CycleState serde depends on traceability_core; verify it is not Draft
    assert_ne!(cycle.state, CycleState::Draft);
}

#[test]
fn import_cycle_with_feature_slugs() {
    let json = r#"{
        "name": "Sprint 1",
        "start_date": "2026-01-01",
        "end_date": "2026-01-14",
        "feature_slugs": ["login", "signup", "forgot-password"]
    }"#;
    let cycle: ImportCycle = serde_json::from_str(json).unwrap();
    assert_eq!(cycle.feature_slugs, vec!["login", "signup", "forgot-password"]);
}

#[test]
fn import_cycle_roundtrip_preserves_dates() {
    let cycle = ImportCycle {
        name: "Q2".into(),
        description: Some("Second quarter".into()),
        start_date: chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        state: CycleState::Draft,
        module_scope_slug: None,
        feature_slugs: vec!["feat-a".into()],
    };
    let json = serde_json::to_string(&cycle).unwrap();
    let restored: ImportCycle = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.start_date, chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap());
    assert_eq!(restored.end_date, chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
}

// ---------------------------------------------------------------------------
// Complex nested bundle roundtrip
// ---------------------------------------------------------------------------

#[test]
fn full_bundle_yaml_roundtrip() {
    let bundle = ImportBundle {
        projects: vec![
            ImportProject {
                slug: Some("proj-a".into()),
                name: "Project A".into(),
                description: Some("desc".into()),
                features: vec![ImportFeature {
                    slug: Some("feat-1".into()),
                    friendly_name: "Feature One".into(),
                    spec_content: "spec content".into(),
                    state: FeatureState::Planned,
                    target_branch: Some("main".into()),
                    labels: vec!["core".into()],
                    module_slug: Some("mod-a".into()),
                    project_id: Some(1),
                    plane_issue_id: Some("PI-1".into()),
                    plane_state_id: Some("PS-1".into()),
                    work_packages: vec![
                        ImportWorkPackage {
                            title: "WP 1".into(),
                            acceptance_criteria: Some("AC 1".into()),
                            sequence: Some(1),
                            file_scope: vec!["a.rs".into()],
                            state: WpState::Doing,
                            agent_id: Some("agent-1".into()),
                            pr_url: None,
                            pr_state: None,
                            worktree_path: None,
                            plane_sub_issue_id: None,
                            depends_on_sequences: vec![],
                        },
                        ImportWorkPackage {
                            title: "WP 2".into(),
                            acceptance_criteria: None,
                            sequence: Some(2),
                            file_scope: vec![],
                            state: WpState::Planned,
                            agent_id: None,
                            pr_url: None,
                            pr_state: None,
                            worktree_path: None,
                            plane_sub_issue_id: None,
                            depends_on_sequences: vec![1],
                        },
                    ],
                }],
            },
        ],
        modules: vec![ImportModule {
            slug: Some("mod-a".into()),
            friendly_name: "Module A".into(),
            description: None,
            parent_slug: None,
        }],
        features: vec![],
        cycles: vec![ImportCycle {
            name: "Sprint 1".into(),
            description: None,
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
            state: CycleState::Draft,
            module_scope_slug: Some("mod-a".into()),
            feature_slugs: vec!["feat-1".into()],
        }],
    };

    let yaml = serde_yaml::to_string(&bundle).unwrap();
    let restored: ImportBundle = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(restored.projects[0].features[0].work_packages.len(), 2);
    assert_eq!(restored.projects[0].features[0].work_packages[1].depends_on_sequences, vec![1]);
    assert_eq!(restored.cycles[0].feature_slugs, vec!["feat-1"]);
}
