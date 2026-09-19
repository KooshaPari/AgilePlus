// SPDX-License-Identifier: MIT OR Apache-2.0
//! Direct-call tests for the cockpit command group.
//!
//! Exercises `commands::cockpit::run` (Publish / Path / Read dispatch and the
//! global-`--repo` guards) and `commands::cockpit_read::run` (missing log,
//! populated log, repo filter, default path resolution). No `--watch` test:
//! `watch_loop` polls forever by design.

use std::path::{Path, PathBuf};

use agileplus_cli::commands::cockpit::{CockpitArgs, CockpitSubcommand, run};
use agileplus_cli::commands::cockpit_read::{CockpitReadArgs, run as read_run};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn bundled_catalog() -> PathBuf {
    manifest_dir().join("../agileplus-governance/data/PILLARS-CATALOG.json")
}

fn publish_args(repo: &Path, catalog: Option<PathBuf>, output: Option<PathBuf>) -> CockpitArgs {
    CockpitArgs {
        sub: CockpitSubcommand::Publish {
            repo: repo.to_path_buf(),
            catalog,
            clusters: Some(vec!["C03".to_string()]),
            output,
            no_probes: false,
        },
    }
}

fn read_args(input: Option<PathBuf>, filter: Option<&str>) -> CockpitArgs {
    CockpitArgs {
        sub: CockpitSubcommand::Read(CockpitReadArgs {
            filter_repo: filter.map(|s| s.to_string()),
            input,
            watch: false,
        }),
    }
}

fn write_log(path: &Path, body: &str) {
    std::fs::write(path, body).expect("write ndjson");
}

// ── cockpit publish ──────────────────────────────────────────────────────────

#[test]
fn publish_appends_one_json_record_per_cluster() {
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("cockpit.ndjson");
    let args = publish_args(&manifest_dir(), Some(bundled_catalog()), Some(log.clone()));
    run(&args, None).expect("publish should succeed");

    let text = std::fs::read_to_string(&log).expect("log written");
    let lines: Vec<&str> = text.lines().collect();
    assert!(!lines.is_empty(), "expected at least one record");
    for line in &lines {
        let v: serde_json::Value = serde_json::from_str(line).expect("valid JSON line");
        assert!(v.get("cluster").is_some());
        assert!(v.get("grade").is_some());
        assert!(v.get("probes").is_some());
    }
}

#[test]
fn publish_with_global_repo_is_allowed() {
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("cockpit.ndjson");
    let args = publish_args(&manifest_dir(), Some(bundled_catalog()), Some(log.clone()));
    run(&args, Some(Path::new("/tmp"))).expect("publish ignores global repo");
    assert!(log.exists());
}

#[test]
fn publish_missing_repo_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("cockpit.ndjson");
    let args = publish_args(
        Path::new("/nonexistent/agileplus-cockpit-repo"),
        Some(bundled_catalog()),
        Some(log),
    );
    let err = run(&args, None).expect_err("missing repo must error");
    assert!(err.to_string().contains("does not exist"), "{err}");
}

#[test]
fn publish_repo_must_be_directory() {
    let catalog = bundled_catalog();
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("cockpit.ndjson");
    let args = publish_args(&catalog, Some(catalog.clone()), Some(log));
    let err = run(&args, None).expect_err("repo as file must error");
    assert!(err.to_string().contains("must be a directory"), "{err}");
}

#[test]
fn publish_missing_catalog_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("cockpit.ndjson");
    let ghost = tmp.path().join("absent.json");
    let args = publish_args(&manifest_dir(), Some(ghost), Some(log));
    let err = run(&args, None).expect_err("missing catalog must error");
    assert!(err.to_string().contains("catalog not found"), "{err}");
}

// ── cockpit path ─────────────────────────────────────────────────────────────

#[test]
fn path_subcommand_prints_default_log_path() {
    run(
        &CockpitArgs {
            sub: CockpitSubcommand::Path,
        },
        None,
    )
    .expect("path should succeed");
}

#[test]
fn path_with_global_repo_is_rejected() {
    let err = run(
        &CockpitArgs {
            sub: CockpitSubcommand::Path,
        },
        Some(Path::new("/tmp")),
    )
    .expect_err("path + global repo must error");
    assert!(err.to_string().contains("--filter-repo"), "{err}");
}

// ── cockpit read dispatch guards ─────────────────────────────────────────────

#[test]
fn read_with_global_repo_is_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("cockpit.ndjson");
    let args = read_args(Some(log), None);
    let err = run(&args, Some(Path::new("/tmp"))).expect_err("read + global repo");
    assert!(err.to_string().contains("--filter-repo"), "{err}");
}

#[test]
fn read_with_missing_log_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("absent.ndjson");
    let args = read_args(Some(log), None);
    run(&args, None).expect("missing log is a cold start, not an error");
}

#[test]
fn read_with_populated_log_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("cockpit.ndjson");
    write_log(
        &log,
        concat!(
            "{\"ts\":\"epoch:1\",\"repo\":\"AgilePlus\",\"cluster\":\"C00\",\"score\":3,\"max\":3,\"grade\":\"A\",\"probes\":0}\n",
            "not json\n",
            "{\"ts\":\"epoch:2\",\"repo\":\"Tracera\",\"cluster\":\"C00\",\"score\":1,\"max\":3,\"grade\":\"D\",\"probes\":0}\n",
        ),
    );
    let args = read_args(Some(log.clone()), Some("AgilePlus"));
    run(&args, None).expect("filtered read should succeed");
}

// ── cockpit_read::run directly ───────────────────────────────────────────────

#[test]
fn cockpit_read_run_missing_input_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    let read = CockpitReadArgs {
        filter_repo: None,
        input: Some(tmp.path().join("nope.ndjson")),
        watch: false,
    };
    read_run(&read).expect("missing input is not an error");
}

#[test]
fn cockpit_read_run_populated_input_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("cockpit.ndjson");
    write_log(
        &log,
        "{\"ts\":\"epoch:1\",\"repo\":\"AgilePlus\",\"cluster\":\"C00\",\"score\":2,\"max\":3,\"grade\":\"C\",\"probes\":1}\n",
    );
    let read = CockpitReadArgs {
        filter_repo: Some("AgilePlus".to_string()),
        input: Some(log),
        watch: false,
    };
    read_run(&read).expect("populated read should succeed");
}

#[test]
fn cockpit_read_run_default_input_path_resolves() {
    // input = None -> default_log_path() reads $HOME/.agileplus/cockpit.ndjson.
    // This only reads (never writes), so it is safe to run.
    let read = CockpitReadArgs {
        filter_repo: None,
        input: None,
        watch: false,
    };
    read_run(&read).expect("default path read should succeed");
}
