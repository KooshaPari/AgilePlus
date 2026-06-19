//! AgilePlus CLI entry point.
//!
//! Parses CLI arguments, initialises adapters, and routes to command handlers.
//! Traceability: WP11-T060, T065 / WP12-T072

use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use agileplus_cli::commands::{
    branch::BranchArgs, cycle::CycleArgs, implement::ImplementArgs, module::ModuleArgs,
    plan::PlanArgs, queue::QueueArgs, research::ResearchArgs, retrospective::RetrospectiveArgs,
    ship::ShipArgs, specify::SpecifyArgs, triage::TriageArgs, validate::ValidateArgs,
};
use agileplus_git::GitVcsAdapter;
use agileplus_sqlite::SqliteStorageAdapter;
use agileplus_subcmds::{
    DashboardArgs, PlatformArgs, SyncArgs, run_dashboard, run_platform, run_sync,
};

mod agent_stub;
use agent_stub::StubAgentAdapter;

/// Spec-driven development engine.
#[derive(Parser)]
#[command(
    name = "agileplus",
    about = "AgilePlus project management CLI",
    version,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Path to SQLite database
    #[arg(long, global = true, default_value = ".agileplus/agileplus.db")]
    db: PathBuf,

    /// Path to git repository root (defaults to current directory)
    #[arg(long, global = true)]
    repo: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage cycles (time-boxed delivery units).
    Cycle(CycleArgs),
    /// Branch management: create, checkout, delete, list, and sync.
    Branch(BranchArgs),
    /// Create or revise a feature specification.
    Specify(SpecifyArgs),
    /// Research a feature (pre-specify codebase scan or post-specify feasibility).
    Research(ResearchArgs),
    /// Generate a plan (work packages) for a researched feature.
    Plan(PlanArgs),
    /// Implement work packages for a planned feature.
    Implement(ImplementArgs),
    /// Validate governance compliance for an implementing feature.
    Validate(ValidateArgs),
    /// Ship a validated feature by merging all WP branches.
    Ship(ShipArgs),
    /// Generate a retrospective report for a shipped feature.
    Retrospective(RetrospectiveArgs),
    /// Classify and route incoming items (bug, feature, idea, task).
    Triage(TriageArgs),
    /// Manage the triage backlog queue.
    Queue(QueueArgs),
    /// Manage modules (product-area groupings of features).
    Module(ModuleArgs),
    /// Open or configure the web dashboard.
    Dashboard(DashboardArgs),
    /// Manage platform services (up, down, status, logs).
    Platform(PlatformArgs),
    /// Sync local features/WPs with Plane.so (push, pull, auto, status, resolve).
    Sync(SyncArgs),
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
    /// List all known cycles
    List,
    /// Print which cycle would become active if id were promoted
    Set {
        /// Cycle id to set as active
        id: i64,
    },
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
    let cli = Cli::parse();

    // Configure logging based on verbosity
    let log_level = match cli.verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .compact()
        .init();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {e:#}");
        process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    // Triage command doesn't need full storage/VCS setup
    match cli.command {
        Commands::Triage(args) => return agileplus_cli::commands::triage::run_triage(args).await,
        Commands::Dashboard(args) => return run_dashboard(args),
        Commands::Platform(args) => return run_platform(args),
        Commands::Sync(args) => return run_sync(args).await,
        _ => {}
    }

    // Module command only needs storage (no VCS)
    if let Commands::Module(args) = cli.command {
        // Initialise storage adapter early for module commands
        if let Some(parent) = cli.db.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating directory {}", parent.display()))?;
            }
        }
        let storage = SqliteStorageAdapter::new(&cli.db)
            .with_context(|| format!("opening database at {}", cli.db.display()))?;
        return agileplus_cli::commands::module::run(args, &storage).await;
    }

    // Initialise storage adapter (create DB directory if needed)
    if let Some(parent) = cli.db.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating directory {}", parent.display()))?;
        }
    }

    let storage = SqliteStorageAdapter::new(&cli.db)
        .with_context(|| format!("opening database at {}", cli.db.display()))?;

    // Initialise VCS adapter
    let vcs = match cli.repo {
        Some(ref path) => {
            GitVcsAdapter::new(path.clone()).context("opening git repository at specified path")?
        }
        None => GitVcsAdapter::from_current_dir()
            .context("Not inside a git repository. Run agileplus from your project root.")?,
    };

    // Stub agent adapter (replaced by agileplus-agents when WP08 is available)
    let agent = StubAgentAdapter;

    match cli.command {
        Commands::Branch(args) => {
            agileplus_cli::commands::branch::run(args, &vcs).await?;
        }
        Commands::Cycle(args) => {
            agileplus_cli::commands::cycle::run(args, &storage).await?;
        }
        Commands::Queue(args) => {
            agileplus_cli::commands::queue::run_queue(args, &storage).await?;
        }
        Commands::Specify(args) => {
            agileplus_cli::commands::specify::run_specify(args, &storage, &vcs).await?;
        }
        Commands::Research(args) => {
            agileplus_cli::commands::research::run_research(args, &storage, &vcs).await?;
        }
        Commands::Plan(args) => {
            agileplus_cli::commands::plan::run_plan(args, &storage, &vcs).await?;
        }
        Commands::Implement(args) => {
            agileplus_cli::commands::implement::run_implement(args, &storage, &vcs, &agent).await?;
        }
        Commands::Validate(args) => {
            agileplus_cli::commands::validate::run_validate(args, &storage, &vcs).await?;
        }
        Commands::Ship(args) => {
            agileplus_cli::commands::ship::run_ship(args, &storage, &vcs).await?;
        }
        Commands::Retrospective(args) => {
            agileplus_cli::commands::retrospective::run_retrospective(args, &storage, &vcs).await?;
        }
        Commands::Triage(_)
        | Commands::Module(_)
        | Commands::Dashboard(_)
        | Commands::Platform(_)
        | Commands::Sync(_) => unreachable!("handled above"),
    }

    Ok(())
}
