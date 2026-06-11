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
    version
)]
struct Cli {
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
}

#[derive(Subcommand)]
enum ModuleCmd {
    /// List all modules
    List,
}

#[derive(Subcommand)]
enum CycleCmd {
    /// Show the current (active) cycle
    Current,
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

fn cmd_version() {
    println!("agileplus-cli {}", env!("CARGO_PKG_VERSION"));
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Resolve the SQLite database path from `AGILEPLUS_DB` env var or fall back
/// to `./agileplus.db` in the current directory.
fn db_path_from_env() -> PathBuf {
    std::env::var("AGILEPLUS_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("agileplus.db"))
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

#[tokio::main]
async fn main() {
    let _telemetry = agileplus_telemetry::init_subscriber().ok();
    let cli = Cli::parse();
    let store = MockStore::seed();

    let result: anyhow::Result<()> = async {
        match cli.command {
            Command::Feature { sub } => match sub {
                FeatureCmd::List => cmd_feature_list(&store),
                FeatureCmd::Show { id } => cmd_feature_show(&store, id),
            },
            Command::Module { sub } => match sub {
                ModuleCmd::List => cmd_module_list(&store),
            },
            Command::Cycle { sub } => match sub {
                CycleCmd::Current => cmd_cycle_current(&store),
            },
            Command::Version => cmd_version(),
            Command::Sync(args) => {
                sync_cmd::run(args, None).await?;
            }
            Command::SeedRequirements(args) => {
                commands::seed_requirements::run(&args)?;
            }
            Command::ListProjects(args) => {
                let db_path = db_path_from_env();
                let storage = agileplus_sqlite::SqliteStorageAdapter::new(&db_path)
                    .map_err(|e| anyhow::anyhow!("open db: {e}"))?;
                commands::list_projects::run(&args, &storage).await?;
            }
            Command::ListEpics(args) => {
                let db_path = db_path_from_env();
                let storage = agileplus_sqlite::SqliteStorageAdapter::new(&db_path)
                    .map_err(|e| anyhow::anyhow!("open db: {e}"))?;
                commands::list_epics::run(&args, &storage).await?;
            }
            Command::ListStories(args) => {
                let db_path = db_path_from_env();
                let storage = agileplus_sqlite::SqliteStorageAdapter::new(&db_path)
                    .map_err(|e| anyhow::anyhow!("open db: {e}"))?;
                commands::list_stories::run(&args, &storage).await?;
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
