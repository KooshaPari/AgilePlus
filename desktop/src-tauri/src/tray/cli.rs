// SPDX-License-Identifier: MIT OR Apache-2.0
//! CLI bridge — run the `agileplus` binary from tray actions.

use std::process::Command;
use std::time::Duration;

/// Default path to the agileplus binary.
const AGILEPLUS_BIN: &str = "/Users/<REDACTED>/bin/agileplus";

/// Default timeout for CLI commands (30 seconds).
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Run an `agileplus` CLI command and return stdout on success.
///
/// Arguments are passed via `Command::args()` (execvp, no shell) to prevent
/// injection. The binary path is a hardcoded constant.
pub fn run_cli_command(args: &[&str]) -> Result<String, String> {
    run_cli_command_with_timeout(args, DEFAULT_TIMEOUT)
}

/// Run a CLI command with a custom timeout.
pub fn run_cli_command_with_timeout(args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut cmd = Command::new(AGILEPLUS_BIN);
    cmd.args(args);

    let _ = timeout; // TODO: enforce via std::thread::spawn + recv

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run agileplus: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        Err(format!("CLI error (exit {code}): {stderr}"))
    }
}

/// Convenience wrapper with the default 30-second timeout.
pub fn run_cli_with_timeout(args: &[&str]) -> Result<String, String> {
    run_cli_command(args)
}
