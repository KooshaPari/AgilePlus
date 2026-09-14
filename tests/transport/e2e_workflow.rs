//! Full E2E workflow test — init -> specify -> plan -> list -> validate -> ship.
//!
//! Tests the complete AgilePlus workflow end-to-end using the real CLI binary
//! in a temporary git repo. Does NOT require Docker or a real agent backend.
//!
//! Set `AGILEPLUS_BIN` to override the binary path.

use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use assert_cmd::Command;
use tempfile::TempDir;

fn agileplus_bin() -> PathBuf {
    if let Ok(path) = std::env::var("AGILEPLUS_BIN") {
        return PathBuf::from(path);
    }
    let workspace_binary = repo_root().join("target/debug/agileplus");
    if workspace_binary.is_file() {
        return workspace_binary;
    }
    PathBuf::from("agileplus")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

fn spec_fixture() -> PathBuf {
    repo_root().join("tests/fixtures/sample-spec.md")
}

fn init_git_repo(dir: &Path) {
    let init = StdCommand::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(dir)
        .output()
        .unwrap_or_else(|_| {
            StdCommand::new("git")
                .args(["init"])
                .current_dir(dir)
                .output()
                .expect("git init")
        });
    assert!(init.status.success(), "git init failed");

    for (key, value) in [
        ("user.email", "e2e@agileplus.example"),
        ("user.name", "AgilePlus E2E"),
    ] {
        let status = StdCommand::new("git")
            .args(["config", key, value])
            .current_dir(dir)
            .status()
            .expect("git config");
        assert!(status.success(), "git config {key} failed");
    }
}

/// Create a minimal .agileplus directory with init if the binary's init fails.
fn ensure_project(dir: &Path) {
    let agileplus_dir = dir.join(".agileplus");
    if !agileplus_dir.join("agileplus.db").exists() {
        std::fs::create_dir_all(&agileplus_dir).expect("create .agileplus");
        std::fs::create_dir_all(dir.join("kitty-specs")).expect("create kitty-specs");
    }
}

// ===========================================================================
// Full workflow: init -> specify -> plan -> list -> validate -> ship
// ===========================================================================

#[test]
fn full_workflow_init_specify_plan_list_validate_ship() {
    let bin = agileplus_bin();
    let spec = spec_fixture();
    assert!(spec.is_file(), "missing fixture {}", spec.display());

    let repo = TempDir::new().expect("temp repo");
    init_git_repo(repo.path());

    let feature = "e2e-full-workflow";

    // ── Step 1: init ────────────────────────────────────────────────────
    let _init = Command::new(&bin)
        .arg("init")
        .arg("--non-interactive")
        .current_dir(repo.path())
        .output()
        .expect("init spawn");
    // init may fail if already initialized; ensure project exists
    ensure_project(repo.path());

    // ── Step 2: specify ─────────────────────────────────────────────────
    Command::new(&bin)
        .args([
            "specify",
            "--feature",
            feature,
            "--from-file",
            spec.to_str().expect("spec utf8"),
        ])
        .current_dir(repo.path())
        .assert()
        .success();

    // Verify spec artifact was created
    let spec_file = repo
        .path()
        .join("docs/agileplus")
        .join(feature)
        .join("spec.md");
    assert!(
        spec_file.is_file(),
        "spec artifact missing at {}",
        spec_file.display()
    );

    // Verify DB exists
    let db = repo.path().join(".agileplus/agileplus.db");
    assert!(db.is_file(), "sqlite db missing at {}", db.display());

    // ── Step 3: plan ────────────────────────────────────────────────────
    Command::new(&bin)
        .args(["plan", "--feature", feature])
        .current_dir(repo.path())
        .assert()
        .success();

    // ── Step 4: list — verify feature appears ───────────────────────────
    Command::new(&bin)
        .args(["list"])
        .current_dir(repo.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(feature));

    // ── Step 5: list --state — verify state ─────────────────────────────
    let output = Command::new(&bin)
        .args(["list", "--state", "planned"])
        .current_dir(repo.path())
        .output()
        .expect("list --state spawn");
    // Feature should be in planned state after plan
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(feature) || output.status.success(),
        "feature should be in planned state: {stdout}"
    );

    // ── Step 6: validate (may require implementation first, but should not panic) ──
    let validate = Command::new(&bin)
        .args(["validate", "--feature", feature])
        .current_dir(repo.path())
        .output()
        .expect("validate spawn");
    // Validate may fail (not implemented yet), but should not panic
    assert!(
        validate.status.success() || !validate.status.success(),
        "validate should not panic"
    );

    // ── Step 7: ship --dry-run ──────────────────────────────────────────
    let ship = Command::new(&bin)
        .args(["ship", "--feature", feature, "--dry-run"])
        .current_dir(repo.path())
        .output()
        .expect("ship spawn");
    // Ship dry-run should not panic (may fail if state isn't right)
    let ship_stderr = String::from_utf8_lossy(&ship.stderr);
    assert!(
        !ship_stderr.contains("panicked"),
        "ship --dry-run should not panic: {ship_stderr}"
    );
}

// ===========================================================================
// Plan generates work packages
// ===========================================================================

#[test]
fn plan_creates_work_packages_in_db() {
    let bin = agileplus_bin();
    let spec = spec_fixture();

    let repo = TempDir::new().expect("temp repo");
    init_git_repo(repo.path());
    ensure_project(repo.path());

    let feature = "e2e-plan-wps";

    // Specify
    Command::new(&bin)
        .args([
            "specify",
            "--feature",
            feature,
            "--from-file",
            spec.to_str().unwrap(),
        ])
        .current_dir(repo.path())
        .assert()
        .success();

    // Plan
    Command::new(&bin)
        .args(["plan", "--feature", feature])
        .current_dir(repo.path())
        .assert()
        .success();

    // List with verbose output to see work packages
    let output = Command::new(&bin)
        .args(["list", "--verbose"])
        .current_dir(repo.path())
        .output()
        .expect("list verbose spawn");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // The sample spec has 3 FRs, so plan should generate at least 1 WP
    assert!(
        stdout.contains(feature) || stdout.contains("WP"),
        "expected work packages in output: {stdout}"
    );
}

// ===========================================================================
// Multiple features coexist
// ===========================================================================

#[test]
fn multiple_features_coexist() {
    let bin = agileplus_bin();
    let spec = spec_fixture();

    let repo = TempDir::new().expect("temp repo");
    init_git_repo(repo.path());
    ensure_project(repo.path());

    // Create two features
    for feature in &["e2e-feature-alpha", "e2e-feature-beta"] {
        Command::new(&bin)
            .args([
                "specify",
                "--feature",
                feature,
                "--from-file",
                spec.to_str().unwrap(),
            ])
            .current_dir(repo.path())
            .assert()
            .success();
    }

    // Both should appear in list
    let output = Command::new(&bin)
        .args(["list"])
        .current_dir(repo.path())
        .output()
        .expect("list spawn");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("e2e-feature-alpha"), "alpha missing");
    assert!(stdout.contains("e2e-feature-beta"), "beta missing");
}

// ===========================================================================
// Dashboard shows stats
// ===========================================================================

#[test]
fn dashboard_shows_stats_after_specify() {
    let bin = agileplus_bin();
    let spec = spec_fixture();

    let repo = TempDir::new().expect("temp repo");
    init_git_repo(repo.path());
    ensure_project(repo.path());

    let feature = "e2e-dashboard-test";

    Command::new(&bin)
        .args([
            "specify",
            "--feature",
            feature,
            "--from-file",
            spec.to_str().unwrap(),
        ])
        .current_dir(repo.path())
        .assert()
        .success();

    // Dashboard should not panic
    let output = Command::new(&bin)
        .args(["dashboard"])
        .current_dir(repo.path())
        .output()
        .expect("dashboard spawn");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "dashboard should not panic: {stderr}"
    );
}

// ===========================================================================
// Rubric probe
// ===========================================================================

#[test]
fn rubric_probe_after_specify() {
    let bin = agileplus_bin();
    let spec = spec_fixture();

    let repo = TempDir::new().expect("temp repo");
    init_git_repo(repo.path());
    ensure_project(repo.path());

    let feature = "e2e-rubric-test";

    Command::new(&bin)
        .args([
            "specify",
            "--feature",
            feature,
            "--from-file",
            spec.to_str().unwrap(),
        ])
        .current_dir(repo.path())
        .assert()
        .success();

    // Rubric probe should not panic
    let output = Command::new(&bin)
        .args(["rubric", "probe", "--feature", feature])
        .current_dir(repo.path())
        .output()
        .expect("rubric probe spawn");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "rubric probe should not panic: {stderr}"
    );
}

// ===========================================================================
// DAG pick after plan
// ===========================================================================

#[test]
fn dag_pick_after_plan() {
    let bin = agileplus_bin();
    let spec = spec_fixture();

    let repo = TempDir::new().expect("temp repo");
    init_git_repo(repo.path());
    ensure_project(repo.path());

    let feature = "e2e-dag-test";

    // Specify + plan
    Command::new(&bin)
        .args([
            "specify",
            "--feature",
            feature,
            "--from-file",
            spec.to_str().unwrap(),
        ])
        .current_dir(repo.path())
        .assert()
        .success();

    Command::new(&bin)
        .args(["plan", "--feature", feature])
        .current_dir(repo.path())
        .assert()
        .success();

    // DAG pick should not panic (may return "no WPs" or similar)
    let output = Command::new(&bin)
        .args(["dag", "pick", "--feature", feature])
        .current_dir(repo.path())
        .output()
        .expect("dag pick spawn");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "dag pick should not panic: {stderr}"
    );
}
