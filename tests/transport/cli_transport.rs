//! CLI transport tests — binary-level verification.
//!
//! Tests the `agileplus` binary for argument parsing, exit codes,
//! help output, and error handling without requiring a full git repo.

use std::process::Command;

fn agileplus_bin() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("AGILEPLUS_BIN") {
        return std::path::PathBuf::from(path);
    }
    let workspace_binary = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/debug/agileplus");
    if workspace_binary.is_file() {
        return workspace_binary;
    }
    std::path::PathBuf::from("agileplus")
}

// ===========================================================================
// Version and help output
// ===========================================================================

#[test]
fn cli_version_succeeds() {
    let output = Command::new(agileplus_bin())
        .arg("--version")
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "--version should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("agileplus"),
        "version should contain 'agileplus': {stdout}"
    );
}

#[test]
fn cli_help_succeeds() {
    let output = Command::new(agileplus_bin())
        .arg("--help")
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "--help should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Spec-driven development engine"),
        "help should describe the tool: {stdout}"
    );
}

#[test]
fn cli_help_lists_core_subcommands() {
    let output = Command::new(agileplus_bin())
        .arg("--help")
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);

    for subcmd in &["cycle", "list", "specify", "dag", "okf", "mvp", "dashboard", "rubric"] {
        assert!(
            stdout.contains(subcmd),
            "help should list subcommand '{subcmd}': {stdout}"
        );
    }
}

// ===========================================================================
// Subcommand help (each should respond to --help)
// ===========================================================================

#[test]
fn subcommand_list_help() {
    let output = Command::new(agileplus_bin())
        .args(["list", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "list --help should succeed");
}

#[test]
fn subcommand_dag_help() {
    let output = Command::new(agileplus_bin())
        .args(["dag", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "dag --help should succeed");
}

#[test]
fn subcommand_okf_help() {
    let output = Command::new(agileplus_bin())
        .args(["okf", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "okf --help should succeed");
}

#[test]
fn subcommand_mvp_help() {
    let output = Command::new(agileplus_bin())
        .args(["mvp", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "mvp --help should succeed");
}

#[test]
fn subcommand_dashboard_help() {
    let output = Command::new(agileplus_bin())
        .args(["dashboard", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "dashboard --help should succeed");
}

#[test]
fn subcommand_rubric_help() {
    let output = Command::new(agileplus_bin())
        .args(["rubric", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "rubric --help should succeed");
}

#[test]
fn subcommand_specify_help() {
    let output = Command::new(agileplus_bin())
        .args(["specify", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "specify --help should succeed");
}

#[test]
fn subcommand_cycle_help() {
    let output = Command::new(agileplus_bin())
        .args(["cycle", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "cycle --help should succeed");
}

#[test]
fn subcommand_queue_help() {
    let output = Command::new(agileplus_bin())
        .args(["queue", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "queue --help should succeed");
}

#[test]
fn subcommand_module_help() {
    let output = Command::new(agileplus_bin())
        .args(["module", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "module --help should succeed");
}

// ===========================================================================
// Error handling
// ===========================================================================

#[test]
fn unknown_subcommand_fails() {
    let output = Command::new(agileplus_bin())
        .args(["nonexistent-subcommand"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "unknown subcommand should fail"
    );
}

#[test]
fn outside_git_repo_returns_error() {
    let temp = tempfile::TempDir::new().unwrap();
    let output = Command::new(agileplus_bin())
        .args(["list"])
        .current_dir(temp.path())
        .output()
        .expect("failed to execute");
    // Should fail (no git repo) but not panic
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success() || stdout.contains("No features") || stderr.contains("git"),
        "outside git repo should fail gracefully: stdout={stdout} stderr={stderr}"
    );
}

// ===========================================================================
// Verbose flag
// ===========================================================================

#[test]
fn verbose_flag_accepted() {
    let output = Command::new(agileplus_bin())
        .args(["-v", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "-v --help should succeed");
}

#[test]
fn double_verbose_flag_accepted() {
    let output = Command::new(agileplus_bin())
        .args(["-vv", "--help"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success(), "-vv --help should succeed");
}
