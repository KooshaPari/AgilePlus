// SPDX-License-Identifier: MIT OR Apache-2.0
//! End-to-end tests for the `agileplus implement` command.
//!
//! These drive the real `run_implement` orchestration with a real SQLite
//! storage adapter, a VCS double that performs genuine filesystem writes for
//! artifact materialization, and a scripted agent. The point is to prove the
//! command reaches `run_review_loop` and that the loop's outcome produces the
//! documented work-package state transitions, not just to retest the loop
//! in isolation (which `review_loop_flow` covers).

mod support;

use agileplus_cli::commands::implement::run_implement;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WpState;
use agileplus_domain::ports::StoragePort;
use agileplus_sqlite::SqliteStorageAdapter;
use support::implement::{
    args, block_on, block_on_paused, seed, AgentBehavior, ScriptedAgent, TempVcs, PLAN, SPEC,
};

/// A feature in an invalid state is rejected before any agent work happens.
#[test]
fn implement_rejects_feature_in_wrong_state() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().expect("in-memory db");
        let (_, wp_id) = seed(&storage, "wrong-state", FeatureState::Shipped).await;
        let vcs = TempVcs::new();
        let agent = ScriptedAgent::success();

        let err = run_implement(args("wrong-state"), &storage, &vcs, &agent)
            .await
            .expect_err("Shipped feature must not be implementable");

        assert!(
            err.to_string()
                .contains("Expected 'Planned' or 'Implementing'"),
            "unexpected error: {err}"
        );
        assert_eq!(agent.polls(), 0, "agent must never be polled");
        assert!(
            vcs.created_worktrees().is_empty(),
            "no worktree may be created for a rejected feature"
        );
        // The WP is untouched.
        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wp.state, WpState::Planned);
        vcs.cleanup();
    });
}

/// A missing feature produces the documented planning hint, not a panic.
#[test]
fn implement_reports_unknown_feature_with_guidance() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().expect("in-memory db");
        let vcs = TempVcs::new();
        let agent = ScriptedAgent::success();

        let err = run_implement(args("nope"), &storage, &vcs, &agent)
            .await
            .expect_err("unknown feature must error");

        let msg = format!("{err:#}");
        assert!(msg.contains("Feature 'nope' not found"), "got: {msg}");
        assert!(
            msg.contains("agileplus plan"),
            "expected guidance, got: {msg}"
        );
        vcs.cleanup();
    });
}

/// A feature with no work packages is rejected before dispatch.
#[test]
fn implement_reports_feature_without_work_packages() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().expect("in-memory db");
        let fid = StoragePort::create_feature(
            &storage,
            &support::implement::feature("empty", FeatureState::Planned),
        )
        .await
        .unwrap();
        assert!(fid > 0);
        let vcs = TempVcs::new();
        let agent = ScriptedAgent::success();

        let err = run_implement(args("empty"), &storage, &vcs, &agent)
            .await
            .expect_err("feature with no WPs must error");

        let msg = format!("{err:#}");
        assert!(
            msg.contains("No work packages found"),
            "unexpected error: {msg}"
        );
        assert_eq!(agent.polls(), 0);
        vcs.cleanup();
    });
}

/// The happy path: a Planned feature is transitioned to Implementing, the WP
/// advances through Doing -> Review -> Done, the worktree is cleaned up, and
/// the feature audit chain records the transition.
#[test]
fn implement_approved_moves_wp_to_done_and_cleans_worktree() {
    block_on_paused(async {
        let storage = SqliteStorageAdapter::in_memory().expect("in-memory db");
        let (fid, wp_id) = seed(&storage, "happy", FeatureState::Planned).await;
        let vcs = TempVcs::new();
        let agent = ScriptedAgent::success();

        run_implement(args("happy"), &storage, &vcs, &agent)
            .await
            .expect("implement should succeed");

        // The feature is now Implementing.
        let feature = StoragePort::get_feature_by_slug(&storage, "happy")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(feature.state, FeatureState::Implementing);

        // The WP reached Done via the approved review outcome.
        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wp.state, WpState::Done, "approved WP must be Done");

        // Exactly one worktree was created and then cleaned up.
        let created = vcs.created_worktrees();
        assert_eq!(created.len(), 1, "one worktree per WP");
        let cleaned = vcs.cleaned_worktrees();
        assert_eq!(cleaned, created, "the created worktree must be cleaned");

        // The review loop actually ran through the real orchestration.
        assert_eq!(agent.polls(), 1, "approved after a single poll");

        // The agent was given a real prompt path and the three context files.
        let task = agent
            .dispatched
            .lock()
            .unwrap()
            .clone()
            .expect("agent dispatched");
        assert!(task.prompt_path.ends_with("tasks/WP01-build-the-thing.md"));
        assert_eq!(task.context_files.len(), 3);
        for f in &task.context_files {
            assert!(
                f.ends_with("spec.md") || f.ends_with("plan.md") || f.ends_with("research.md"),
                "unexpected context file: {f:?}"
            );
        }

        // The plan artifacts were materialized as real files in the worktree,
        // under the `kitty-specs/<slug>/` layout the prompt path is built from.
        let wt = &created[0];
        let kitty = wt.join("kitty-specs/happy");
        assert_eq!(
            std::fs::read_to_string(kitty.join("spec.md")).unwrap(),
            SPEC
        );
        assert_eq!(
            std::fs::read_to_string(kitty.join("plan.md")).unwrap(),
            PLAN
        );
        let prompt = kitty.join("tasks/WP01-build-the-thing.md");
        assert!(prompt.exists(), "prompt artifact must be materialized");
        // The dispatched task points at the same on-disk prompt.
        assert_eq!(
            task.prompt_path, prompt,
            "agent must be pointed at the materialized prompt"
        );

        // The audit chain recorded the feature transition and the WP completion.
        let entries = StoragePort::get_audit_trail(&storage, fid).await.unwrap();
        let transitions: Vec<&str> = entries.iter().map(|e| e.transition.as_str()).collect();
        assert!(
            transitions
                .iter()
                .any(|t| t.contains("Planned -> Implementing")),
            "expected the feature transition in the audit chain, got {transitions:?}"
        );
        assert!(
            transitions.iter().any(|t| t.contains("Planned -> Done")),
            "expected the WP completion in the audit chain, got {transitions:?}"
        );

        vcs.cleanup();
    });
}

/// A WP whose agent never succeeds exhausts the review cycles and is marked
/// Blocked, with the last feedback surfaced and fed back to the agent.
#[test]
fn implement_max_cycles_blocks_wp_with_last_feedback() {
    block_on_paused(async {
        let storage = SqliteStorageAdapter::in_memory().expect("in-memory db");
        let (_, wp_id) = seed(&storage, "loopy", FeatureState::Planned).await;
        let vcs = TempVcs::new();
        let agent = ScriptedAgent::new(AgentBehavior::AlwaysFail(
            "clippy: unused import".to_string(),
        ));

        // Two configured cycles means two polls and one re-instruction.
        run_implement(args("loopy"), &storage, &vcs, &agent)
            .await
            .expect("max cycles is a non-fatal outcome");

        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wp.state, WpState::Blocked, "exhausted WP must be Blocked");

        assert_eq!(agent.polls(), 2, "one poll per configured cycle");
        let instructions = agent.instructions.lock().unwrap().clone();
        assert_eq!(
            instructions.len(),
            1,
            "cycles - 1 re-instructions, since the last cycle is not fed back"
        );
        assert!(
            instructions[0].contains("clippy: unused import"),
            "stderr must be fed back verbatim, got: {instructions:?}"
        );

        // A blocked WP is not cleaned up, so the worktree is left for inspection.
        assert!(
            vcs.cleaned_worktrees().is_empty(),
            "blocked WP must keep its worktree"
        );

        vcs.cleanup();
    });
}

/// A hard agent failure is fatal to the command and blocks the WP.
#[test]
fn implement_hard_agent_failure_blocks_wp_and_errors() {
    block_on_paused(async {
        let storage = SqliteStorageAdapter::in_memory().expect("in-memory db");
        let (_, wp_id) = seed(&storage, "crash", FeatureState::Planned).await;
        let vcs = TempVcs::new();
        let agent = ScriptedAgent::new(AgentBehavior::HardFail("agent segfault".to_string()));

        let err = run_implement(args("crash"), &storage, &vcs, &agent)
            .await
            .expect_err("a hard agent failure must fail the command");

        let msg = format!("{err:#}");
        assert!(
            msg.contains("Agent failed for WP01"),
            "unexpected error: {msg}"
        );
        assert!(
            msg.contains("agent segfault"),
            "error must propagate: {msg}"
        );

        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wp.state, WpState::Blocked);

        vcs.cleanup();
    });
}

/// Targeting a single WP by its `WP01` selector runs only that work package.
#[test]
fn implement_single_wp_selector_runs_only_that_wp() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().expect("in-memory db");
        let fid = StoragePort::create_feature(
            &storage,
            &support::implement::feature("picker", FeatureState::Planned),
        )
        .await
        .unwrap();
        // Two independent WPs.
        for seq in 1..=2 {
            let mut wp = agileplus_domain::domain::work_package::WorkPackage::new(
                fid,
                &format!("Task {seq}"),
                seq,
                "done",
            );
            wp.id = 0;
            StoragePort::create_work_package(&storage, &wp)
                .await
                .unwrap();
        }

        let vcs = TempVcs::new();
        let agent = ScriptedAgent::success();
        let mut a = args("picker");
        a.wp = Some("WP02".to_string());

        run_implement(a, &storage, &vcs, &agent)
            .await
            .expect("should succeed");

        let wps = StoragePort::list_wps_by_feature(&storage, fid)
            .await
            .unwrap();
        assert_eq!(wps.len(), 2);
        let by_seq = |seq: i32| {
            wps.iter()
                .find(|w| w.sequence == seq)
                .unwrap()
                .state
                .clone()
        };
        assert_eq!(by_seq(1), WpState::Planned, "WP01 must be untouched");
        assert_eq!(by_seq(2), WpState::Done, "WP02 must be done");

        // Only the selected WP was dispatched and given a worktree.
        assert_eq!(vcs.created_worktrees().len(), 1);
        let task = agent.dispatched.lock().unwrap().clone().unwrap();
        assert!(
            task.prompt_path.ends_with("tasks/WP02-task-2.md"),
            "{task:?}"
        );

        vcs.cleanup();
    });
}

/// An unknown WP selector is rejected before any agent dispatch.
#[test]
fn implement_unknown_wp_selector_errors() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().expect("in-memory db");
        seed(&storage, "missing-wp", FeatureState::Planned).await;
        let vcs = TempVcs::new();
        let agent = ScriptedAgent::success();
        let mut a = args("missing-wp");
        a.wp = Some("WP99".to_string());

        let err = run_implement(a, &storage, &vcs, &agent)
            .await
            .expect_err("unknown WP must error");

        let msg = format!("{err:#}");
        assert!(msg.contains("Work package 'WP99' not found"), "got: {msg}");
        assert_eq!(agent.polls(), 0);
        vcs.cleanup();
    });
}

/// A missing prompt artifact aborts before the agent is dispatched, and the
/// error names the artifact that could not be read.
#[test]
fn implement_missing_prompt_artifact_aborts_before_dispatch() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().expect("in-memory db");
        seed(&storage, "no-prompt", FeatureState::Planned).await;
        let vcs = TempVcs::new().with_missing_prompt();
        let agent = ScriptedAgent::success();

        let err = run_implement(args("no-prompt"), &storage, &vcs, &agent)
            .await
            .expect_err("a missing prompt artifact must abort");

        let msg = format!("{err:#}");
        assert!(
            msg.contains("reading artifact tasks/WP01"),
            "error should name the artifact, got: {msg}"
        );
        assert_eq!(agent.polls(), 0, "agent must not be polled");
        assert!(
            agent.dispatched.lock().unwrap().is_none(),
            "agent must not be dispatched"
        );

        vcs.cleanup();
    });
}
