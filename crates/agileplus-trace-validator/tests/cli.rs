use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

#[test]
fn validate_accepts_trace_directory() {
    let repo = fixture_repo();

    Command::cargo_bin("agileplus-trace-validator")
        .unwrap()
        .args(["validate", repo.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("validated 1 trace files"));
}

#[test]
fn stats_prints_trace_counts() {
    let repo = fixture_repo();

    Command::cargo_bin("agileplus-trace-validator")
        .unwrap()
        .args(["stats", repo.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("traces: 1"))
        .stdout(contains("references: 4"));
}

fn fixture_repo() -> TempDir {
    let repo = tempfile::tempdir().unwrap();
    fs::create_dir(repo.path().join("traces")).unwrap();
    fs::create_dir_all(repo.path().join("docs/operations/journeys")).unwrap();
    fs::create_dir_all(repo.path().join("src")).unwrap();

    fs::write(repo.path().join("docs/traceability.md"), "trace docs").unwrap();
    fs::write(
        repo.path().join("docs/operations/journeys/FR-1.md"),
        "journey",
    )
    .unwrap();
    fs::write(repo.path().join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(repo.path().join("FUNCTIONAL_REQUIREMENTS.md"), "- FR-1").unwrap();
    fs::write(
        repo.path().join("traces/FR-1.json"),
        r##"{
  "fr_id": "FR-1",
  "spec_slug": "eco-024-traceability",
  "spec_anchor": "#fr-1",
  "docs_pages": ["docs/traceability.md"],
  "tests": ["tests/cli.rs::validate_accepts_trace_directory"],
  "code_modules": ["src/main.rs"],
  "journeys": ["docs/operations/journeys/FR-1.md"]
}"##,
    )
    .unwrap();

    repo
}
