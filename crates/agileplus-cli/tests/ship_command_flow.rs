// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for `agileplus ship` gating and merge behavior
//! (commands/ship.rs).
//!
//! Covers feature lookup, state enforcement (including `--skip-validate`),
//! the incomplete-work-package gate, dry-run short-circuiting, branch-name
//! derivation, merge ordering, target selection, conflict reporting, and the
//! non-fatal handling of merge errors.

use agileplus_cli::commands::ship::run_ship;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WpState;
use agileplus_domain::ports::StoragePort;
use agileplus_sqlite::SqliteStorageAdapter;

mod support;
use support::ship::{MergeOutcome, RecordingVcs, args, block_on, seed, seed_with_worktree_wp};

#[test]
fn ship_errors_for_unknown_feature() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        let err = run_ship(args("no-such-feature"), &storage, &vcs)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("not found"), "got: {err}");
        assert!(vcs.merges.lock().unwrap().is_empty(), "no merges attempted");
    })
}

#[test]
fn ship_rejects_non_validated_state_without_skip() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        seed(&storage, "planned-feat", FeatureState::Planned, &[]).await;
        let err = run_ship(args("planned-feat"), &storage, &vcs)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("Expected 'Validated'"),
            "got: {err}"
        );
        let f = StoragePort::get_feature_by_slug(&storage, "planned-feat")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.state, FeatureState::Planned, "state must not change");
    })
}

#[test]
fn ship_skip_validate_overrides_state_check() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        // No WPs, so the incomplete gate is satisfied.
        let id = seed(&storage, "forced-feat", FeatureState::Implementing, &[]).await;
        let mut a = args("forced-feat");
        a.skip_validate = true;
        run_ship(a, &storage, &vcs)
            .await
            .expect("skip-validate override");
        let f = StoragePort::get_feature_by_id(&storage, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.state, FeatureState::Shipped);
    })
}

#[test]
fn ship_rejects_incomplete_work_packages() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        seed(
            &storage,
            "partial-feat",
            FeatureState::Validated,
            &[(1, WpState::Done), (2, WpState::Doing)],
        )
        .await;
        let err = run_ship(args("partial-feat"), &storage, &vcs)
            .await
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("incomplete work packages"), "got: {msg}");
        assert!(
            msg.contains("WP02 'WP 2'"),
            "must name the blocked WP: {msg}"
        );
        assert!(msg.contains("Doing"), "must name the state: {msg}");
        assert!(vcs.merges.lock().unwrap().is_empty(), "no merges attempted");
    })
}

#[test]
fn ship_dry_run_makes_no_changes() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        let id = seed(
            &storage,
            "dry-feat",
            FeatureState::Validated,
            &[(1, WpState::Done), (2, WpState::Done)],
        )
        .await;
        let mut a = args("dry-feat");
        a.dry_run = true;
        run_ship(a, &storage, &vcs).await.expect("dry run succeeds");
        assert!(
            vcs.merges.lock().unwrap().is_empty(),
            "dry run merges nothing"
        );
        assert!(
            vcs.artifacts.lock().unwrap().is_empty(),
            "no artifact written"
        );
        let f = StoragePort::get_feature_by_id(&storage, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            f.state,
            FeatureState::Validated,
            "dry run must not transition"
        );
    })
}

#[test]
fn ship_dry_run_with_no_wps_succeeds() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        seed(&storage, "empty-dry", FeatureState::Validated, &[]).await;
        let mut a = args("empty-dry");
        a.dry_run = true;
        run_ship(a, &storage, &vcs)
            .await
            .expect("dry run with zero WPs succeeds");
    })
}

#[test]
fn ship_derives_branches_and_merges_in_sequence_order() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        seed(
            &storage,
            "merge-feat",
            FeatureState::Validated,
            &[(2, WpState::Done), (1, WpState::Done), (10, WpState::Done)],
        )
        .await;
        run_ship(args("merge-feat"), &storage, &vcs)
            .await
            .expect("all WPs done");
        let merges = vcs.merges.lock().unwrap().clone();
        let sources: Vec<&str> = merges.iter().map(|(s, _)| s.as_str()).collect();
        assert_eq!(
            sources,
            vec![
                "feature/merge-feat/wp01",
                "feature/merge-feat/wp02",
                "feature/merge-feat/wp10"
            ],
            "WP10 must be zero-padded, and order must follow sequence"
        );
        assert!(
            merges.iter().all(|(_, t)| t == "main"),
            "all merges target the feature's target_branch"
        );
    })
}

#[test]
fn ship_honors_target_branch_override() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        seed(
            &storage,
            "target-feat",
            FeatureState::Validated,
            &[(1, WpState::Done)],
        )
        .await;
        let mut a = args("target-feat");
        a.target = Some("release/1.x".to_string());
        run_ship(a, &storage, &vcs)
            .await
            .expect("ship with override");
        let merges = vcs.merges.lock().unwrap().clone();
        assert_eq!(merges.len(), 1);
        assert_eq!(merges[0].1, "release/1.x", "override must win");
    })
}

#[test]
fn ship_uses_worktree_path_as_branch_when_present() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        seed_with_worktree_wp(
            &storage,
            "wt-feat",
            FeatureState::Validated,
            1,
            ".worktrees/wt-feat-WP01",
        )
        .await;
        run_ship(args("wt-feat"), &storage, &vcs)
            .await
            .expect("ship");
        let merges = vcs.merges.lock().unwrap().clone();
        assert_eq!(
            merges[0].0, "wt-feat-WP01",
            "worktree file_name wins over the convention"
        );
    })
}

#[test]
fn ship_reports_merge_conflicts_and_stops() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new().with_merge_outcome(MergeOutcome::Conflicts(vec![
            "src/a.rs".into(),
            "src/b.rs".into(),
        ]));
        let id = seed(
            &storage,
            "conflict-feat",
            FeatureState::Validated,
            &[(1, WpState::Done), (2, WpState::Done)],
        )
        .await;
        let err = run_ship(args("conflict-feat"), &storage, &vcs)
            .await
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Merge conflict"), "got: {msg}");
        assert!(msg.contains("src/a.rs") && msg.contains("src/b.rs"));
        assert_eq!(
            vcs.merges.lock().unwrap().len(),
            1,
            "must abort on the first conflicting merge"
        );
        let f = StoragePort::get_feature_by_id(&storage, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            f.state,
            FeatureState::Validated,
            "conflicted feature must not ship"
        );
    })
}

#[test]
fn ship_skips_branches_whose_merge_errors() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        // All merges error, so nothing is merged, but shipping still completes.
        let vcs = RecordingVcs::new().with_merge_outcome(MergeOutcome::Error);
        let id = seed(
            &storage,
            "err-feat",
            FeatureState::Validated,
            &[(1, WpState::Done), (2, WpState::Done)],
        )
        .await;
        run_ship(args("err-feat"), &storage, &vcs)
            .await
            .expect("merge errors are non-fatal, they are skipped");
        assert_eq!(vcs.merges.lock().unwrap().len(), 2, "both attempted");
        let f = StoragePort::get_feature_by_id(&storage, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.state, FeatureState::Shipped);
        let artifacts = vcs.artifacts.lock().unwrap().clone();
        let meta: serde_json::Value = serde_json::from_str(&artifacts[0].2).unwrap();
        assert_eq!(
            meta["merged_branches"].as_array().unwrap().len(),
            0,
            "no branches actually merged"
        );
    })
}
