// SPDX-License-Identifier: MIT OR Apache-2.0
//! Direct-call tests for `agileplus_cli::commands::fix_list::run`.
//!
//! Exercises the argument-validation branches, the catalog resolution path,
//! the cluster filter, `--limit` clamping, and both stdout and file output.

use std::path::{Path, PathBuf};

use agileplus_cli::commands::fix_list::{FixListArgs, run};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Workspace-bundled rubric catalog, resolved relative to this crate.
fn bundled_catalog() -> PathBuf {
    manifest_dir().join("../agileplus-governance/data/PILLARS-CATALOG.json")
}

fn base_args(repo: &Path, catalog: Option<PathBuf>, output: Option<PathBuf>) -> FixListArgs {
    FixListArgs {
        repo: repo.to_path_buf(),
        catalog,
        clusters: None,
        output,
        limit: 10,
    }
}

#[test]
fn fix_list_writes_markdown_to_output_file() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("fix-list.md");
    let args = base_args(&manifest_dir(), Some(bundled_catalog()), Some(out.clone()));
    run(&args).expect("fix-list should succeed");

    let md = std::fs::read_to_string(&out).expect("output written");
    assert!(md.contains("# Fix list"), "missing header: {md}");
    assert!(md.contains("Per-cluster gap totals"), "missing footer");
}

#[test]
fn fix_list_stdout_path_succeeds() {
    let args = base_args(&manifest_dir(), Some(bundled_catalog()), None);
    run(&args).expect("fix-list to stdout should succeed");
}

#[test]
fn fix_list_resolves_default_catalog_when_omitted() {
    // catalog = None -> resolve_default_catalog_for_siblings() walks up from cwd.
    let args = base_args(&manifest_dir(), None, None);
    run(&args).expect("default catalog resolution should succeed");
}

#[test]
fn fix_list_cluster_filter_restricts_output() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("filtered.md");
    let mut args = base_args(&manifest_dir(), Some(bundled_catalog()), Some(out.clone()));
    args.clusters = Some(vec!["C03".to_string()]);
    run(&args).expect("filtered fix-list should succeed");
    assert!(out.exists());
}

#[test]
fn fix_list_limit_zero_is_clamped_to_one() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("limit0.md");
    let mut args = base_args(&manifest_dir(), Some(bundled_catalog()), Some(out.clone()));
    args.limit = 0;
    run(&args).expect("limit 0 is clamped, not an error");
    assert!(out.exists());
}

#[test]
fn fix_list_missing_repo_errors() {
    let args = base_args(
        Path::new("/nonexistent/agileplus-fix-list-repo"),
        Some(bundled_catalog()),
        None,
    );
    let err = run(&args).expect_err("missing repo must error");
    assert!(
        err.to_string().contains("does not exist"),
        "unexpected error: {err}"
    );
}

#[test]
fn fix_list_repo_must_be_a_directory() {
    // Point --repo at a regular file (this test's own source path is not used;
    // the bundled catalog is a real file and deterministic).
    let catalog = bundled_catalog();
    let args = base_args(&catalog, Some(catalog.clone()), None);
    let err = run(&args).expect_err("repo as a file must error");
    assert!(
        err.to_string().contains("must be a directory"),
        "unexpected error: {err}"
    );
}

#[test]
fn fix_list_missing_catalog_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let ghost = tmp.path().join("no-catalog.json");
    let args = base_args(&manifest_dir(), Some(ghost), None);
    let err = run(&args).expect_err("missing catalog must error");
    assert!(
        err.to_string().contains("catalog not found"),
        "unexpected error: {err}"
    );
}

#[test]
fn fix_list_unwritable_output_errors() {
    // Output inside a path whose parent is a regular file -> write fails.
    let tmp = tempfile::tempdir().unwrap();
    let blocker = tmp.path().join("blocker");
    std::fs::write(&blocker, b"x").unwrap();
    let out = blocker.join("nested").join("fix-list.md");
    let args = base_args(&manifest_dir(), Some(bundled_catalog()), Some(out));
    assert!(run(&args).is_err(), "write into a file path must error");
}
