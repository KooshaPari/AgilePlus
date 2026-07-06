// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for the `ap cockpit` subcommand.
//!
//! These shell out to the built `agileplus` binary, point it at the
//! `agileplus-cli` crate directory, score against the workspace-bundled
//! rubric catalog, and verify the per-cluster records land as one-line
//! JSON in the requested NDJSON output file.

use std::path::PathBuf;

use assert_cmd::Command;

fn self_repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn cli() -> Command {
    Command::cargo_bin("agileplus").expect("agileplus binary should be built")
}

#[test]
fn cockpit_publish_writes_ndjson_with_one_record_per_cluster() {
    let repo = self_repo();
    let tmp = tempfile::tempdir().expect("tempdir");
    let log = tmp.path().join("cockpit.ndjson");

    cli()
        .args([
            "cockpit",
            "publish",
            "--repo",
            repo.to_str().unwrap(),
            "--output",
            log.to_str().unwrap(),
            "--clusters",
            "C03,C04",
        ])
        .assert()
        .success();

    let text = std::fs::read_to_string(&log).expect("log should exist");
    let lines: Vec<&str> = text.lines().collect();
    // ≥2 clusters yielded ≥2 lines; the exact count depends on whether
    // C03/C04 are present in the bundled catalog. Whichever count we got,
    // every line must be valid JSON with the expected shape.
    assert!(lines.len() >= 2, "expected ≥2 records, got {}", lines.len());
    for line in &lines {
        let v: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("invalid JSON line: {e}\n{line}"));
        assert!(v.get("ts").is_some());
        assert!(v.get("repo").is_some());
        assert!(v.get("cluster").is_some());
        assert!(v.get("score").is_some());
        assert!(v.get("max").is_some());
        assert!(v.get("grade").is_some());
        assert!(v.get("probes").is_some());
    }
}

#[test]
fn cockpit_publish_appends_to_existing_log() {
    // Two consecutive publish invocations against the same log file
    // must both succeed and double the record count.
    let repo = self_repo();
    let tmp = tempfile::tempdir().expect("tempdir");
    let log = tmp.path().join("cockpit.ndjson");

    for _ in 0..2 {
        cli()
            .args([
                "cockpit",
                "publish",
                "--repo",
                repo.to_str().unwrap(),
                "--output",
                log.to_str().unwrap(),
                "--clusters",
                "C01",
            ])
            .assert()
            .success();
    }

    let text = std::fs::read_to_string(&log).expect("log");
    let n = text.lines().count();
    // At least one cluster record per invocation × 2 = ≥2 total lines.
    assert!(n >= 2, "expected ≥2 lines after 2 publishes, got {n}");
}

#[test]
fn cockpit_path_subcommand_prints_resolved_log_path() {
    let output = cli()
        .args(["cockpit", "path"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("agileplus"), "unexpected path: {stdout}");
    assert!(
        stdout.contains("cockpit.ndjson"),
        "expected cockpit.ndjson in path: {stdout}"
    );
}

#[test]
fn cockpit_publish_help_lists_required_flags() {
    let output = cli()
        .args(["cockpit", "publish", "--help"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--repo"), "missing --repo flag: {stdout}");
    assert!(stdout.contains("--output"), "missing --output flag: {stdout}");
    assert!(stdout.contains("--clusters"), "missing --clusters flag: {stdout}");
}
