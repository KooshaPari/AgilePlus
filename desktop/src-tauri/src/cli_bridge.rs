// SPDX-License-Identifier: MIT OR Apache-2.0
//! CLI bridge: shell-out to the `agileplus` binary.

use serde::Serialize;
use std::process::Command;
use std::time::Duration;
use tauri::State;

use crate::AppState;

/// Default path to the agileplus binary.
const AGILEPLUS_BIN: &str = "/Users/<REDACTED>/bin/agileplus";

/// Read command timeout: 30 seconds.
const READ_TIMEOUT_SECS: u64 = 30;

/// Lifecycle command timeout: 5 minutes.
const LIFECYCLE_TIMEOUT_SECS: u64 = 300;

/// Result of a CLI command execution.
#[derive(Debug, Serialize)]
pub struct CliResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Run a read-only CLI command (30s timeout).
#[tauri::command]
pub fn run_cli_read(state: State<'_, AppState>, args: Vec<String>) -> Result<CliResult, String> {
    let repo_path = state.repo_path();
    run_cli_command(&args, &repo_path, READ_TIMEOUT_SECS)
}

/// Run a lifecycle CLI command (5min timeout).
#[tauri::command]
pub fn run_cli_lifecycle(
    state: State<'_, AppState>,
    args: Vec<String>,
) -> Result<CliResult, String> {
    let repo_path = state.repo_path();
    run_cli_command(&args, &repo_path, LIFECYCLE_TIMEOUT_SECS)
}

/// Spawn the agileplus binary with the given arguments.
///
/// Uses `std::process::Command` directly (no shell invocation) to avoid
/// shell injection. The current working directory is set to `repo_path`.
fn run_cli_command(
    args: &[String],
    repo_path: &str,
    timeout_secs: u64,
) -> Result<CliResult, String> {
    let mut cmd = Command::new(AGILEPLUS_BIN);
    cmd.args(args).current_dir(repo_path);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run agileplus CLI: {e}"))?;

    // TODO: Enforce timeout using std::thread::spawn + recv with Duration
    // For now, Command::output() is synchronous. A future improvement would
    // be to wrap this in spawn_blocking with a tokio timeout.
    let _ = Duration::from_secs(timeout_secs);

    Ok(CliResult {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
