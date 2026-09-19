// SPDX-License-Identifier: MIT OR Apache-2.0
//! Direct-call tests for `agileplus_cli::commands::rubric::run`.
//!
//! Covers the `Score` dispatch (default-catalog resolution, probe modes,
//! output-file and stdout routing, the summary footer) and the `FixList`
//! delegation, plus every argument-validation error branch.

use std::path::{Path, PathBuf};

use agileplus_cli::commands::fix_list::FixListArgs;
use agileplus_cli::commands::rubric::{ProbeMode, RubricArgs, RubricSubcommand, run};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn bundled_catalog() -> PathBuf {
    manifest_dir().join("../agileplus-governance/data/PILLARS-CATALOG.json")
}

fn score_args(repo: &Path, catalog: Option<PathBuf>, output: Option<PathBuf>, probes: ProbeMode) -> RubricArgs {
    RubricArgs {
        sub: RubricSubcommand::Score {
            repo: repo.to_path_buf(),
            catalog,
            clusters: None,
            output,
            probes,
        },
    }
}

// ── Score success paths ──────────────────────────────────────────────────────

#[test]
fn score_with_none_probes_succeeds() {
    let args = score_args(&manifest_dir(), Some(bundled_catalog()), None, ProbeMode::None);
    run(&args).expect("v1 probe-free scoring should succeed");
}

#[test]
fn score_with_auto_probes_succeeds() {
    let args = score_args(&manifest_dir(), Some(bundled_catalog()), None, ProbeMode::Auto);
    run(&args).expect("auto probe scoring should succeed");
}

#[test]
fn score_with_all_probes_succeeds() {
    let args = score_args(&manifest_dir(), Some(bundled_catalog()), None, ProbeMode::All);
    run(&args).expect("all probe scoring should succeed");
}

#[test]
fn score_default_catalog_is_resolved_when_omitted() {
    let args = score_args(&manifest_dir(), None, None, ProbeMode::None);
    run(&args).expect("default catalog resolution should succeed");
}

#[test]
fn score_writes_scorecard_to_output_file() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("scorecard.md");
    let args = score_args(
        &manifest_dir(),
        Some(bundled_catalog()),
        Some(out.clone()),
        ProbeMode::None,
    );
    run(&args).expect("scorecard should be written");
    let md = std::fs::read_to_string(&out).expect("scorecard file exists");
    assert!(md.contains("CLUSTER_START"), "missing cluster markers");
    assert!(md.contains("CLUSTER_DONE"), "missing cluster markers");
}

#[test]
fn score_cluster_filter_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("filtered.md");
    let mut args = score_args(
        &manifest_dir(),
        Some(bundled_catalog()),
        Some(out.clone()),
        ProbeMode::None,
    );
    if let RubricSubcommand::Score { clusters, .. } = &mut args.sub {
        *clusters = Some(vec!["C03".to_string(), "C04".to_string()]);
    }
    run(&args).expect("cluster-filtered score should succeed");
    assert!(out.exists());
}

// ── Score error paths ────────────────────────────────────────────────────────

#[test]
fn score_missing_repo_errors() {
    let args = score_args(
        Path::new("/nonexistent/agileplus-rubric-repo"),
        Some(bundled_catalog()),
        None,
        ProbeMode::None,
    );
    let err = run(&args).expect_err("missing repo must error");
    assert!(err.to_string().contains("does not exist"), "{err}");
}

#[test]
fn score_repo_must_be_directory() {
    let catalog = bundled_catalog();
    let args = score_args(&catalog, Some(catalog.clone()), None, ProbeMode::None);
    let err = run(&args).expect_err("repo as file must error");
    assert!(err.to_string().contains("must be a directory"), "{err}");
}

#[test]
fn score_missing_catalog_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let ghost = tmp.path().join("absent.json");
    let args = score_args(&manifest_dir(), Some(ghost), None, ProbeMode::None);
    let err = run(&args).expect_err("missing catalog must error");
    assert!(err.to_string().contains("catalog not found"), "{err}");
}

#[test]
fn score_unwritable_output_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let blocker = tmp.path().join("blocker");
    std::fs::write(&blocker, b"x").unwrap();
    let out = blocker.join("nested").join("scorecard.md");
    let args = score_args(
        &manifest_dir(),
        Some(bundled_catalog()),
        Some(out),
        ProbeMode::None,
    );
    assert!(run(&args).is_err());
}

// ── FixList delegation ───────────────────────────────────────────────────────

#[test]
fn fix_list_subcommand_delegates_and_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("fix-list.md");
    let args = RubricArgs {
        sub: RubricSubcommand::FixList(FixListArgs {
            repo: manifest_dir(),
            catalog: Some(bundled_catalog()),
            clusters: None,
            output: Some(out.clone()),
            limit: 5,
        }),
    };
    run(&args).expect("fix-list delegation should succeed");
    assert!(out.exists());
}
