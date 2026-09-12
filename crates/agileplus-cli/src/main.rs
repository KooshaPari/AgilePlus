//! AgilePlus CLI entry point — spec-driven development surface.
//!
//! Parses CLI arguments, initialises adapters, and routes to command handlers.
//! Platform health uses real HTTP/TCP probes via `agileplus-subcmds`.
//! Traceability: WP11-T060, T065 / WP12-T072 / WP14-T084..T087

mod agent_adapter;

use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use agent_adapter::RealAgentAdapter;
use agileplus_cli::commands::{
    cockpit::CockpitArgs, cycle::CycleArgs, dag::DagArgs, dashboard::DashboardArgs, list::ListArgs,
    module::ModuleArgs, mvp::MvpArgs, okf::OkfArgs, queue::QueueArgs, rubric::RubricArgs,
    specify::SpecifyArgs,
};
#[cfg(feature = "full-deps")]
use agileplus_cli::commands::{
    implement::ImplementArgs, plan::PlanArgs, research::ResearchArgs,
    retrospective::RetrospectiveArgs, ship::ShipArgs, triage::TriageArgs, validate::ValidateArgs,
};
use agileplus_git::{GitVcsAdapter, ProjectContext};
use agileplus_sqlite::SqliteStorageAdapter;
use agileplus_subcmds::{PlatformArgs, run_platform};

/// Spec-driven development engine.
#[derive(Parser)]
#[command(name = "agileplus", version, about = "Spec-driven development engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Deprecated: state is always derived from the repository root.
    #[arg(long, global = true, hide = true)]
    db: Option<PathBuf>,

    /// Path to git repository root (defaults to current directory)
    #[arg(long, global = true)]
    repo: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage cycles (time-boxed delivery units).
    Cycle(CycleArgs),
    /// List features in the database (optional filter by state).
    List(ListArgs),
    /// Create or revise a feature specification.
    Specify(SpecifyArgs),
    /// Manage the triage backlog queue.
    Queue(QueueArgs),
    /// Manage modules (product-area groupings of features).
    Module(ModuleArgs),
    /// Manage platform services (up, down, status, logs).
    Platform(PlatformArgs),
    /// Research a feature (codebase scan / feasibility).
    #[cfg(feature = "full-deps")]
    Research(ResearchArgs),
    /// Generate a delivery plan and work packages.
    #[cfg(feature = "full-deps")]
    Plan(PlanArgs),
    /// Implement work packages (dispatches agents).
    #[cfg(feature = "full-deps")]
    Implement(ImplementArgs),
    /// Validate governance evidence and policies.
    #[cfg(feature = "full-deps")]
    Validate(ValidateArgs),
    /// Ship a validated feature (merge + archive).
    #[cfg(feature = "full-deps")]
    Ship(ShipArgs),
    /// Generate a retrospective for a shipped feature.
    #[cfg(feature = "full-deps")]
    Retrospective(RetrospectiveArgs),
    /// Classify and route incoming items to the backlog.
    #[cfg(feature = "full-deps")]
    Triage(TriageArgs),
    /// Score a repo against the governance rubric catalog.
    Rubric(RubricArgs),
    /// Manage the DAG of work packages (pick, claim, release, dedup, scan).
    Dag(DagArgs),
    /// Validate, summarize, and merge OKF v1.0 documents.
    Okf(OkfArgs),
    /// MVP project/epic/story/work-package management.
    Mvp(MvpArgs),
    /// Render an in-flight DAG / status dashboard from SQLite.
    Dashboard(DashboardArgs),
    /// Publish or read local cockpit scorecards.
    Cockpit(CockpitArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

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

fn open_vcs(repo: &Option<PathBuf>) -> Result<GitVcsAdapter> {
    match repo {
        Some(path) => Ok(GitVcsAdapter::new(path.clone())),
        None => GitVcsAdapter::from_current_dir()
            .context("Not inside a git repository. Run agileplus from your project root."),
    }
}

fn project_context(repo: &Option<PathBuf>) -> Result<ProjectContext> {
    let start = match repo {
        Some(path) => path.clone(),
        None => std::env::current_dir().context("reading current directory")?,
    };
    ProjectContext::discover(&start)
        .map_err(anyhow::Error::msg)
        .context("Not inside a git repository. Run agileplus from your project root.")
}

fn repository_database(repo: &Option<PathBuf>) -> Result<PathBuf> {
    let database = project_context(repo)?.database_path();
    if let Some(parent) = database.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating repository state directory {}", parent.display()))?;
    }
    Ok(database)
}

async fn run(cli: Cli) -> Result<()> {
    if cli.db.is_some() {
        anyhow::bail!("--db is no longer supported; AgilePlus state is repository-local");
    }

    match cli.command {
        Commands::Platform(args) => run_platform(args),
        Commands::Rubric(args) => agileplus_cli::commands::rubric::run(&args),
        Commands::Dag(args) => agileplus_cli::commands::dag::run_dag(args).await,
        Commands::Okf(args) => {
            let code = agileplus_cli::commands::okf::run(&args)?;
            if code != 0 {
                process::exit(code);
            }
            Ok(())
        }
        Commands::Mvp(args) => {
            let db = repository_database(&cli.repo)?;
            let storage = SqliteStorageAdapter::new(&db)
                .with_context(|| format!("opening database at {}", db.display()))?;
            agileplus_cli::commands::mvp::run_mvp(args, &storage).await
        }
        Commands::Dashboard(mut args) => {
            // Prefer global --db when the subcommand did not set its own path.
            if args.db.is_none() {
                args.db = Some(repository_database(&cli.repo)?);
            }
            agileplus_cli::commands::dashboard::run(&args)
        }
        Commands::Module(args) => {
            let db = repository_database(&cli.repo)?;
            let storage = SqliteStorageAdapter::new(&db)
                .with_context(|| format!("opening database at {}", db.display()))?;
            agileplus_cli::commands::module::run(args, &storage).await
        }
        Commands::Cycle(args) => {
            let db = repository_database(&cli.repo)?;
            let storage = SqliteStorageAdapter::new(&db)
                .with_context(|| format!("opening database at {}", db.display()))?;
            agileplus_cli::commands::cycle::run(args, &storage).await
        }
        Commands::Queue(args) => {
            let db = repository_database(&cli.repo)?;
            let storage = SqliteStorageAdapter::new(&db)
                .with_context(|| format!("opening database at {}", db.display()))?;
            agileplus_cli::commands::queue::run_queue(args, &storage).await
        }
        Commands::List(args) => {
            let db = repository_database(&cli.repo)?;
            let storage = SqliteStorageAdapter::new(&db)
                .with_context(|| format!("opening database at {}", db.display()))?;
            agileplus_cli::commands::list::run(args, &storage).await
        }
        Commands::Specify(args) => {
            let db = repository_database(&cli.repo)?;
            let storage = SqliteStorageAdapter::new(&db)
                .with_context(|| format!("opening database at {}", db.display()))?;
            let vcs = open_vcs(&cli.repo)?;
            agileplus_cli::commands::specify::run_specify(args, &storage, &vcs).await
        }
        #[cfg(feature = "full-deps")]
        Commands::Research(args) => {
            let db = repository_database(&cli.repo)?;
            let storage = SqliteStorageAdapter::new(&db)
                .with_context(|| format!("opening database at {}", db.display()))?;
            let vcs = open_vcs(&cli.repo)?;
            agileplus_cli::commands::research::run_research(args, &storage, &vcs).await
        }
        #[cfg(feature = "full-deps")]
        Commands::Plan(args) => {
            let db = repository_database(&cli.repo)?;
            let storage = SqliteStorageAdapter::new(&db)
                .with_context(|| format!("opening database at {}", db.display()))?;
            let vcs = open_vcs(&cli.repo)?;
            agileplus_cli::commands::plan::run_plan(args, &storage, &vcs).await
        }
        #[cfg(feature = "full-deps")]
        Commands::Implement(args) => {
            let db = repository_database(&cli.repo)?;
            let storage = SqliteStorageAdapter::new(&db)
                .with_context(|| format!("opening database at {}", db.display()))?;
            let vcs = open_vcs(&cli.repo)?;
            let agent = RealAgentAdapter::new();
            agileplus_cli::commands::implement::run_implement(args, &storage, &vcs, &agent).await
        }
        #[cfg(feature = "full-deps")]
        Commands::Validate(args) => {
            let db = repository_database(&cli.repo)?;
            let storage = SqliteStorageAdapter::new(&db)
                .with_context(|| format!("opening database at {}", db.display()))?;
            let vcs = open_vcs(&cli.repo)?;
            agileplus_cli::commands::validate::run_validate(args, &storage, &vcs).await
        }
        #[cfg(feature = "full-deps")]
        Commands::Ship(args) => {
            let db = repository_database(&cli.repo)?;
            let storage = SqliteStorageAdapter::new(&db)
                .with_context(|| format!("opening database at {}", db.display()))?;
            let vcs = open_vcs(&cli.repo)?;
            agileplus_cli::commands::ship::run_ship(args, &storage, &vcs).await
        }
        #[cfg(feature = "full-deps")]
        Commands::Retrospective(args) => {
            let db = repository_database(&cli.repo)?;
            let storage = SqliteStorageAdapter::new(&db)
                .with_context(|| format!("opening database at {}", db.display()))?;
            let vcs = open_vcs(&cli.repo)?;
            agileplus_cli::commands::retrospective::run_retrospective(args, &storage, &vcs).await
        }
        #[cfg(feature = "full-deps")]
        Commands::Triage(args) => agileplus_cli::commands::triage::run_triage(args).await,
        Commands::Cockpit(args) => {
            let global_repo = cli.repo.as_deref();
            agileplus_cli::commands::cockpit::run(&args, global_repo)
        }
    }
}
