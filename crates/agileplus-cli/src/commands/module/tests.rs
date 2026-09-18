use super::*;
use clap::Parser;

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::module::Module;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::StoragePort;

use crate::commands::list_tests::{MemStore, MemFault};

/// Wrap ModuleArgs so we can parse it from a top-level binary name.
#[derive(Debug, clap::Parser)]
struct TestCli {
    #[command(subcommand)]
    command: ModuleCommand,
}

fn parse(args: &[&str]) -> ModuleCommand {
    TestCli::parse_from(args).command
}

#[test]
fn parse_create_minimal() {
    let cmd = parse(&["cli", "create", "--name", "Auth"]);
    match cmd {
        ModuleCommand::Create(a) => {
            assert_eq!(a.name, "Auth");
            assert!(a.description.is_none());
            assert!(a.parent.is_none());
        }
        _ => panic!("expected Create"),
    }
}

#[test]
fn parse_create_full() {
    let cmd = parse(&[
        "cli",
        "create",
        "--name",
        "Auth",
        "--description",
        "Authentication module",
        "--parent",
        "platform",
    ]);
    match cmd {
        ModuleCommand::Create(a) => {
            assert_eq!(a.name, "Auth");
            assert_eq!(a.description.as_deref(), Some("Authentication module"));
            assert_eq!(a.parent.as_deref(), Some("platform"));
        }
        _ => panic!("expected Create"),
    }
}

#[test]
fn parse_list_flat() {
    let cmd = parse(&["cli", "list"]);
    match cmd {
        ModuleCommand::List(a) => assert!(!a.tree),
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_list_tree() {
    let cmd = parse(&["cli", "list", "--tree"]);
    match cmd {
        ModuleCommand::List(a) => assert!(a.tree),
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_show() {
    let cmd = parse(&["cli", "show", "my-module"]);
    match cmd {
        ModuleCommand::Show(a) => assert_eq!(a.slug, "my-module"),
        _ => panic!("expected Show"),
    }
}

#[test]
fn parse_assign() {
    let cmd = parse(&["cli", "assign", "--module", "platform", "--feature", "auth"]);
    match cmd {
        ModuleCommand::Assign(a) => {
            assert_eq!(a.module, "platform");
            assert_eq!(a.feature, "auth");
        }
        _ => panic!("expected Assign"),
    }
}

#[test]
fn parse_tag() {
    let cmd = parse(&["cli", "tag", "--module", "platform", "--feature", "auth"]);
    match cmd {
        ModuleCommand::Tag(a) => {
            assert_eq!(a.module, "platform");
            assert_eq!(a.feature, "auth");
        }
        _ => panic!("expected Tag"),
    }
}

#[test]
fn parse_untag() {
    let cmd = parse(&["cli", "untag", "--module", "platform", "--feature", "auth"]);
    match cmd {
        ModuleCommand::Untag(a) => {
            assert_eq!(a.module, "platform");
            assert_eq!(a.feature, "auth");
        }
        _ => panic!("expected Untag"),
    }
}

#[test]
fn parse_delete() {
    let cmd = parse(&["cli", "delete", "old-module"]);
    match cmd {
        ModuleCommand::Delete(a) => assert_eq!(a.slug, "old-module"),
        _ => panic!("expected Delete"),
    }
}

// ── Handler behaviour tests ──────────────────────────────────────────────────
//
// These drive the real `run_*` handlers against the in-memory `StoragePort`
// double and assert on observable storage state and error messages. They never
// capture process stdout, so they stay parallel-safe.

/// Seed a root/child module directly into the store and return its id.
fn seed_module(store: &MemStore, name: &str, parent: Option<i64>) -> i64 {
    let mut guard = store.modules.lock().unwrap();
    let id = guard.len() as i64 + 1;
    let mut m = Module::new(name, parent);
    m.id = id;
    guard.push(m);
    id
}

/// Seed a feature directly into the store and return its id.
fn seed_feature(store: &MemStore, slug: &str, title: &str) -> i64 {
    let mut guard = store.features.lock().unwrap();
    let id = guard.len() as i64 + 1;
    let mut f = Feature::new(slug, title, [0u8; 32], None);
    f.id = id;
    f.state = FeatureState::Created;
    guard.push(f);
    id
}

fn module_slugs(store: &MemStore) -> Vec<String> {
    let mut slugs: Vec<String> = store
        .modules
        .lock()
        .unwrap()
        .iter()
        .map(|m| m.slug.clone())
        .collect();
    slugs.sort();
    slugs
}

#[tokio::test]
async fn create_persists_slug_and_description() {
    let store = MemStore::default();
    let args = CreateArgs {
        name: "Payments Platform".to_string(),
        description: Some("billing surfaces".to_string()),
        parent: None,
    };

    super::create::run_create(args, &store).await.unwrap();

    let persisted = store
        .get_module_by_slug("payments-platform")
        .await
        .unwrap()
        .expect("module should be persisted under its derived slug");
    assert_eq!(persisted.friendly_name, "Payments Platform");
    assert_eq!(persisted.description.as_deref(), Some("billing surfaces"));
    assert!(persisted.parent_module_id.is_none());
}

#[tokio::test]
async fn create_with_parent_records_parent_id() {
    let store = MemStore::default();
    let parent_id = seed_module(&store, "Platform", None);

    let args = CreateArgs {
        name: "Auth".to_string(),
        description: None,
        parent: Some("platform".to_string()),
    };
    super::create::run_create(args, &store).await.unwrap();

    let child = store
        .get_module_by_slug("auth")
        .await
        .unwrap()
        .expect("child module persisted");
    assert_eq!(child.parent_module_id, Some(parent_id));
    assert!(child.description.is_none());
}

#[tokio::test]
async fn create_with_unknown_parent_fails_with_actionable_message() {
    let store = MemStore::default();
    let args = CreateArgs {
        name: "Auth".to_string(),
        description: None,
        parent: Some("ghost".to_string()),
    };

    let err = super::create::run_create(args, &store)
        .await
        .expect_err("unknown parent must fail");
    let message = format!("{err:#}");

    assert!(
        message.contains("parent module 'ghost' not found"),
        "unexpected error: {message}"
    );
    assert!(store.modules.lock().unwrap().is_empty(), "nothing persisted");
}

#[tokio::test]
async fn list_without_modules_is_ok() {
    let store = MemStore::default();
    super::list::run_list(ListArgs { tree: false }, &store)
        .await
        .unwrap();
}

#[tokio::test]
async fn list_flat_enumerates_nested_modules() {
    let store = MemStore::default();
    let root = seed_module(&store, "Platform", None);
    let child = seed_module(&store, "Auth", Some(root));
    seed_module(&store, "Tokens", Some(child));

    super::list::run_list(ListArgs { tree: false }, &store)
        .await
        .unwrap();
    super::list::run_list(ListArgs { tree: true }, &store)
        .await
        .unwrap();

    assert_eq!(module_slugs(&store), vec!["auth", "platform", "tokens"]);
}

#[tokio::test]
async fn show_missing_module_fails() {
    let store = MemStore::default();
    let args = ShowArgs {
        slug: "nope".to_string(),
    };
    let err = super::show::run_show(args, &store)
        .await
        .expect_err("missing module must fail");
    assert!(format!("{err:#}").contains("module 'nope' not found"));
}

#[tokio::test]
async fn show_renders_all_sections_for_populated_module() {
    let store = MemStore::default();
    let root = seed_module(&store, "Platform", None);
    store.modules.lock().unwrap()[0].description = Some("core".to_string());
    let child = seed_module(&store, "Auth", Some(root));
    let feature_id = seed_feature(&store, "login", "Login");
    store
        .module_feature_tags
        .lock()
        .unwrap()
        .push(agileplus_domain::domain::module::ModuleFeatureTag::new(
            root, feature_id,
        ));

    super::show::run_show(
        ShowArgs {
            slug: "platform".to_string(),
        },
        &store,
    )
    .await
    .unwrap();

    let details = store.get_module_with_features(root).await.unwrap().unwrap();
    assert_eq!(details.module.description.as_deref(), Some("core"));
    assert_eq!(details.tagged_features.len(), 1);
    assert_eq!(details.tagged_features[0].slug, "login");
    assert_eq!(details.child_modules.len(), 1);
    assert_eq!(details.child_modules[0].id, child);
}

#[tokio::test]
async fn show_module_that_disappears_between_reads_fails() {
    let store = MemStore::default();
    let id = seed_module(&store, "Platform", None);
    *store.module_fault.lock().unwrap() = MemFault::DetailMissing;

    let err = super::show::run_show(
        ShowArgs {
            slug: "platform".to_string(),
        },
        &store,
    )
    .await
    .expect_err("detail lookup gap must surface");
    assert!(format!("{err:#}").contains("disappeared during load"));
    assert_eq!(store.modules.lock().unwrap()[0].id, id);
}

#[tokio::test]
async fn tag_links_feature_to_module() {
    let store = MemStore::default();
    let module_id = seed_module(&store, "Platform", None);
    let feature_id = seed_feature(&store, "login", "Login");

    super::tag::run_tag(
        TagArgs {
            module: "platform".to_string(),
            feature: "login".to_string(),
        },
        &store,
    )
    .await
    .unwrap();

    let tags = store.module_feature_tags.lock().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].module_id, module_id);
    assert_eq!(tags[0].feature_id, feature_id);
}

#[tokio::test]
async fn tag_unknown_feature_reports_missing_slug() {
    let store = MemStore::default();
    seed_module(&store, "Platform", None);

    let err = super::tag::run_tag(
        TagArgs {
            module: "platform".to_string(),
            feature: "ghost".to_string(),
        },
        &store,
    )
    .await
    .expect_err("unknown feature must fail");
    assert!(format!("{err:#}").contains("feature 'ghost' not found"));
    assert!(store.module_feature_tags.lock().unwrap().is_empty());
}

#[tokio::test]
async fn tag_unknown_module_reports_missing_slug() {
    let store = MemStore::default();
    seed_feature(&store, "login", "Login");

    let err = super::tag::run_tag(
        TagArgs {
            module: "ghost".to_string(),
            feature: "login".to_string(),
        },
        &store,
    )
    .await
    .expect_err("unknown module must fail");
    assert!(format!("{err:#}").contains("module 'ghost' not found"));
}

#[tokio::test]
async fn untag_removes_existing_link_only() {
    let store = MemStore::default();
    let module_id = seed_module(&store, "Platform", None);
    let other_id = seed_module(&store, "Billing", None);
    let feature_id = seed_feature(&store, "login", "Login");

    {
        let mut tags = store.module_feature_tags.lock().unwrap();
        tags.push(agileplus_domain::domain::module::ModuleFeatureTag::new(
            module_id,
            feature_id,
        ));
        tags.push(agileplus_domain::domain::module::ModuleFeatureTag::new(
            other_id,
            feature_id,
        ));
    }

    super::untag::run_untag(
        UntagArgs {
            module: "platform".to_string(),
            feature: "login".to_string(),
        },
        &store,
    )
    .await
    .unwrap();

    let tags = store.module_feature_tags.lock().unwrap();
    assert_eq!(tags.len(), 1, "only the targeted link is removed");
    assert_eq!(tags[0].module_id, other_id);
}

#[tokio::test]
async fn assign_records_ownership_tag() {
    let store = MemStore::default();
    let module_id = seed_module(&store, "Platform", None);
    let feature_id = seed_feature(&store, "login", "Login");

    super::assign::run_assign(
        AssignArgs {
            module: "platform".to_string(),
            feature: "login".to_string(),
        },
        &store,
    )
    .await
    .unwrap();

    let tags = store.module_feature_tags.lock().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].module_id, module_id);
    assert_eq!(tags[0].feature_id, feature_id);
}

#[tokio::test]
async fn assign_unknown_feature_fails() {
    let store = MemStore::default();
    seed_module(&store, "Platform", None);
    let err = super::assign::run_assign(
        AssignArgs {
            module: "platform".to_string(),
            feature: "ghost".to_string(),
        },
        &store,
    )
    .await
    .expect_err("unknown feature must fail");
    assert!(format!("{err:#}").contains("feature 'ghost' not found"));
}

#[tokio::test]
async fn delete_removes_module() {
    let store = MemStore::default();
    seed_module(&store, "Platform", None);

    super::delete::run_delete(
        DeleteArgs {
            slug: "platform".to_string(),
        },
        &store,
    )
    .await
    .unwrap();

    assert!(store.modules.lock().unwrap().is_empty());
}

#[tokio::test]
async fn delete_missing_module_fails() {
    let store = MemStore::default();
    let err = super::delete::run_delete(
        DeleteArgs {
            slug: "ghost".to_string(),
        },
        &store,
    )
    .await
    .expect_err("missing module must fail");
    assert!(format!("{err:#}").contains("module 'ghost' not found"));
}

#[tokio::test]
async fn delete_with_dependents_explains_remediation() {
    let store = MemStore::default();
    seed_module(&store, "Platform", None);
    *store.module_fault.lock().unwrap() = MemFault::DeleteHasDependents;

    let err = super::delete::run_delete(
        DeleteArgs {
            slug: "platform".to_string(),
        },
        &store,
    )
    .await
    .expect_err("dependent module must not delete");
    let message = format!("{err:#}");

    assert!(message.contains("cannot delete module 'platform'"), "{message}");
    assert!(message.contains("it still has children or owned features"), "{message}");
    assert!(message.contains("module still owns 2 features"), "{message}");
    assert_eq!(store.modules.lock().unwrap().len(), 1, "module retained");
}

#[tokio::test]
async fn delete_storage_failure_is_wrapped_with_slug() {
    let store = MemStore::default();
    seed_module(&store, "Platform", None);
    *store.module_fault.lock().unwrap() = MemFault::DeleteStorageError;

    let err = super::delete::run_delete(
        DeleteArgs {
            slug: "platform".to_string(),
        },
        &store,
    )
    .await
    .expect_err("storage failure must propagate");
    let message = format!("{err:#}");

    assert!(message.contains("deleting module 'platform'"), "{message}");
    assert!(message.contains("disk on fire"), "{message}");
}

#[tokio::test]
async fn dispatcher_routes_every_module_subcommand() {
    let store = MemStore::default();
    seed_feature(&store, "login", "Login");

    for command in [
        ModuleCommand::Create(CreateArgs {
            name: "Platform".to_string(),
            description: None,
            parent: None,
        }),
        ModuleCommand::List(ListArgs { tree: true }),
        ModuleCommand::Show(ShowArgs {
            slug: "platform".to_string(),
        }),
        ModuleCommand::Tag(TagArgs {
            module: "platform".to_string(),
            feature: "login".to_string(),
        }),
        ModuleCommand::Untag(UntagArgs {
            module: "platform".to_string(),
            feature: "login".to_string(),
        }),
        ModuleCommand::Assign(AssignArgs {
            module: "platform".to_string(),
            feature: "login".to_string(),
        }),
        ModuleCommand::Delete(DeleteArgs {
            slug: "platform".to_string(),
        }),
    ] {
        run(ModuleArgs { command }, &store)
            .await
            .expect("dispatcher should route to a working handler");
    }

    assert!(store.modules.lock().unwrap().is_empty());
    // `untag` removed the tag and the later `assign` re-created it, so exactly
    // one ownership tag survives the delete (the mem-store keeps orphan tags).
    let tags = store.module_feature_tags.lock().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].module_id, 1);
    assert_eq!(tags[0].feature_id, 1);
}
