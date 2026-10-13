// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tauri IPC bridge to the installed `agileplus` CLI.

use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;
use tauri::State;

use crate::{AppState, cli_runtime};

/// Read command timeout: 30 seconds.
const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Lifecycle command timeout: 5 minutes.
const LIFECYCLE_TIMEOUT: Duration = Duration::from_secs(300);

/// Result of a CLI command execution.
#[derive(Debug, Serialize)]
pub struct CliResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Run a read-only CLI command with a 30-second timeout.
#[tauri::command]
pub fn run_cli_read(state: State<'_, AppState>, args: Vec<String>) -> Result<CliResult, String> {
    run_cli_command(&state, &args, READ_TIMEOUT)
}

/// Run a lifecycle CLI command with a five-minute timeout.
#[tauri::command]
pub fn run_cli_lifecycle(
    state: State<'_, AppState>,
    args: Vec<String>,
) -> Result<CliResult, String> {
    run_cli_command(&state, &args, LIFECYCLE_TIMEOUT)
}

fn run_cli_command(
    state: &State<'_, AppState>,
    args: &[String],
    timeout: Duration,
) -> Result<CliResult, String> {
    let repo_path = selected_repo_path(state)?;
    let output = cli_runtime::run_agileplus(args, &repo_path, timeout)?;
    Ok(CliResult {
        code: output.code,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

fn selected_repo_path(state: &State<'_, AppState>) -> Result<PathBuf, String> {
    let repo_path = state.repo_path();
    if repo_path.trim().is_empty() {
        return Err("No AgilePlus project is selected. Open a project first.".into());
    }
    Ok(PathBuf::from(repo_path))
}
