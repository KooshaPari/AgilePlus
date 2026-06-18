use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use agileplus_trace_validator::validate_trace_path;

#[derive(Debug, Parser)]
#[command(name = "agileplus-trace-validator")]
#[command(about = "Validate AgilePlus traceability metadata")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate trace files under a repo or traces directory.
    Validate { path: PathBuf },
    /// Print a simple FR to artifact graph.
    Graph { path: Option<PathBuf> },
    /// Print trace coverage statistics.
    Stats { path: Option<PathBuf> },
    /// Print missing functional requirement traces.
    Missing { path: Option<PathBuf> },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Validate { path } => {
            let validation = validate_trace_path(path)?;
            println!("validated {} trace files", validation.trace_count());
        }
        Command::Graph { path } => {
            let validation = validate_trace_path(path.unwrap_or_else(|| PathBuf::from(".")))?;
            for trace in validation.traces {
                println!(
                    "{} -> docs:{} tests:{} code:{} journeys:{}",
                    trace.fr_id,
                    trace.docs_pages.len(),
                    trace.tests.len(),
                    trace.code_modules.len(),
                    trace.journeys.len()
                );
            }
        }
        Command::Stats { path } => {
            let validation = validate_trace_path(path.unwrap_or_else(|| PathBuf::from(".")))?;
            println!("traces: {}", validation.trace_count());
            println!("references: {}", validation.referenced_path_count());
        }
        Command::Missing { path } => {
            let validation = validate_trace_path(path.unwrap_or_else(|| PathBuf::from(".")))?;
            if validation.missing_functional_requirements.is_empty() {
                println!("missing: 0");
            } else {
                for fr_id in validation.missing_functional_requirements {
                    println!("{fr_id}");
                }
            }
        }
    }

    Ok(())
}
