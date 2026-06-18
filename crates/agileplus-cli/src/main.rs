// SPDX-License-Identifier: MIT OR Apache-2.0
//! agileplus-cli — minimal smoke-test CLI backed by in-memory mock data.

mod sync_cmd;

pub mod commands;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use agileplus_domain::domain::{
    cycle::{Cycle, CycleState},
    feature::Feature,
    module::Module,
    state_machine::FeatureState,
};
use chrono::NaiveDate;

use sync_cmd::SyncArgs;

// ── top-level CLI ────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "agileplus",
    about = "AgilePlus project management CLI",
    version,
    arg_required_else_help = true
)]
struct Cli {
    /// SQLite database path. Defaults to AGILEPLUS_DB, then ./agileplus.db.
    #[arg(long, env = "AGILEPLUS_DB", value_name = "PATH", global = true)]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Feature management
    Feature {
        #[command(subcommand)]
        sub: FeatureCmd,
    },
    /// Module management
    Module {
        #[command(subcommand)]
        sub: ModuleCmd,
    },
    /// Cycle management
    Cycle {
        #[command(subcommand)]
        sub: CycleCmd,
    },
    /// Print CLI version information
    Version,
    /// Sync a GitHub repository with an AgilePlus project
    Sync(SyncArgs),
    /// Seed FR/NFR catalogs as Epics + Stories (Tracera traceability)
    SeedRequirements(commands::seed_requirements::SeedRequirementsArgs),
    /// List projects stored in the local database
    ListProjects(commands::list_projects::ListProjectsArgs),
    /// List epics, optionally filtered by project
    ListEpics(commands::list_epics::ListEpicsArgs),
    /// List stories, optionally filtered by epic and/or status
    ListStories(commands::list_stories::ListStoriesArgs),
    /// Project management (MVP)
    Project {
        #[command(subcommand)]
        sub: commands::mvp::ProjectCmd,
    },
    /// Epic management (MVP)
    Epic {
        #[command(subcommand)]
        sub: commands::mvp::EpicCmd,
    },
    /// Story management (MVP)
    Story {
        #[command(subcommand)]
        sub: commands::mvp::StoryCmd,
    },
    /// Work-package management (MVP)
    Wp {
        #[command(subcommand)]
        sub: commands::mvp::WpCmd,
    },
    /// Work-package dependency management (MVP)
    Dep {
        #[command(subcommand)]
        sub: commands::mvp::DepCmd,
    },
    /// Transition a work-package or story to a new state (MVP)
    Transition(commands::mvp::TransitionArgs),
    /// List planned work packages that are ready to start (MVP)
    NextReady(commands::mvp::NextReadyArgs),
}

#[derive(Subcommand)]
enum FeatureCmd {
    /// List all features
    List,
    /// Show detail for a feature by id
    Show {
        /// Feature id
        id: i64,
    },
    /// Count features, optionally filtered by state
    Count {
        /// Optional state filter (created, specified, researched, planned,
        /// implementing, validated, shipped, retrospected)
        #[arg(long, value_name = "STATE")]
        state: Option<String>,
    },
    /// Search features by slug, name, or label substring
    Search {
        /// Substring to match against slug, friendly name, or labels
        query: String,
    },
    /// List features whose state is `validated` (ready to ship)
    Ready,
}

#[derive(Subcommand)]
enum ModuleCmd {
    /// List all modules
    List,
    /// Show detail for a module by id
    Show {
        /// Module id
        id: i64,
    },
    /// Search modules by slug or friendly name
    Search {
        /// Substring to match against slug or friendly name
        query: String,
    },
}

#[derive(Subcommand)]
enum CycleCmd {
    /// Show the current (active) cycle
    Current,
    /// Create a cycle
    Create(commands::mvp::CycleCreateArgs),
    /// Add a story (or all stories of an epic) to a cycle
    Add(commands::mvp::CycleAddArgs),
}

// ── in-memory mock store ─────────────────────────────────────────────────────

struct MockStore {
    features: Vec<Feature>,
    modules: Vec<Module>,
    cycles: Vec<Cycle>,
}

impl MockStore {
    fn seed() -> Self {
        let start = NaiveDate::from_ymd_opt(2026, 5, 26).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();

        let mut f1 = Feature::new("feat-cli-bootstrap", "CLI Bootstrap", [0u8; 32], None);
        f1.id = 1;
        f1.module_id = Some(1);

        let mut f2 = Feature::new(
            "feat-domain-events",
            "Domain Events",
            [1u8; 32],
            Some("feat/domain-events"),
        );
        f2.id = 2;
        f2.state = FeatureState::Specified;
        f2.module_id = Some(1);

        let mut f3 = Feature::new(
            "feat-sqlite-persistence",
            "SQLite Persistence",
            [2u8; 32],
            None,
        );
        f3.id = 3;
        f3.state = FeatureState::Planned;
        f3.module_id = Some(2);

        let mut m1 = Module::new("Core Platform", None);
        m1.id = 1;
        m1.description = Some("Core domain and CLI components".to_string());

        let mut m2 = Module::new("Persistence", None);
        m2.id = 2;
        m2.description = Some("Storage adapters".to_string());

        let mut cycle = Cycle::new("Sprint 1", start, end, None).unwrap();
        cycle.id = 1;
        cycle.state = CycleState::Active;

        MockStore {
            features: vec![f1, f2, f3],
            modules: vec![m1, m2],
            cycles: vec![cycle],
        }
    }
}

// ── handlers ─────────────────────────────────────────────────────────────────

#[allow(clippy::print_literal)] // table header uses literal strings
fn cmd_feature_list(store: &MockStore) {
    println!("{:<5} {:<28} {:<14} {}", "ID", "SLUG", "STATE", "NAME");
    println!("{}", "-".repeat(70));
    for f in &store.features {
        println!(
            "{:<5} {:<28} {:<14} {}",
            f.id, f.slug, f.state, f.friendly_name
        );
    }
}

fn cmd_feature_show(store: &MockStore, id: i64) {
    match store.features.iter().find(|f| f.id == id) {
        Some(f) => {
            println!("id           : {}", f.id);
            println!("slug         : {}", f.slug);
            println!("name         : {}", f.friendly_name);
            println!("state        : {}", f.state);
            println!("target_branch: {}", f.target_branch);
            println!(
                "module_id    : {}",
                f.module_id
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "\u{2014}".to_string())
            );
            println!("labels       : [{}]", f.labels.join(", "));
            println!(
                "created_at   : {}",
                f.created_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!(
                "updated_at   : {}",
                f.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
        }
        None => eprintln!("error: feature {id} not found"),
    }
}

#[allow(clippy::print_literal)] // table header uses literal strings
fn cmd_module_list(store: &MockStore) {
    println!("{:<5} {:<20} {}", "ID", "SLUG", "NAME");
    println!("{}", "-".repeat(50));
    for m in &store.modules {
        println!("{:<5} {:<20} {}", m.id, m.slug, m.friendly_name);
    }
}

fn cmd_cycle_current(store: &MockStore) {
    match store.cycles.iter().find(|c| c.state == CycleState::Active) {
        Some(c) => {
            println!("id    : {}", c.id);
            println!("name  : {}", c.name);
            println!("state : {}", c.state);
            println!("start : {}", c.start_date);
            println!("end   : {}", c.end_date);
        }
        None => println!("no active cycle"),
    }
}

fn cmd_cycle_list(store: &MockStore) {
    if store.cycles.is_empty() {
        println!("No cycles found.");
        return;
    }
    println!(
        "{:<5} {:<24} {:<12} {:<12} {:<12}",
        "ID", "NAME", "STATE", "START", "END"
    );
    println!("{}", "-".repeat(70));
    let mut cycles: Vec<&Cycle> = store.cycles.iter().collect();
    cycles.sort_by_key(|c| c.start_date);
    for c in cycles {
        println!(
            "{:<5} {:<24} {:<12} {:<12} {:<12}",
            c.id,
            truncate(&c.name, 24),
            c.state,
            c.start_date,
            c.end_date
        );
    }
}

fn cmd_cycle_set(store: &MockStore, id: i64) -> anyhow::Result<()> {
    let target = match store.cycles.iter().find(|c| c.id == id) {
        Some(c) => c,
        None => {
            anyhow::bail!("cycle {id} not found");
        }
    };
    if target.state == CycleState::Active {
        println!("Cycle {} ({}) is already active.", target.id, target.name);
        return Ok(());
    }
    if !matches!(target.state, CycleState::Draft | CycleState::Review) {
        anyhow::bail!(
            "cycle {} is in state `{}` and cannot be activated",
            target.id,
            target.state
        );
    }
    let active_count = store
        .cycles
        .iter()
        .filter(|c| c.state == CycleState::Active)
        .count();
    println!(
        "Cycle {} ({}) is eligible for activation. Currently {} active cycle(s).",
        target.id, target.name, active_count
    );
    Ok(())
}

fn cmd_feature_count(store: &MockStore, state: Option<&str>) -> anyhow::Result<()> {
    let parsed_state = match state {
        Some(raw) => Some(
            raw.parse::<FeatureState>()
                .map_err(|e| anyhow::anyhow!("invalid --state `{raw}`: {e}"))?,
        ),
        None => None,
    };
    let total = store.features.len();
    let mut by_state: std::collections::HashMap<FeatureState, usize> =
        std::collections::HashMap::new();
    for f in &store.features {
        *by_state.entry(f.state).or_insert(0) += 1;
    }
    match parsed_state {
        Some(s) => {
            let n = by_state.get(&s).copied().unwrap_or(0);
            println!("{n}");
        }
        None => {
            // Stable column ordering: iterate all known states first, then
            // any states that exist in data but not in the canonical list.
            let mut states: Vec<FeatureState> = by_state.keys().copied().collect();
            states.sort_by_key(|s| format!("{s}"));
            println!("{:<14} COUNT", "STATE");
            println!("{}", "-".repeat(22));
            for s in &states {
                println!("{:<14} {}", s, by_state.get(s).copied().unwrap_or(0));
            }
            println!("{}", "-".repeat(22));
            println!("{:<14} {}", "TOTAL", total);
        }
    }
    Ok(())
}

fn cmd_feature_search(store: &MockStore, query: &str) {
    let needle = query.to_lowercase();
    let matches: Vec<&Feature> = store
        .features
        .iter()
        .filter(|f| {
            f.slug.to_lowercase().contains(&needle)
                || f.friendly_name.to_lowercase().contains(&needle)
                || f.labels.iter().any(|l| l.to_lowercase().contains(&needle))
        })
        .collect();
    if matches.is_empty() {
        println!("No features matched `{query}`.");
        return;
    }
    println!("{:<5} {:<28} {:<14} NAME", "ID", "SLUG", "STATE");
    println!("{}", "-".repeat(70));
    for f in matches {
        println!(
            "{:<5} {:<28} {:<14} {}",
            f.id, f.slug, f.state, f.friendly_name
        );
    }
}

fn cmd_feature_ready(store: &MockStore) {
    let ready: Vec<&Feature> = store
        .features
        .iter()
        .filter(|f| f.state == FeatureState::Validated)
        .collect();
    if ready.is_empty() {
        println!("No features are currently in the `validated` state.");
        return;
    }
    println!("{:<5} {:<28} NAME", "ID", "SLUG");
    println!("{}", "-".repeat(50));
    for f in ready {
        println!("{:<5} {:<28} {}", f.id, f.slug, f.friendly_name);
    }
}

fn cmd_module_show(store: &MockStore, id: i64) -> anyhow::Result<()> {
    match store.modules.iter().find(|m| m.id == id) {
        Some(m) => {
            println!("id          : {}", m.id);
            println!("slug        : {}", m.slug);
            println!("name        : {}", m.friendly_name);
            println!(
                "description : {}",
                m.description
                    .clone()
                    .unwrap_or_else(|| "\u{2014}".to_string())
            );
            let feature_count = store
                .features
                .iter()
                .filter(|f| f.module_id == Some(m.id))
                .count();
            println!("features    : {feature_count}");
            println!(
                "created_at  : {}",
                m.created_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!(
                "updated_at  : {}",
                m.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
            Ok(())
        }
        None => anyhow::bail!("module {id} not found"),
    }
}

fn cmd_module_search(store: &MockStore, query: &str) {
    let needle = query.to_lowercase();
    let matches: Vec<&Module> = store
        .modules
        .iter()
        .filter(|m| {
            m.slug.to_lowercase().contains(&needle)
                || m.friendly_name.to_lowercase().contains(&needle)
        })
        .collect();
    if matches.is_empty() {
        println!("No modules matched `{query}`.");
        return;
    }
    println!("{:<5} {:<20} NAME", "ID", "SLUG");
    println!("{}", "-".repeat(50));
    for m in matches {
        println!("{:<5} {:<20} {}", m.id, m.slug, m.friendly_name);
    }
}

fn cmd_status(store: &MockStore) {
    let total_features = store.features.len();
    let mut by_state: std::collections::HashMap<FeatureState, usize> =
        std::collections::HashMap::new();
    for f in &store.features {
        *by_state.entry(f.state).or_insert(0) += 1;
    }
    let active = store.cycles.iter().find(|c| c.state == CycleState::Active);
    let total_modules = store.modules.len();
    let total_cycles = store.cycles.len();

    let active_label = match active {
        Some(c) => format!("{} ({} -> {})", c.name, c.start_date, c.end_date),
        None => "\u{2014}".to_string(),
    };

    println!("AgilePlus project status");
    println!("{}", "=".repeat(40));
    println!("Modules : {total_modules}");
    println!("Features: {total_features}");
    println!("Cycles  : {total_cycles}");
    println!();
    println!("Active cycle: {active_label}");
    println!();
    println!("Features by state:");
    for (s, n) in &by_state {
        println!("  {s:<14} {n}");
    }
}

fn cmd_version() {
    println!("agileplus-cli v{}", env!("CARGO_PKG_VERSION"));
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Truncate a string to at most `max` visible characters, appending `…` if
/// the input was longer.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let t: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{t}…")
}

/// Resolve the SQLite database path from `AGILEPLUS_DB` env var or fall back
/// to `./agileplus.db` in the current directory.
fn db_path(cli: &Cli) -> PathBuf {
    cli.db
        .clone()
        .unwrap_or_else(|| PathBuf::from("agileplus.db"))
}

/// Open the file-backed SQLite storage adapter at the resolved DB path.
fn open_storage(
    db_path: &std::path::Path,
) -> anyhow::Result<agileplus_sqlite::SqliteStorageAdapter> {
    agileplus_sqlite::SqliteStorageAdapter::new(db_path)
        .map_err(|e| anyhow::anyhow!("open db: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_store_seed_contains_cli_fixtures() {
        let store = MockStore::seed();

        assert_eq!(store.features.len(), 3);
        assert_eq!(store.modules.len(), 2);
        assert_eq!(store.cycles.len(), 1);
        assert_eq!(store.cycles[0].state, CycleState::Active);
    }

    #[test]
    fn db_path_defaults_when_env_missing() {
        std::env::remove_var("AGILEPLUS_DB");
        assert_eq!(db_path_from_env(), PathBuf::from("agileplus.db"));
    }

    #[test]
    fn db_path_uses_env_override() {
        std::env::set_var("AGILEPLUS_DB", "/tmp/agileplus-test.db");
        assert_eq!(db_path_from_env(), PathBuf::from("/tmp/agileplus-test.db"));
        std::env::remove_var("AGILEPLUS_DB");
    }
}

// ── entry point ──────────────────────────────────────────────────────────────

#[allow(clippy::items_after_test_module)]
#[tokio::main]
async fn main() {
    let _telemetry = agileplus_telemetry::init_subscriber().ok();
    let cli = Cli::parse();
    let db_path = db_path(&cli);
    let store = MockStore::seed();

    let result: anyhow::Result<()> = async {
        match cli.command {
            Command::Feature { sub } => match sub {
                FeatureCmd::List => cmd_feature_list(&store),
                FeatureCmd::Show { id } => cmd_feature_show(&store, id),
                FeatureCmd::Count { state } => cmd_feature_count(&store, state.as_deref())?,
                FeatureCmd::Search { query } => cmd_feature_search(&store, &query),
                FeatureCmd::Ready => cmd_feature_ready(&store),
            },
            Command::Module { sub } => match sub {
                ModuleCmd::List => cmd_module_list(&store),
                ModuleCmd::Show { id } => cmd_module_show(&store, id)?,
                ModuleCmd::Search { query } => cmd_module_search(&store, &query),
            },
            Command::Cycle { sub } => match sub {
                CycleCmd::Current => cmd_cycle_current(&store),
                CycleCmd::Create(args) => {
                    let storage = open_storage(&db_path)?;
                    commands::mvp::cycle_create(&args, &storage).await?;
                }
                CycleCmd::Add(args) => {
                    let storage = open_storage(&db_path)?;
                    commands::mvp::cycle_add(&args, &storage).await?;
                }
            },
            Command::Status => cmd_status(&store),
            Command::Version => cmd_version(),
            Command::Sync(args) => {
                sync_cmd::run(args, None, None).await?;
            }
            Command::SeedRequirements(args) => {
                commands::seed_requirements::run(&args)?;
            }
            Command::ListProjects(args) => {
                let storage = open_storage(&db_path)?;
                commands::list_projects::run(&args, &storage).await?;
            }
            Command::ListEpics(args) => {
                let storage = open_storage(&db_path)?;
                commands::list_epics::run(&args, &storage).await?;
            }
            Command::ListStories(args) => {
                let storage = open_storage(&db_path)?;
                commands::list_stories::run(&args, &storage).await?;
            }
            Command::Worklog(args) => {
                let db_path = db_path_from_env();
                commands::worklog::run_with_db(&args, &db_path)?;
            }
        }
        Ok(())
    }
    .await;

    if let Err(e) = result {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
