use super::*;
use agileplus_domain::domain::cycle::{Cycle, CycleState};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::module::Module;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::StoragePort;
use chrono::NaiveDate;

use crate::commands::list_tests::{MemFault, MemStore};

/// Traces to: FR-C01
#[test]
fn create_args_round_trip() {
    let args = CreateArgs {
        name: "Q1-2026".to_string(),
        start: "2026-01-01".to_string(),
        end: "2026-03-31".to_string(),
        description: Some("First quarter".to_string()),
        module: None,
    };
    assert_eq!(args.name, "Q1-2026");
    let start = NaiveDate::parse_from_str(&args.start, "%Y-%m-%d").unwrap();
    let end = NaiveDate::parse_from_str(&args.end, "%Y-%m-%d").unwrap();
    assert!(end > start);
}

/// Traces to: FR-C01
#[test]
fn create_args_invalid_date_format() {
    let bad = "2026/01/01";
    let result = NaiveDate::parse_from_str(bad, "%Y-%m-%d");
    assert!(result.is_err(), "bad date format should not parse");
}

/// Traces to: FR-C02
#[test]
fn transition_args_state_parsing() {
    let valid = ["Draft", "Active", "Review", "Shipped", "Archived"];
    for s in valid {
        assert!(
            s.parse::<CycleState>().is_ok(),
            "state '{}' should parse",
            s
        );
    }
    let bad = "unknown".parse::<CycleState>();
    assert!(bad.is_err());
}

/// Traces to: FR-C02
#[test]
fn prior_state_label_coverage() {
    let cycle = Cycle::new(
        "c",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        None,
    )
    .unwrap();
    let label = prior_state_label(CycleState::Active, &cycle);
    assert!(!label.is_empty());
}

/// Traces to: FR-C03
#[test]
fn add_args_fields() {
    let args = AddArgs {
        cycle: "Q1".to_string(),
        feature: "feat-auth".to_string(),
    };
    assert_eq!(args.cycle, "Q1");
    assert_eq!(args.feature, "feat-auth");
}

/// Traces to: FR-C03
#[test]
fn remove_args_fields() {
    let args = RemoveArgs {
        cycle: "Q1".to_string(),
        feature: "feat-auth".to_string(),
    };
    assert_eq!(args.cycle, "Q1");
    assert_eq!(args.feature, "feat-auth");
}

/// Traces to: FR-C04
#[test]
fn list_args_no_state() {
    let args = ListArgs { state: None };
    assert!(args.state.is_none());
}

/// Traces to: FR-C04
#[test]
fn list_args_with_state() {
    let args = ListArgs {
        state: Some("Active".to_string()),
    };
    let state = args.state.unwrap().parse::<CycleState>().unwrap();
    assert_eq!(state, CycleState::Active);
}

/// Traces to: FR-C05
#[test]
fn show_args_name() {
    let args = ShowArgs {
        name: "Q1-2026".to_string(),
    };
    assert_eq!(args.name, "Q1-2026");
}

// ── Handler behaviour tests ──────────────────────────────────────────────────
//
// These drive the real `cmd_*` handlers against the in-memory `StoragePort`
// double and assert on persisted state plus error messages.

fn seed_cycle(store: &MemStore, name: &str, state: CycleState, scope: Option<i64>) -> i64 {
    let mut guard = store.cycles.lock().unwrap();
    let id = guard.len() as i64 + 1;
    let mut cycle = Cycle::new(
        name,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
        scope,
    )
    .expect("fixture cycle dates are valid");
    cycle.id = id;
    cycle.state = state;
    guard.push(cycle);
    id
}

fn seed_module(store: &MemStore, name: &str) -> i64 {
    let mut guard = store.modules.lock().unwrap();
    let id = guard.len() as i64 + 1;
    let mut m = Module::new(name, None);
    m.id = id;
    guard.push(m);
    id
}

fn seed_feature(store: &MemStore, slug: &str, state: FeatureState) -> i64 {
    let mut guard = store.features.lock().unwrap();
    let id = guard.len() as i64 + 1;
    let mut f = Feature::new(slug, slug, [0u8; 32], None);
    f.id = id;
    f.state = state;
    guard.push(f);
    id
}

fn link_feature(store: &MemStore, cycle_id: i64, feature_id: i64) {
    store
        .cycle_features
        .lock()
        .unwrap()
        .push(agileplus_domain::domain::cycle::CycleFeature::new(
            cycle_id, feature_id,
        ));
}

fn create_args(name: &str, start: &str, end: &str, scope: Option<&str>) -> CreateArgs {
    CreateArgs {
        name: name.to_string(),
        start: start.to_string(),
        end: end.to_string(),
        description: None,
        module: scope.map(|s| s.to_string()),
    }
}

#[tokio::test]
async fn create_persists_parsed_dates_and_description() {
    let store = MemStore::default();
    let mut args = create_args("Q1-2026", "2026-01-01", "2026-03-31", None);
    args.description = Some("First quarter".to_string());

    super::create::cmd_create(args, &store).await.unwrap();

    let cycles = store.cycles.lock().unwrap();
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].name, "Q1-2026");
    assert_eq!(
        cycles[0].start_date,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );
    assert_eq!(
        cycles[0].end_date,
        NaiveDate::from_ymd_opt(2026, 3, 31).unwrap()
    );
    assert_eq!(cycles[0].description.as_deref(), Some("First quarter"));
    assert_eq!(cycles[0].state, CycleState::Draft);
    assert!(cycles[0].module_scope_id.is_none());
}

#[tokio::test]
async fn create_rejects_malformed_start_date() {
    let store = MemStore::default();
    let err = super::create::cmd_create(
        create_args("Q1", "2026/01/01", "2026-03-31", None),
        &store,
    )
    .await
    .expect_err("bad start date must fail");

    let message = format!("{err:#}");
    assert!(message.contains("invalid start date '2026/01/01'"), "{message}");
    assert!(message.contains("YYYY-MM-DD"), "{message}");
    assert!(store.cycles.lock().unwrap().is_empty());
}

#[tokio::test]
async fn create_rejects_malformed_end_date() {
    let store = MemStore::default();
    let err = super::create::cmd_create(
        create_args("Q1", "2026-01-01", "2026-13-40", None),
        &store,
    )
    .await
    .expect_err("bad end date must fail");
    assert!(format!("{err:#}").contains("invalid end date '2026-13-40'"));
}

#[tokio::test]
async fn create_rejects_end_before_start() {
    let store = MemStore::default();
    let err = super::create::cmd_create(
        create_args("Q1", "2026-03-31", "2026-01-01", None),
        &store,
    )
    .await
    .expect_err("inverted range must fail");
    assert!(format!("{err:#}").contains("end_date must be after start_date"));
    assert!(store.cycles.lock().unwrap().is_empty());
}

#[tokio::test]
async fn create_scopes_cycle_to_resolved_module() {
    let store = MemStore::default();
    let module_id = seed_module(&store, "Platform");

    super::create::cmd_create(
        create_args("Q1", "2026-01-01", "2026-03-31", Some("platform")),
        &store,
    )
    .await
    .unwrap();

    assert_eq!(store.cycles.lock().unwrap()[0].module_scope_id, Some(module_id));
}

#[tokio::test]
async fn create_rejects_unknown_module_scope() {
    let store = MemStore::default();
    let err = super::create::cmd_create(
        create_args("Q1", "2026-01-01", "2026-03-31", Some("ghost")),
        &store,
    )
    .await
    .expect_err("unknown scope must fail");
    assert!(format!("{err:#}").contains("Module 'ghost' not found."));
    assert!(store.cycles.lock().unwrap().is_empty());
}

#[tokio::test]
async fn list_is_ok_for_empty_store() {
    let store = MemStore::default();
    super::list::cmd_list(ListArgs { state: None }, &store)
        .await
        .unwrap();
}

#[tokio::test]
async fn list_filters_by_state_and_resolves_scope_labels() {
    let store = MemStore::default();
    let module_id = seed_module(&store, "Platform");
    seed_cycle(&store, "Active-1", CycleState::Active, Some(module_id));
    seed_cycle(&store, "Draft-1", CycleState::Draft, None);
    // Scope points at a module that no longer exists -> `id:N` fallback.
    seed_cycle(&store, "Active-orphan", CycleState::Active, Some(404));

    super::list::cmd_list(
        ListArgs {
            state: Some("Active".to_string()),
        },
        &store,
    )
    .await
    .unwrap();
    super::list::cmd_list(ListArgs { state: None }, &store)
        .await
        .unwrap();
}

#[tokio::test]
async fn list_rejects_unknown_state_filter() {
    let store = MemStore::default();
    let err = super::list::cmd_list(
        ListArgs {
            state: Some("Nope".to_string()),
        },
        &store,
    )
    .await
    .expect_err("unknown state must fail");
    assert!(format!("{err:#}").contains("unknown cycle state: Nope"));
}

#[tokio::test]
async fn show_reports_missing_cycle_with_remediation() {
    let store = MemStore::default();
    let err = super::show::cmd_show(
        ShowArgs {
            name: "ghost".to_string(),
        },
        &store,
    )
    .await
    .expect_err("missing cycle must fail");

    let message = format!("{err:#}");
    assert!(message.contains("Cycle 'ghost' not found"), "{message}");
    assert!(message.contains("agileplus cycle create --name ghost"), "{message}");
}

#[tokio::test]
async fn show_renders_features_and_work_package_progress() {
    let store = MemStore::default();
    let module_id = seed_module(&store, "Platform");
    let cycle_id = seed_cycle(&store, "Q1-2026", CycleState::Active, Some(module_id));
    let blocked = seed_feature(&store, "login", FeatureState::Implementing);
    let done = seed_feature(&store, "signup", FeatureState::Validated);
    link_feature(&store, cycle_id, blocked);
    link_feature(&store, cycle_id, done);
    store.cycles.lock().unwrap()[0].description = Some("scope".to_string());
    *store.cycle_wp_progress.lock().unwrap() = Some(
        agileplus_domain::domain::cycle::WpProgressSummary {
            total: 5,
            planned: 1,
            in_progress: 1,
            done: 2,
            blocked: 1,
        },
    );

    super::show::cmd_show(
        ShowArgs {
            name: "Q1-2026".to_string(),
        },
        &store,
    )
    .await
    .unwrap();

    // The rendered view derives from the same port data the handler reads.
    let cwf = store
        .get_cycle_with_features(cycle_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cwf.features.len(), 2);
    assert_eq!(cwf.wp_progress.total, 5);
    assert!(!cwf.is_shippable(), "implementing feature blocks the cycle");
}

#[tokio::test]
async fn show_handles_cycle_without_features_or_scope() {
    let store = MemStore::default();
    seed_cycle(&store, "Q2", CycleState::Draft, None);
    *store.cycle_wp_progress.lock().unwrap() = Some(
        agileplus_domain::domain::cycle::WpProgressSummary::default(),
    );

    super::show::cmd_show(
        ShowArgs {
            name: "Q2".to_string(),
        },
        &store,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn show_detects_cycle_that_disappears_between_reads() {
    let store = MemStore::default();
    seed_cycle(&store, "Q1", CycleState::Draft, None);
    *store.module_fault.lock().unwrap() = MemFault::CycleDetailMissing;

    let err = super::show::cmd_show(
        ShowArgs {
            name: "Q1".to_string(),
        },
        &store,
    )
    .await
    .expect_err("detail gap must surface");
    assert!(format!("{err:#}").contains("disappeared unexpectedly"));
}

#[tokio::test]
async fn add_links_feature_to_cycle() {
    let store = MemStore::default();
    let cycle_id = seed_cycle(&store, "Q1", CycleState::Draft, None);
    let feature_id = seed_feature(&store, "login", FeatureState::Created);

    super::add::cmd_add(
        AddArgs {
            cycle: "Q1".to_string(),
            feature: "login".to_string(),
        },
        &store,
    )
    .await
    .unwrap();

    let links = store.cycle_features.lock().unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].cycle_id, cycle_id);
    assert_eq!(links[0].feature_id, feature_id);
}

#[tokio::test]
async fn add_reports_unknown_cycle() {
    let store = MemStore::default();
    seed_feature(&store, "login", FeatureState::Created);
    let err = super::add::cmd_add(
        AddArgs {
            cycle: "ghost".to_string(),
            feature: "login".to_string(),
        },
        &store,
    )
    .await
    .expect_err("unknown cycle must fail");
    assert!(format!("{err:#}").contains("Cycle 'ghost' not found"));
}

#[tokio::test]
async fn add_reports_unknown_feature_with_specify_hint() {
    let store = MemStore::default();
    seed_cycle(&store, "Q1", CycleState::Draft, None);
    let err = super::add::cmd_add(
        AddArgs {
            cycle: "Q1".to_string(),
            feature: "ghost".to_string(),
        },
        &store,
    )
    .await
    .expect_err("unknown feature must fail");

    let message = format!("{err:#}");
    assert!(message.contains("Feature 'ghost' not found."), "{message}");
    assert!(message.contains("agileplus specify --feature ghost"), "{message}");
}

#[tokio::test]
async fn add_explains_module_scope_violation() {
    let store = MemStore::default();
    seed_cycle(&store, "Q1", CycleState::Draft, None);
    seed_feature(&store, "login", FeatureState::Created);
    *store.module_fault.lock().unwrap() = MemFault::AddFeatureOutOfScope;

    let err = super::add::cmd_add(
        AddArgs {
            cycle: "Q1".to_string(),
            feature: "login".to_string(),
        },
        &store,
    )
    .await
    .expect_err("out-of-scope feature must be refused");
    let message = format!("{err:#}");

    assert!(
        message.contains("the cycle is scoped to module 'platform'"),
        "{message}"
    );
    assert!(
        message.contains("agileplus module tag --module platform --feature login"),
        "{message}"
    );
    assert!(store.cycle_features.lock().unwrap().is_empty());
}

#[tokio::test]
async fn add_wraps_unexpected_storage_failure() {
    let store = MemStore::default();
    seed_cycle(&store, "Q1", CycleState::Draft, None);
    seed_feature(&store, "login", FeatureState::Created);
    *store.module_fault.lock().unwrap() = MemFault::AddFeatureStorageError;

    let err = super::add::cmd_add(
        AddArgs {
            cycle: "Q1".to_string(),
            feature: "login".to_string(),
        },
        &store,
    )
    .await
    .expect_err("storage failure must propagate");
    let message = format!("{err:#}");
    assert!(message.contains("adding feature to cycle"), "{message}");
    assert!(message.contains("cycle write refused"), "{message}");
}

#[tokio::test]
async fn remove_unlinks_feature_and_leaves_others() {
    let store = MemStore::default();
    let cycle_id = seed_cycle(&store, "Q1", CycleState::Draft, None);
    let kept_cycle = seed_cycle(&store, "Q2", CycleState::Draft, None);
    let login = seed_feature(&store, "login", FeatureState::Created);
    link_feature(&store, cycle_id, login);
    link_feature(&store, kept_cycle, login);

    super::remove::cmd_remove(
        RemoveArgs {
            cycle: "Q1".to_string(),
            feature: "login".to_string(),
        },
        &store,
    )
    .await
    .unwrap();

    let links = store.cycle_features.lock().unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].cycle_id, kept_cycle);
}

#[tokio::test]
async fn remove_reports_unknown_feature() {
    let store = MemStore::default();
    seed_cycle(&store, "Q1", CycleState::Draft, None);
    let err = super::remove::cmd_remove(
        RemoveArgs {
            cycle: "Q1".to_string(),
            feature: "ghost".to_string(),
        },
        &store,
    )
    .await
    .expect_err("unknown feature must fail");
    assert!(format!("{err:#}").contains("Feature 'ghost' not found."));
}

#[tokio::test]
async fn remove_wraps_storage_failure() {
    let store = MemStore::default();
    seed_cycle(&store, "Q1", CycleState::Draft, None);
    seed_feature(&store, "login", FeatureState::Created);
    *store.module_fault.lock().unwrap() = MemFault::RemoveFeatureStorageError;

    let err = super::remove::cmd_remove(
        RemoveArgs {
            cycle: "Q1".to_string(),
            feature: "login".to_string(),
        },
        &store,
    )
    .await
    .expect_err("storage failure must propagate");
    let message = format!("{err:#}");
    assert!(message.contains("removing feature from cycle"), "{message}");
    assert!(message.contains("cycle delete refused"), "{message}");
}

#[tokio::test]
async fn transition_advances_state_and_persists() {
    let store = MemStore::default();
    seed_cycle(&store, "Q1", CycleState::Draft, None);

    super::transition::cmd_transition(
        TransitionArgs {
            cycle: "Q1".to_string(),
            to: "Active".to_string(),
        },
        &store,
    )
    .await
    .unwrap();

    assert_eq!(store.cycles.lock().unwrap()[0].state, CycleState::Active);
}

#[tokio::test]
async fn transition_rejects_unknown_target_state() {
    let store = MemStore::default();
    seed_cycle(&store, "Q1", CycleState::Draft, None);
    let err = super::transition::cmd_transition(
        TransitionArgs {
            cycle: "Q1".to_string(),
            to: "Sideways".to_string(),
        },
        &store,
    )
    .await
    .expect_err("unknown state must fail");
    assert!(format!("{err:#}").contains("unknown cycle state: Sideways"));
    assert_eq!(store.cycles.lock().unwrap()[0].state, CycleState::Draft);
}

#[tokio::test]
async fn transition_rejects_edge_outside_state_graph() {
    let store = MemStore::default();
    seed_cycle(&store, "Q1", CycleState::Draft, None);
    let err = super::transition::cmd_transition(
        TransitionArgs {
            cycle: "Q1".to_string(),
            to: "Review".to_string(),
        },
        &store,
    )
    .await
    .expect_err("Draft -> Review is not a permitted edge");
    assert!(format!("{err:#}").contains("Invalid transition from Draft to Review"));
    assert_eq!(store.cycles.lock().unwrap()[0].state, CycleState::Draft);
}

#[tokio::test]
async fn shipped_gate_blocks_unfinished_features() {
    let store = MemStore::default();
    let cycle_id = seed_cycle(&store, "Q1", CycleState::Review, None);
    let login = seed_feature(&store, "login", FeatureState::Implementing);
    link_feature(&store, cycle_id, login);

    let err = super::transition::cmd_transition(
        TransitionArgs {
            cycle: "Q1".to_string(),
            to: "Shipped".to_string(),
        },
        &store,
    )
    .await
    .expect_err("unfinished features must block Shipped");
    let message = format!("{err:#}");

    assert!(
        message.contains("Cannot transition cycle 'Q1' to Shipped"),
        "{message}"
    );
    assert!(
        message.contains("1 feature(s) are not Validated or Shipped"),
        "{message}"
    );
    assert!(message.contains("login (state: "), "{message}");
    assert!(
        message.contains("agileplus validate --feature <slug>"),
        "{message}"
    );
    assert_eq!(
        store.cycles.lock().unwrap()[0].state,
        CycleState::Review,
        "state must not advance when the gate blocks"
    );
}

#[tokio::test]
async fn shipped_gate_allows_validated_features() {
    let store = MemStore::default();
    let cycle_id = seed_cycle(&store, "Q1", CycleState::Review, None);
    let login = seed_feature(&store, "login", FeatureState::Validated);
    let signup = seed_feature(&store, "signup", FeatureState::Shipped);
    link_feature(&store, cycle_id, login);
    link_feature(&store, cycle_id, signup);

    super::transition::cmd_transition(
        TransitionArgs {
            cycle: "Q1".to_string(),
            to: "Shipped".to_string(),
        },
        &store,
    )
    .await
    .unwrap();

    assert_eq!(store.cycles.lock().unwrap()[0].state, CycleState::Shipped);
}

#[tokio::test]
async fn shipped_gate_fails_when_cycle_detail_is_unavailable() {
    let store = MemStore::default();
    seed_cycle(&store, "Q1", CycleState::Review, None);
    *store.module_fault.lock().unwrap() = MemFault::CycleDetailMissing;

    let err = super::transition::cmd_transition(
        TransitionArgs {
            cycle: "Q1".to_string(),
            to: "Shipped".to_string(),
        },
        &store,
    )
    .await
    .expect_err("missing detail must fail the gate");
    assert!(format!("{err:#}").contains("disappeared unexpectedly"));
}

#[tokio::test]
async fn dispatcher_routes_every_cycle_subcommand() {
    let store = MemStore::default();
    seed_feature(&store, "login", FeatureState::Created);

    for command in [
        CycleCommand::Create(create_args("Q1", "2026-01-01", "2026-03-31", None)),
        CycleCommand::List(ListArgs { state: None }),
        CycleCommand::Show(ShowArgs {
            name: "Q1".to_string(),
        }),
        CycleCommand::Add(AddArgs {
            cycle: "Q1".to_string(),
            feature: "login".to_string(),
        }),
        CycleCommand::Remove(RemoveArgs {
            cycle: "Q1".to_string(),
            feature: "login".to_string(),
        }),
        CycleCommand::Transition(TransitionArgs {
            cycle: "Q1".to_string(),
            to: "Active".to_string(),
        }),
    ] {
        run(CycleArgs { command }, &store)
            .await
            .expect("dispatcher should route to a working handler");
    }

    assert_eq!(store.cycles.lock().unwrap()[0].state, CycleState::Active);
    assert!(store.cycle_features.lock().unwrap().is_empty());
}
