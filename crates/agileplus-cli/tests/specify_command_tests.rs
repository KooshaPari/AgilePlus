// SPDX-License-Identifier: MIT OR Apache-2.0
//! Direct-call tests for `agileplus_cli::commands::specify::run_specify`.
//!
//! `run_specify` is generic over `StoragePort` + `VcsPort`. Here it is driven
//! with the real in-memory SQLite adapter and an in-memory `MockVcs`, covering
//! the create path, the refinement path (diff + revision counting), the
//! "no changes" short circuit, and the governance enforcement branch.

mod support;

use std::path::PathBuf;

use agileplus_cli::commands::specify::{SpecifyArgs, run_specify};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::StoragePort;
use agileplus_sqlite::SqliteStorageAdapter;

use support::MockVcs;

const VALID_SPEC: &str = "\
# Specification: Feature Alpha

## Problem Statement
Operators need a reproducible spec pipeline.

## Functional Requirements
- **FR-1**: create a feature from a file

## Acceptance Criteria
The feature is persisted in the Specified state.
";

const REVISED_SPEC: &str = "\
# Specification: Feature Alpha

## Problem Statement
Operators need a reproducible spec pipeline, revised.

## Functional Requirements
- **FR-1**: create a feature from a file
- **FR-2**: record a revision diff

## Acceptance Criteria
The feature is persisted in the Specified state.
";

const CONSTITUTION_PATH: &str = "../.kittify/memory/constitution.md";

fn write_spec(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, content).expect("write spec");
    p
}

fn args(feature: Option<&str>, from_file: PathBuf) -> SpecifyArgs {
    SpecifyArgs {
        feature: feature.map(|s| s.to_string()),
        from_file: Some(from_file),
        target_branch: "main".to_string(),
        force: false,
    }
}

// ── create path ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn creates_feature_from_file_and_writes_spec_artifact() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let vcs = MockVcs::new();
    let tmp = tempfile::tempdir().unwrap();
    let spec = write_spec(tmp.path(), "spec.md", VALID_SPEC);

    run_specify(args(Some("alpha"), spec), &storage, &vcs)
        .await
        .expect("create should succeed");

    let feature = storage
        .get_feature_by_slug("alpha")
        .await
        .unwrap()
        .expect("feature exists");
    assert!(matches!(feature.state, FeatureState::Specified));
    assert_eq!(vcs.get("alpha", "spec.md").as_deref(), Some(VALID_SPEC));

    // One audit entry was appended for Created -> Specified.
    let trail = storage.get_audit_trail(feature.id).await.unwrap();
    assert_eq!(trail.len(), 1);
    assert_eq!(trail[0].transition, "Created -> Specified");
}

#[tokio::test]
async fn derives_slug_from_first_heading_when_feature_omitted() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let vcs = MockVcs::new();
    let tmp = tempfile::tempdir().unwrap();
    let spec = write_spec(tmp.path(), "spec.md", VALID_SPEC);

    run_specify(args(None, spec), &storage, &vcs)
        .await
        .expect("create should succeed");

    // "# Specification: Feature Alpha" -> slug "feature-alpha".
    assert!(
        storage
            .get_feature_by_slug("feature-alpha")
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn falls_back_to_unnamed_feature_without_a_heading() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let vcs = MockVcs::new();
    let tmp = tempfile::tempdir().unwrap();
    let body = "No heading here.\n\n## Problem Statement\nx\n";
    let spec = write_spec(tmp.path(), "spec.md", body);

    run_specify(args(None, spec), &storage, &vcs)
        .await
        .expect("create should succeed");

    assert!(
        storage
            .get_feature_by_slug("unnamed-feature")
            .await
            .unwrap()
            .is_some()
    );
}

// ── refinement path ──────────────────────────────────────────────────────────

#[tokio::test]
async fn refines_existing_feature_and_records_revision_diff() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let vcs = MockVcs::new();
    let tmp = tempfile::tempdir().unwrap();

    let first = write_spec(tmp.path(), "spec.md", VALID_SPEC);
    run_specify(args(Some("alpha"), first), &storage, &vcs)
        .await
        .expect("initial create");

    let second = write_spec(tmp.path(), "spec2.md", REVISED_SPEC);
    run_specify(args(Some("alpha"), second), &storage, &vcs)
        .await
        .expect("refinement");

    assert_eq!(vcs.get("alpha", "spec.md").as_deref(), Some(REVISED_SPEC));
    let diff = vcs
        .get("alpha", "evidence/spec-revisions/rev-1.diff")
        .expect("rev-1 diff written");
    assert!(diff.contains("revised"), "diff should show the change: {diff}");

    let feature = storage
        .get_feature_by_slug("alpha")
        .await
        .unwrap()
        .unwrap();
    // create audit + refinement audit
    let trail = storage.get_audit_trail(feature.id).await.unwrap();
    assert_eq!(trail.len(), 2);
}

#[tokio::test]
async fn second_refinement_increments_revision_number() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let vcs = MockVcs::new();
    let tmp = tempfile::tempdir().unwrap();

    run_specify(
        args(Some("alpha"), write_spec(tmp.path(), "s1.md", VALID_SPEC)),
        &storage,
        &vcs,
    )
    .await
    .unwrap();
    run_specify(
        args(Some("alpha"), write_spec(tmp.path(), "s2.md", REVISED_SPEC)),
        &storage,
        &vcs,
    )
    .await
    .unwrap();

    let third = format!("{REVISED_SPEC}\n- **FR-3**: a third requirement\n");
    run_specify(
        args(Some("alpha"), write_spec(tmp.path(), "s3.md", &third)),
        &storage,
        &vcs,
    )
    .await
    .unwrap();

    assert!(vcs.get("alpha", "evidence/spec-revisions/rev-2.diff").is_some());
}

#[tokio::test]
async fn identical_spec_short_circuits_without_a_new_revision() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let vcs = MockVcs::new();
    let tmp = tempfile::tempdir().unwrap();

    run_specify(
        args(Some("alpha"), write_spec(tmp.path(), "s1.md", VALID_SPEC)),
        &storage,
        &vcs,
    )
    .await
    .unwrap();
    let after_create = vcs.artifact_count();

    // Same content again -> run_refinement detects no change.
    run_specify(
        args(Some("alpha"), write_spec(tmp.path(), "s2.md", VALID_SPEC)),
        &storage,
        &vcs,
    )
    .await
    .unwrap();

    assert_eq!(vcs.artifact_count(), after_create, "no new artifacts");
    assert!(vcs.get("alpha", "evidence/spec-revisions/rev-1.diff").is_none());
}

// ── governance ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn governance_blocks_spec_missing_required_sections() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let vcs = MockVcs::new().with_artifact("", CONSTITUTION_PATH, "constitution rules");
    let tmp = tempfile::tempdir().unwrap();
    let bad = write_spec(tmp.path(), "bad.md", "# Just a title\nnothing else\n");

    let err = run_specify(args(Some("alpha"), bad), &storage, &vcs)
        .await
        .expect_err("governance must reject the spec");
    assert!(
        err.to_string().contains("Governance checks failed"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn governance_passes_and_feature_is_created_with_constitution_present() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let vcs = MockVcs::new().with_artifact("", CONSTITUTION_PATH, "constitution rules");
    let tmp = tempfile::tempdir().unwrap();
    let spec = write_spec(tmp.path(), "spec.md", VALID_SPEC);

    run_specify(args(Some("alpha"), spec), &storage, &vcs)
        .await
        .expect("valid spec passes governance");
    assert!(
        storage
            .get_feature_by_slug("alpha")
            .await
            .unwrap()
            .is_some()
    );
}

// ── error paths ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn missing_spec_file_is_an_error() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let vcs = MockVcs::new();
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("does-not-exist.md");

    assert!(run_specify(args(Some("alpha"), missing), &storage, &vcs).await.is_err());
}
