// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for `agileplus ship` side effects and persistence
//! (commands/ship.rs).
//!
//! Covers worktree cleanup filtering, the non-fatal handling of cleanup,
//! worktree-listing, and artifact-write failures, the `meta.json` contents,
//! the Validated -> Shipped state transition, the audit hash chain, and the
//! chained state-transition event.

use agileplus_cli::commands::ship::run_ship;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WpState;
use agileplus_domain::ports::StoragePort;
use agileplus_events::EventStore;
use agileplus_sqlite::SqliteStorageAdapter;

mod support;
use support::ship::{RecordingVcs, args, block_on, seed};

#[test]
fn ship_cleans_up_only_matching_worktrees() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new().with_worktrees(vec![
            RecordingVcs::worktree("/tmp/wt/cleanup-me-WP01", "cleanup-feat"),
            RecordingVcs::worktree("/tmp/wt/other-WP01", "someone-else"),
        ]);
        seed(
            &storage,
            "cleanup-feat",
            FeatureState::Validated,
            &[(1, WpState::Done)],
        )
        .await;
        run_ship(args("cleanup-feat"), &storage, &vcs)
            .await
            .expect("ship");
        let cleaned = vcs.cleaned.lock().unwrap().clone();
        assert_eq!(cleaned.len(), 1, "only this feature's worktree");
        assert!(cleaned[0].to_string_lossy().contains("cleanup-me-WP01"));
    })
}

#[test]
fn ship_continues_when_worktree_cleanup_fails() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new()
            .with_cleanup_failure()
            .with_worktrees(vec![RecordingVcs::worktree("/tmp/wt/stuck", "stuck-feat")]);
        let id = seed(
            &storage,
            "stuck-feat",
            FeatureState::Validated,
            &[(1, WpState::Done)],
        )
        .await;
        run_ship(args("stuck-feat"), &storage, &vcs)
            .await
            .expect("cleanup failure is non-fatal");
        let f = StoragePort::get_feature_by_id(&storage, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.state, FeatureState::Shipped);
        assert!(vcs.cleaned.lock().unwrap().is_empty());
    })
}

#[test]
fn ship_continues_when_worktree_listing_fails() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new().with_worktree_listing_failure();
        let id = seed(
            &storage,
            "listfail-feat",
            FeatureState::Validated,
            &[(1, WpState::Done)],
        )
        .await;
        run_ship(args("listfail-feat"), &storage, &vcs)
            .await
            .expect("worktree listing failure is non-fatal");
        let f = StoragePort::get_feature_by_id(&storage, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.state, FeatureState::Shipped);
    })
}

#[test]
fn ship_continues_when_artifact_write_fails() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new().with_artifact_write_failure();
        let id = seed(
            &storage,
            "artfail-feat",
            FeatureState::Validated,
            &[(1, WpState::Done)],
        )
        .await;
        run_ship(args("artfail-feat"), &storage, &vcs)
            .await
            .expect("artifact write failure is non-fatal");
        assert!(vcs.artifacts.lock().unwrap().is_empty());
        let f = StoragePort::get_feature_by_id(&storage, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.state, FeatureState::Shipped);
    })
}

#[test]
fn ship_meta_artifact_records_slug_target_and_merged_branches() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        seed(
            &storage,
            "meta-feat",
            FeatureState::Validated,
            &[(1, WpState::Done), (2, WpState::Done)],
        )
        .await;
        let mut a = args("meta-feat");
        a.target = Some("main".to_string());
        run_ship(a, &storage, &vcs).await.expect("ship");
        let artifacts = vcs.artifacts.lock().unwrap().clone();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].0, "meta-feat", "artifact keyed by slug");
        assert_eq!(artifacts[0].1, "meta.json");
        let meta: serde_json::Value = serde_json::from_str(&artifacts[0].2).unwrap();
        assert_eq!(meta["feature_slug"], "meta-feat");
        assert_eq!(meta["state"], "shipped");
        assert_eq!(meta["target_branch"], "main");
        assert_eq!(meta["wp_count"], 2);
        assert_eq!(meta["merged_branches"].as_array().unwrap().len(), 2);
        assert!(
            meta["shipped_at"].as_str().unwrap().contains('T'),
            "shipped_at must be an RFC3339 timestamp"
        );
    })
}

#[test]
fn ship_transitions_state_and_writes_audit_chain() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        let id = seed(
            &storage,
            "audit-feat",
            FeatureState::Validated,
            &[(1, WpState::Done)],
        )
        .await;
        run_ship(args("audit-feat"), &storage, &vcs)
            .await
            .expect("ship");
        let f = StoragePort::get_feature_by_id(&storage, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.state, FeatureState::Shipped);

        let trail = StoragePort::get_audit_trail(&storage, id).await.unwrap();
        assert_eq!(trail.len(), 1, "exactly one audit entry for the ship");
        let entry = &trail[0];
        assert_eq!(entry.transition, "Validated -> Shipped");
        assert_eq!(entry.actor, "user");
        assert!(entry.hash != [0u8; 32], "audit hash must be computed");
        assert_eq!(entry.prev_hash, [0u8; 32], "no prior entry, so zero prev");
    })
}

#[test]
fn ship_audit_entry_chains_to_prior_entry() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        let id = seed(
            &storage,
            "chain-feat",
            FeatureState::Validated,
            &[(1, WpState::Done)],
        )
        .await;
        // Prepend a prior audit entry so the ship entry must chain to it.
        let mut prior = agileplus_domain::domain::audit::AuditEntry {
            id: 0,
            feature_id: id,
            wp_id: None,
            timestamp: chrono::Utc::now(),
            actor: "earlier".into(),
            transition: "Planned -> Implementing".into(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        prior.hash = agileplus_domain::domain::audit::hash_entry(&prior);
        let prior_hash = prior.hash;
        StoragePort::append_audit_entry(&storage, &prior)
            .await
            .unwrap();

        run_ship(args("chain-feat"), &storage, &vcs)
            .await
            .expect("ship");
        let trail = StoragePort::get_audit_trail(&storage, id).await.unwrap();
        assert_eq!(trail.len(), 2);
        let ship_entry = trail.last().unwrap();
        assert_eq!(ship_entry.transition, "Validated -> Shipped");
        assert_eq!(
            ship_entry.prev_hash, prior_hash,
            "ship entry must chain to the prior entry's hash"
        );
    })
}

#[test]
fn ship_appends_state_transition_event_with_chained_hash() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let vcs = RecordingVcs::new();
        let id = seed(
            &storage,
            "event-feat",
            FeatureState::Validated,
            &[(1, WpState::Done)],
        )
        .await;
        // Seed a prior event so the transition event chains and sequences.
        let mut prior = agileplus_domain::domain::event::Event::new(
            "feature",
            id,
            "created",
            serde_json::json!({"slug": "event-feat"}),
            "planner",
        );
        // `append_event` persists the sequence it is handed, so seed the stream
        // at 1 and assert the ship event lands on 2.
        prior.sequence = 1;
        prior.hash = agileplus_events::compute_hash(
            prior.entity_id,
            &prior.entity_type,
            &prior.event_type,
            &prior.payload,
            prior.timestamp,
            &prior.actor,
            &prior.prev_hash,
        )
        .unwrap();
        let prior_hash = prior.hash;
        storage.append(&prior).await.unwrap();

        run_ship(args("event-feat"), &storage, &vcs)
            .await
            .expect("ship");
        let events = storage.get_events("feature", id).await.unwrap();
        assert_eq!(events.len(), 2, "prior + transition event");
        let transition = events.last().unwrap();
        assert_eq!(transition.event_type, "state_transitioned");
        assert_eq!(transition.sequence, 2, "sequence continues the stream");
        assert_eq!(transition.prev_hash, prior_hash, "event chain linked");
        assert!(transition.hash != [0u8; 32], "event hash computed");
        assert_eq!(transition.payload["from"], "Validated");
        assert_eq!(transition.payload["to"], "Shipped");
    })
}
