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

use agileplus_cli::commands::implement::{run_implement, summary_counts};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WpState;
use agileplus_domain::domain::work_package::{DependencyType, WpDependency};
use agileplus_domain::ports::StoragePort;
use agileplus_sqlite::SqliteStorageAdapter;
use support::implement::{
    AgentBehavior, PLAN, SPEC, ScriptedAgent, TempVcs, args, block_on, block_on_paused, seed,
    seed_recorded_candidate,
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

/// A WP already in Review has a retained candidate worktree recorded as a
/// Completed Attempt on its active Assignment. A later normal `implement` run
/// must not create a second worktree or dispatch another candidate for it;
/// independent acceptance belongs to `validate`.
#[test]
fn implement_skips_review_candidate_without_creating_another_worktree() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().expect("in-memory db");
        let (fid, wp_id) = seed(&storage, "review-candidate", FeatureState::Implementing).await;
        StoragePort::update_wp_state(&storage, wp_id, WpState::Review)
            .await
            .unwrap();
        let candidate_ref = seed_recorded_candidate(&storage, fid, wp_id).await;
        let vcs = TempVcs::new().with_retained_candidate("review-candidate", "WP01");
        let agent = ScriptedAgent::success();

        run_implement(args("review-candidate"), &storage, &vcs, &agent)
            .await
            .expect("skipping a Review candidate must succeed");

        assert!(
            vcs.created_worktrees().is_empty(),
            "a Review candidate must not get a second worktree: {:?}",
            vcs.created_worktrees()
        );
        assert_eq!(
            agent.polls(),
            0,
            "a Review candidate must not get another agent dispatch"
        );
        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wp.state, WpState::Review);
        // The skip is only legitimate because the ledger holds the candidate,
        // and the missing Review audit entry is repaired before the WP counts.
        let trail = StoragePort::get_audit_trail(&storage, fid).await.unwrap();
        assert!(
            trail
                .iter()
                .any(|entry| entry.wp_id == Some(wp_id)
                    && entry.transition == "WP01: Doing -> Review"),
            "the Review transition must have an audit entry: {trail:?}"
        );
        assert!(
            candidate_ref.starts_with("git:"),
            "the recorded candidate must be an exact ref, got {candidate_ref}"
        );
        vcs.cleanup();
    });
}

/// A retained worktree with no recorded candidate is an inconsistent ledger,
/// not a completed producer outcome: the WP recovers to Doing.
#[test]
fn implement_recovers_review_worktree_without_recorded_candidate() {
    block_on(async {
        let storage = SqliteStorageAdapter::in_memory().expect("in-memory db");
        let (_, wp_id) = seed(&storage, "review-unrecorded", FeatureState::Implementing).await;
        StoragePort::update_wp_state(&storage, wp_id, WpState::Review)
            .await
            .unwrap();
        let vcs = TempVcs::new().with_retained_candidate("review-unrecorded", "WP01");
        let agent = ScriptedAgent::success();

        run_implement(args("review-unrecorded"), &storage, &vcs, &agent)
            .await
            .expect("recovering an unrecorded Review candidate must succeed");

        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            wp.state,
            WpState::Doing,
            "a worktree name alone cannot keep a WP counted as a Review outcome"
        );
        assert!(
            vcs.created_worktrees().is_empty(),
            "the canonical worktree must be reused, not re-created: {:?}",
            vcs.created_worktrees()
        );
        assert_eq!(
            agent.polls(),
            1,
            "recovery must re-dispatch instead of returning before dispatch"
        );
        vcs.cleanup();
    });
}

/// A VCS listing failure must not be reported as a missing candidate.
#[test]
fn implement_propagates_worktree_listing_failure() {
    block_on_paused(async {
        use agileplus_cli::commands::evidence::{
            AttachArgs, EvidenceArgs, EvidenceCommand, run_evidence,
        };
        use agileplus_domain::domain::governance::{GovernanceContract, GovernanceRule};

        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let (fid, wp_id) = seed(&storage, "vcs-failure", FeatureState::Planned).await;
        StoragePort::create_governance_contract(
            &storage,
            &GovernanceContract {
                id: 0,
                feature_id: fid,
                version: 1,
                rules: vec![GovernanceRule {
                    transition: "WP01: Doing -> Review".into(),
                    required_evidence: vec!["FR-CI:ci_output".into()],
                    policy_refs: vec![],
                }],
                bound_at: chrono::Utc::now(),
            },
        )
        .await
        .unwrap();
        let artifact = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(artifact.path(), b"ci output").unwrap();
        run_evidence(
            EvidenceArgs {
                command: EvidenceCommand::Attach(AttachArgs {
                    feature: "vcs-failure".into(),
                    wp: "WP01".into(),
                    fr: "FR-CI".into(),
                    evidence_type: "ci_output".into(),
                    artifact: artifact.path().to_path_buf(),
                    result: "pass".into(),
                }),
            },
            &storage,
        )
        .await
        .unwrap();

        let vcs = TempVcs::new().with_list_worktrees_error();
        let agent = ScriptedAgent::success();
        run_implement(args("vcs-failure"), &storage, &vcs, &agent)
            .await
            .expect("a listing fault must be recorded, not abort the run");

        // The approved worker finished but the candidate could not be
        // resolved, so the attempt has to be terminal. A Running attempt with
        // no ended_at reads as a live job in the ledger.
        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        let assignment =
            agileplus_domain::ports::ExecutionRecordPort::get_active_assignment(&storage, wp.id)
                .await
                .unwrap()
                .expect("the dispatch must have an active assignment");
        let attempts =
            agileplus_domain::ports::ExecutionRecordPort::list_attempts(&storage, &assignment.id)
                .await
                .unwrap();
        let attempt = attempts.last().expect("the dispatch must have an attempt");
        assert_eq!(
            attempt.status,
            agileplus_domain::domain::execution::AttemptStatus::Failed,
            "a listing fault must leave a terminal attempt"
        );
        assert_eq!(
            attempt.failure_class.as_deref(),
            Some("candidate_list_failed")
        );
        assert!(
            attempt.ended_at.is_some(),
            "a terminal attempt needs ended_at"
        );
        vcs.cleanup();
    });
}

/// A Review prerequisite releases its dependent for producer work without
/// being accepted: the prerequisite stays in Review.
#[test]
fn implement_review_prerequisite_releases_dependent_without_accepting_it() {
    block_on_paused(async {
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let (_, first_id) = seed(&storage, "dependency-gate", FeatureState::Implementing).await;
        let mut second = agileplus_domain::domain::work_package::WorkPackage::new(
            first_id, // replaced below with the real feature id
            "Second", 2, "works",
        );
        let fid = StoragePort::get_work_package(&storage, first_id)
            .await
            .unwrap()
            .unwrap()
            .feature_id;
        second.feature_id = fid;
        second.id = 0;
        let second_id = StoragePort::create_work_package(&storage, &second)
            .await
            .unwrap();
        StoragePort::add_wp_dependency(
            &storage,
            &WpDependency {
                wp_id: second_id,
                depends_on: first_id,
                dep_type: DependencyType::Explicit,
            },
        )
        .await
        .unwrap();
        StoragePort::update_wp_state(&storage, first_id, WpState::Review)
            .await
            .unwrap();
        seed_recorded_candidate(&storage, fid, first_id).await;

        let vcs = TempVcs::new().with_retained_candidate("dependency-gate", "WP01");
        let agent = ScriptedAgent::success();
        run_implement(args("dependency-gate"), &storage, &vcs, &agent)
            .await
            .expect("scheduling a dependent of an unaccepted prerequisite must succeed");

        let prerequisite = StoragePort::get_work_package(&storage, first_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            prerequisite.state,
            WpState::Review,
            "scheduling dependents must not accept the prerequisite"
        );
        let dependent = StoragePort::get_work_package(&storage, second_id)
            .await
            .unwrap()
            .unwrap();
        assert_ne!(
            dependent.state,
            WpState::Planned,
            "producer completion releases the dependent for implementation"
        );
        assert_eq!(
            agent.polls(),
            1,
            "the dependent must be dispatched once its prerequisite produced a candidate"
        );
        vcs.cleanup();
    });
}

/// A Review WP with no retained candidate is re-dispatched instead of stranded.
#[test]
fn implement_recovers_review_wp_without_candidate() {
    block_on_paused(async {
        use agileplus_domain::domain::governance::{GovernanceContract, GovernanceRule};

        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let (fid, wp_id) = seed(&storage, "review-recovery", FeatureState::Implementing).await;
        StoragePort::create_governance_contract(
            &storage,
            &GovernanceContract {
                id: 0,
                feature_id: fid,
                version: 1,
                rules: vec![GovernanceRule {
                    transition: "WP01: Doing -> Review".into(),
                    required_evidence: vec!["FR-CI:ci_output".into()],
                    policy_refs: vec![],
                }],
                bound_at: chrono::Utc::now(),
            },
        )
        .await
        .unwrap();
        StoragePort::update_wp_state(&storage, wp_id, WpState::Review)
            .await
            .unwrap();

        let vcs = TempVcs::new();
        let agent = ScriptedAgent::success();
        run_implement(args("review-recovery"), &storage, &vcs, &agent)
            .await
            .expect("Review candidate recovery must succeed");

        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            wp.state,
            WpState::Doing,
            "recovered candidate awaits evidence before Review"
        );
        assert_eq!(vcs.created_worktrees().len(), 1);
        assert_eq!(agent.polls(), 1);
        vcs.cleanup();
    });
}

/// The run summary must count a WP that reached Review in this run.
#[test]
fn implement_summary_counts_wp_moved_to_review() {
    block_on_paused(async {
        use agileplus_cli::commands::evidence::{
            AttachArgs, EvidenceArgs, EvidenceCommand, run_evidence,
        };
        use agileplus_domain::domain::governance::{GovernanceContract, GovernanceRule};

        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let (fid, wp_id) = seed(&storage, "summary-review", FeatureState::Planned).await;
        StoragePort::create_governance_contract(
            &storage,
            &GovernanceContract {
                id: 0,
                feature_id: fid,
                version: 1,
                rules: vec![GovernanceRule {
                    transition: "WP01: Doing -> Review".into(),
                    required_evidence: vec!["FR-CI:ci_output".into()],
                    policy_refs: vec![],
                }],
                bound_at: chrono::Utc::now(),
            },
        )
        .await
        .unwrap();
        let artifact = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            artifact.path(),
            b"isolated fixture CI output for summary test",
        )
        .unwrap();
        run_evidence(
            EvidenceArgs {
                command: EvidenceCommand::Attach(AttachArgs {
                    feature: "summary-review".into(),
                    wp: "WP01".into(),
                    fr: "FR-CI".into(),
                    evidence_type: "ci_output".into(),
                    artifact: artifact.path().to_path_buf(),
                    result: "pass".into(),
                }),
            },
            &storage,
        )
        .await
        .unwrap();

        let vcs = TempVcs::new();
        let agent = ScriptedAgent::success();
        run_implement(args("summary-review"), &storage, &vcs, &agent)
            .await
            .expect("implement should succeed");

        // The WP must be in Review from this run's own single candidate. The
        // summary count is derived from the same mutable set that records that
        // transition, so this state assertion pins the counted outcome.
        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wp.state, WpState::Review);
        assert_eq!(
            vcs.created_worktrees().len(),
            1,
            "the Review transition should come from this run's single candidate"
        );
        let completed = std::collections::HashSet::from([wp.id]);
        assert_eq!(
            summary_counts(&std::collections::HashSet::from([wp.id]), &completed),
            (1, 1),
            "the Review run must report one completed work package out of one"
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

/// Agent success does not imply CI or independent review approval. The WP
/// remains Doing and its worktree is retained when required evidence is absent.
#[test]
fn implement_agent_success_requires_contract_evidence_and_retains_worktree() {
    block_on_paused(async {
        let storage = SqliteStorageAdapter::in_memory().expect("in-memory db");
        let (fid, wp_id) = seed(&storage, "happy", FeatureState::Planned).await;
        use agileplus_domain::domain::governance::{GovernanceContract, GovernanceRule};
        StoragePort::create_governance_contract(
            &storage,
            &GovernanceContract {
                id: 0,
                feature_id: fid,
                version: 1,
                rules: vec![
                    GovernanceRule {
                        transition: "WP01: Doing -> Review".into(),
                        required_evidence: vec!["FR-CI:ci_output".into()],
                        policy_refs: vec![],
                    },
                    GovernanceRule {
                        transition: "WP01: Review -> Done".into(),
                        required_evidence: vec!["FR-REVIEW:review_approval".into()],
                        policy_refs: vec![],
                    },
                ],
                bound_at: chrono::Utc::now(),
            },
        )
        .await
        .unwrap();
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

        // Agent completion cannot bypass the missing CI evidence.
        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            wp.state,
            WpState::Doing,
            "agent completion is not CI approval"
        );

        // Keep the worktree available for evidence collection and review.
        let created = vcs.created_worktrees();
        assert_eq!(created.len(), 1, "one worktree per WP");
        let cleaned = vcs.cleaned_worktrees();
        assert!(cleaned.is_empty(), "unaccepted worktree must be retained");

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

        // Only the genuine feature transition is recorded, not false completion.
        let entries = StoragePort::get_audit_trail(&storage, fid).await.unwrap();
        let transitions: Vec<&str> = entries.iter().map(|e| e.transition.as_str()).collect();
        assert!(
            transitions
                .iter()
                .any(|t| t.contains("Planned -> Implementing")),
            "expected the feature transition in the audit chain, got {transitions:?}"
        );
        assert!(
            !transitions.iter().any(|t| t.contains("-> Done")),
            "must not audit unapproved completion, got {transitions:?}"
        );

        vcs.cleanup();
    });
}

/// An empty contract is not an implicit authorization to complete agent work.
#[test]
fn implement_agent_success_with_empty_contract_keeps_work_in_progress() {
    block_on_paused(async {
        use agileplus_domain::domain::governance::GovernanceContract;
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let (fid, wp_id) = seed(&storage, "empty-contract", FeatureState::Planned).await;
        StoragePort::create_governance_contract(
            &storage,
            &GovernanceContract {
                id: 0,
                feature_id: fid,
                version: 1,
                rules: vec![],
                bound_at: chrono::Utc::now(),
            },
        )
        .await
        .unwrap();
        let vcs = TempVcs::new();
        let agent = ScriptedAgent::success();
        run_implement(args("empty-contract"), &storage, &vcs, &agent)
            .await
            .unwrap();
        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wp.state, WpState::Doing);
        assert!(vcs.cleaned_worktrees().is_empty());
        assert_eq!(agent.polls(), 1);
        vcs.cleanup();
    });
}

/// A simulated CI artifact is confined to this in-memory test fixture. Even
/// with that gate satisfied, agent success cannot invent review approval.
#[test]
fn implement_ci_fixture_without_review_stays_in_review_and_retains_worktree() {
    block_on_paused(async {
        use agileplus_cli::commands::evidence::{
            AttachArgs, EvidenceArgs, EvidenceCommand, run_evidence,
        };
        use agileplus_domain::domain::governance::{GovernanceContract, GovernanceRule};
        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let (fid, wp_id) = seed(&storage, "pending-review", FeatureState::Planned).await;
        StoragePort::create_governance_contract(
            &storage,
            &GovernanceContract {
                id: 0,
                feature_id: fid,
                version: 1,
                rules: vec![
                    GovernanceRule {
                        transition: "WP01: Doing -> Review".into(),
                        required_evidence: vec!["FR-CI:ci_output".into()],
                        policy_refs: vec![],
                    },
                    GovernanceRule {
                        transition: "WP01: Review -> Done".into(),
                        required_evidence: vec!["FR-REVIEW:review_approval".into()],
                        policy_refs: vec![],
                    },
                ],
                bound_at: chrono::Utc::now(),
            },
        )
        .await
        .unwrap();
        let artifact = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(artifact.path(), b"Simulated CI output for an isolated negative review-gate test; not real CI acceptance.").unwrap();
        run_evidence(
            EvidenceArgs {
                command: EvidenceCommand::Attach(AttachArgs {
                    feature: "pending-review".into(),
                    wp: "WP01".into(),
                    fr: "FR-CI".into(),
                    evidence_type: "ci_output".into(),
                    artifact: artifact.path().to_path_buf(),
                    result: "pass".into(),
                }),
            },
            &storage,
        )
        .await
        .unwrap();
        let vcs = TempVcs::new();
        let agent = ScriptedAgent::success();
        run_implement(args("pending-review"), &storage, &vcs, &agent)
            .await
            .unwrap();
        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wp.state, WpState::Review);
        assert!(vcs.cleaned_worktrees().is_empty());
        let audit = StoragePort::get_audit_trail(&storage, fid).await.unwrap();
        assert!(
            audit
                .iter()
                .any(|entry| entry.transition == "WP01: Doing -> Review")
        );
        assert!(
            !audit
                .iter()
                .any(|entry| entry.transition.contains("-> Done"))
        );
        assert!(
            StoragePort::get_evidence_by_wp(&storage, wp_id)
                .await
                .unwrap()
                .iter()
                .all(|e| e.fr_id != "FR-REVIEW")
        );
        vcs.cleanup();
    });
}

/// A successful agent run with both evidence gates satisfied must leave a
/// terminal, candidate-bearing Attempt. The work package reaching Done is not
/// enough: the producer record is what an independent evaluator consumes.
#[test]
fn implement_success_records_completed_attempt_with_candidate() {
    block_on_paused(async {
        use agileplus_cli::commands::evidence::{
            AttachArgs, EvidenceArgs, EvidenceCommand, run_evidence,
        };
        use agileplus_domain::domain::execution::AttemptStatus;
        use agileplus_domain::domain::governance::{GovernanceContract, GovernanceRule};
        use agileplus_domain::ports::ExecutionRecordPort;

        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let (fid, wp_id) = seed(&storage, "complete", FeatureState::Planned).await;
        StoragePort::create_governance_contract(
            &storage,
            &GovernanceContract {
                id: 0,
                feature_id: fid,
                version: 1,
                rules: vec![
                    GovernanceRule {
                        transition: "WP01: Doing -> Review".into(),
                        required_evidence: vec!["FR-CI:ci_output".into()],
                        policy_refs: vec![],
                    },
                    GovernanceRule {
                        transition: "WP01: Review -> Done".into(),
                        required_evidence: vec!["FR-REVIEW:review_approval".into()],
                        policy_refs: vec![],
                    },
                ],
                bound_at: chrono::Utc::now(),
            },
        )
        .await
        .unwrap();

        let mut artifacts = Vec::new();
        for (fr, kind, payload) in [
            ("FR-CI", "ci_output", "isolated fixture CI output"),
            (
                "FR-REVIEW",
                "review_approval",
                "isolated fixture review approval",
            ),
        ] {
            let artifact = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(artifact.path(), payload).unwrap();
            let artifact_path = artifact.path().to_path_buf();
            artifacts.push(artifact);
            run_evidence(
                EvidenceArgs {
                    command: EvidenceCommand::Attach(AttachArgs {
                        feature: "complete".into(),
                        wp: "WP01".into(),
                        fr: fr.into(),
                        evidence_type: kind.into(),
                        artifact: artifact_path,
                        result: "pass".into(),
                    }),
                },
                &storage,
            )
            .await
            .unwrap();
        }

        let vcs = TempVcs::new();
        let agent = ScriptedAgent::success();
        run_implement(args("complete"), &storage, &vcs, &agent)
            .await
            .expect("implement should succeed");

        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            wp.state,
            WpState::Review,
            "agent completion moves the WP to Review, not terminal Done"
        );
        let audit = StoragePort::get_audit_trail(&storage, fid).await.unwrap();
        assert!(
            audit
                .iter()
                .any(|entry| entry.transition == "WP01: Doing -> Review")
        );
        assert!(
            !audit
                .iter()
                .any(|entry| entry.transition.contains("-> Done")),
            "agent completion must not self-award terminal Done"
        );

        let assignment = ExecutionRecordPort::get_active_assignment(&storage, wp_id)
            .await
            .unwrap()
            .expect("active assignment");
        let attempts = ExecutionRecordPort::list_attempts(&storage, &assignment.id)
            .await
            .unwrap();
        assert_eq!(attempts.len(), 1, "one attempt per invocation");
        let attempt = &attempts[0];
        assert_eq!(
            attempt.status,
            AttemptStatus::Completed,
            "successful implementation must persist a Completed attempt"
        );
        assert!(
            attempt.result_candidate_ref.is_some(),
            "successful implementation must persist the exact candidate"
        );
        assert!(
            attempt.ended_at.is_some(),
            "terminal attempt must record ended_at"
        );

        vcs.cleanup();
    });
}

/// The producer, the independent evaluator, and the acceptance transaction form
/// one normal CLI sequence: implement, evaluate, validate. A Review candidate
/// must reach `accept_feature` without first being marked Done.
#[test]
fn implement_then_evaluate_then_validate_accepts_review_candidate() {
    block_on_paused(async {
        use agileplus_cli::commands::evaluate::{EvaluateArgs, run_evaluate};
        use agileplus_cli::commands::evidence::{
            AttachArgs, EvidenceArgs, EvidenceCommand, run_evidence,
        };
        use agileplus_cli::commands::validate::{ValidateArgs, run_validate};
        use agileplus_domain::domain::governance::{GovernanceContract, GovernanceRule};
        use agileplus_domain::ports::ExecutionRecordPort;
        use agileplus_domain::ports::execution::AtomicAcceptancePort;
        use agileplus_git::GitVcsAdapter;

        let storage = SqliteStorageAdapter::in_memory().unwrap();
        let (fid, wp_id) = seed(&storage, "journey", FeatureState::Planned).await;
        StoragePort::create_governance_contract(
            &storage,
            &GovernanceContract {
                id: 0,
                feature_id: fid,
                version: 1,
                rules: vec![
                    GovernanceRule {
                        transition: "WP01: Doing -> Review".into(),
                        required_evidence: vec!["FR-CI:ci_output".into()],
                        policy_refs: vec![],
                    },
                    GovernanceRule {
                        transition: "WP01: Review -> Done".into(),
                        required_evidence: vec!["FR-REVIEW:review_approval".into()],
                        policy_refs: vec![],
                    },
                ],
                bound_at: chrono::Utc::now(),
            },
        )
        .await
        .unwrap();

        let mut artifacts = Vec::new();
        for (fr, kind) in [("FR-CI", "ci_output"), ("FR-REVIEW", "review_approval")] {
            let artifact = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(artifact.path(), b"isolated fixture evidence").unwrap();
            let path = artifact.path().to_path_buf();
            artifacts.push(artifact);
            run_evidence(
                EvidenceArgs {
                    command: EvidenceCommand::Attach(AttachArgs {
                        feature: "journey".into(),
                        wp: "WP01".into(),
                        fr: fr.into(),
                        evidence_type: kind.into(),
                        artifact: path,
                        result: "pass".into(),
                    }),
                },
                &storage,
            )
            .await
            .unwrap();
        }

        let vcs = TempVcs::new();
        let agent = ScriptedAgent::success();
        run_implement(args("journey"), &storage, &vcs, &agent)
            .await
            .expect("implement should succeed");

        let assignment = ExecutionRecordPort::get_active_assignment(&storage, wp_id)
            .await
            .unwrap()
            .expect("active assignment");
        let criteria = ExecutionRecordPort::list_assignment_criteria(&storage, &assignment.id)
            .await
            .unwrap();
        let attempts = ExecutionRecordPort::list_attempts(&storage, &assignment.id)
            .await
            .unwrap();
        let candidate = attempts
            .iter()
            .find_map(|attempt| attempt.result_candidate_ref.clone())
            .expect("producer captured a candidate");

        let criterion_results: Vec<serde_json::Value> = criteria
            .iter()
            .map(|criterion| {
                serde_json::json!({
                    "criterion_id": criterion.id.clone(),
                    "result": "satisfied",
                    "evidence_refs": ["evidence:independent"],
                    "rationale": "independent fixture evaluation",
                })
            })
            .collect();
        let receipt = serde_json::json!({
            "candidate_ref": candidate,
            "evaluator_id": "independent-evaluator",
            "evaluator_version": "1",
            "criterion_results": criterion_results,
        });
        let receipt_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            receipt_file.path(),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();

        run_evaluate(
            EvaluateArgs {
                feature: "journey".into(),
                wp: "WP01".into(),
                receipt: receipt_file.path().to_path_buf(),
            },
            &storage,
        )
        .await
        .expect("independent evaluation should be recorded");

        let artifact_root = tempfile::TempDir::new().expect("create artifact root");
        let git_vcs = GitVcsAdapter::new(artifact_root.path().to_path_buf());
        run_validate(
            ValidateArgs {
                feature: "journey".into(),
                format: "markdown".into(),
                skip_policies: false,
                output: None,
                force: false,
            },
            &storage,
            &git_vcs,
        )
        .await
        .expect("validate should accept the Review candidate");

        let feature = StoragePort::get_feature_by_slug(&storage, "journey")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(feature.state, FeatureState::Validated);
        let wp = StoragePort::get_work_package(&storage, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wp.state, WpState::Done);
        let accepted = AtomicAcceptancePort::get_feature_acceptance_receipt(&storage, fid)
            .await
            .unwrap();
        assert!(accepted.is_some(), "acceptance receipt must be durable");
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
        let by_seq = |seq: i32| wps.iter().find(|w| w.sequence == seq).unwrap().state;
        assert_eq!(by_seq(1), WpState::Planned, "WP01 must be untouched");
        assert_eq!(
            by_seq(2),
            WpState::Doing,
            "WP02 awaits its governance contract"
        );

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
