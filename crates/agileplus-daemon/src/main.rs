// SPDX-License-Identifier: MIT OR Apache-2.0
//! Registry-administration entry point for the future AgilePlus device daemon.

use std::error::Error;
use std::path::PathBuf;

use agileplus_daemon::{DeviceRegistry, DesiredState, default_registry_path};
use clap::{Parser, Subcommand, ValueEnum};
use uuid::Uuid;

#[derive(Debug, Parser)]
#[command(
    name = "agileplusd",
    about = "AgilePlus device registry administration; process supervision is not enabled yet"
)]
struct Cli {
    /// Override the platform-default device-registry path.
    #[arg(long)]
    registry: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect or mutate the device project registry.
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },
}

#[derive(Debug, Subcommand)]
enum RegistryCommand {
    /// Validate the registry and print its metadata.
    Check,
    /// List all registered projects.
    List,
    /// Register one canonical Git project root.
    Register {
        /// Git worktree root to register.
        root: PathBuf,
        /// Stable identity for controlled imports; generated when omitted.
        #[arg(long)]
        project_uid: Option<Uuid>,
    },
    /// Remove a registry entry without deleting project-local state.
    Unregister {
        /// Stable project identity.
        project_uid: Uuid,
    },
    /// Change whether the future supervisor should run a project core.
    SetDesired {
        /// Stable project identity.
        project_uid: Uuid,
        /// Requested lifecycle state.
        state: DesiredStateArgument,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DesiredStateArgument {
    Running,
    Stopped,
}

impl From<DesiredStateArgument> for DesiredState {
    fn from(value: DesiredStateArgument) -> Self {
        match value {
            DesiredStateArgument::Running => Self::Running,
            DesiredStateArgument::Stopped => Self::Stopped,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let registry_path = match cli.registry {
        Some(path) => path,
        None => default_registry_path()?,
    };
    let mut registry = DeviceRegistry::open(registry_path)?;

    match cli.command {
        Command::Registry { command } => match command {
            RegistryCommand::Check => print_json(&registry.summary()?)?,
            RegistryCommand::List => print_json(&registry.list_projects()?)?,
            RegistryCommand::Register { root, project_uid } => {
                let project = match project_uid {
                    Some(project_uid) => registry.register_project_with_uid(project_uid, &root)?,
                    None => registry.register_project(&root)?,
                };
                print_json(&project)?;
            }
            RegistryCommand::Unregister { project_uid } => {
                print_json(&registry.unregister_project(project_uid)?)?;
            }
            RegistryCommand::SetDesired { project_uid, state } => {
                print_json(&registry.set_desired_state(project_uid, state.into())?)?;
            }
        },
    }
    Ok(())
}

fn print_json(value: &impl serde::Serialize) -> Result<(), serde_json::Error> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
