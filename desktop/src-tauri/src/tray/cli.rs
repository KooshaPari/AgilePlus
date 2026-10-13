// SPDX-License-Identifier: MIT OR Apache-2.0
//! CLI bridge for tray actions.

use std::path::Path;
use std::time::Duration;

use crate::cli_runtime;

/// Default timeout for tray CLI commands.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Run an `agileplus` CLI command and return stdout on success.
pub fn run_cli_command(args: &[&str], repo_path: &Path) -> Result<String, String> {
    run_cli_command_with_timeout(args, repo_path, DEFAULT_TIMEOUT)
}

/// Run a CLI command with a custom timeout.
pub fn run_cli_command_with_timeout(
    args: &[&str],
    repo_path: &Path,
    timeout: Duration,
) -> Result<String, String> {
    let args: Vec<String> = args
        .iter()
        .map(|argument| (*argument).to_string())
        .collect();
    let output = cli_runtime::run_agileplus(&args, repo_path, timeout)?;

    if output.code == 0 {
        Ok(output.stdout)
    } else {
        Err(format!(
            "AgilePlus CLI failed with exit code {}: {}",
            output.code,
            output.stderr.trim()
        ))
    }
}

/// Convenience wrapper with the default timeout.
pub fn run_cli_with_timeout(args: &[&str], repo_path: &Path) -> Result<String, String> {
    run_cli_command(args, repo_path)
}
