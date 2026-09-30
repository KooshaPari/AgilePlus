// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for `agileplus review` loop orchestration
//! (commands/review_loop.rs).
//!
//! The in-file tests only cover the pure `format_feedback` helper and the
//! `ReviewOutcome` variants. These drive `run_review_loop` itself with a
//! scripted `AgentPort` double, covering every `AgentStatus` arm: successful
//! completion, failed completion with feedback re-instruction, waiting for
//! review, hard agent failure, running, pending, and status-query errors, plus
//! the max-cycles exit and the resulting `last_feedback` payload.
//!
//! The double and the domain-value constructors live in
//! `tests/support/review_loop.rs`, extracted when this file passed the project's
//! 500-line cap. This file holds the assertions.

mod support;

use support::review_loop::{ScriptedAgent, Step, block_on, config, result, work_package};

use agileplus_cli::commands::review_loop::{ReviewOutcome, run_review_loop};
use agileplus_domain::ports::agent::AgentStatus;

#[test]
fn review_loop_approves_on_successful_completion() {
    block_on(async {
        let agent = ScriptedAgent::new(vec![Step::Status(AgentStatus::Completed {
            result: result(true, ""),
        })]);
        let outcome = run_review_loop(&work_package(), "job-1", &agent, &config(), 3, 0).await;
        assert!(matches!(outcome, ReviewOutcome::Approved));
        assert_eq!(agent.poll_count(), 1, "approved without another poll");
        assert!(
            agent.instructions().is_empty(),
            "no re-instruction on success"
        );
    })
}

#[test]
fn review_loop_approves_when_agent_waits_for_review() {
    block_on(async {
        let agent = ScriptedAgent::new(vec![Step::Status(AgentStatus::WaitingForReview {
            pr_url: "https://example.test/pr/7".to_string(),
        })]);
        let outcome = run_review_loop(&work_package(), "job-1", &agent, &config(), 3, 0).await;
        assert!(matches!(outcome, ReviewOutcome::Approved));
        assert_eq!(agent.poll_count(), 1);
    })
}

#[test]
fn review_loop_reports_agent_hard_failure() {
    block_on(async {
        let agent = ScriptedAgent::new(vec![Step::Status(AgentStatus::Failed {
            error: "agent crashed".to_string(),
        })]);
        let outcome = run_review_loop(&work_package(), "job-1", &agent, &config(), 3, 0).await;
        match outcome {
            ReviewOutcome::AgentFailed { error } => assert_eq!(error, "agent crashed"),
            other => panic!("expected AgentFailed, got {other:?}"),
        }
        assert_eq!(agent.poll_count(), 1, "fatal failure stops the loop");
    })
}

#[test]
fn review_loop_refeeds_failure_stderr_as_instruction() {
    block_on(async {
        let agent = ScriptedAgent::new(vec![
            Step::Status(AgentStatus::Completed {
                result: result(false, "clippy: unused import"),
            }),
            Step::Status(AgentStatus::Completed {
                result: result(true, ""),
            }),
        ]);
        let outcome = run_review_loop(&work_package(), "job-1", &agent, &config(), 3, 0).await;
        assert!(matches!(outcome, ReviewOutcome::Approved));
        let instructions = agent.instructions();
        assert_eq!(instructions.len(), 1, "one retry after the failure");
        let sent = &instructions[0];
        assert!(
            sent.contains("Your previous attempt failed"),
            "instruction must frame the retry: {sent}"
        );
        assert!(
            sent.contains("clippy: unused import"),
            "stderr must be fed back verbatim: {sent}"
        );
    })
}

#[test]
fn review_loop_does_not_refeed_on_last_cycle() {
    block_on(async {
        // A single cycle that fails: cycle == max_cycles, so no retry.
        let agent = ScriptedAgent::new(vec![Step::Status(AgentStatus::Completed {
            result: result(false, "final failure"),
        })]);
        let outcome = run_review_loop(&work_package(), "job-1", &agent, &config(), 1, 0).await;
        match outcome {
            ReviewOutcome::MaxCyclesReached {
                cycles,
                last_feedback,
            } => {
                assert_eq!(cycles, 1);
                assert_eq!(last_feedback, "final failure");
            }
            other => panic!("expected MaxCyclesReached, got {other:?}"),
        }
        assert!(
            agent.instructions().is_empty(),
            "no retry after the final cycle"
        );
    })
}

#[test]
fn review_loop_continues_when_instruction_send_fails() {
    block_on(async {
        let agent = ScriptedAgent::new(vec![
            Step::Status(AgentStatus::Completed {
                result: result(false, "lint error"),
            }),
            Step::Status(AgentStatus::Completed {
                result: result(true, ""),
            }),
        ])
        .with_send_failure();
        let outcome = run_review_loop(&work_package(), "job-1", &agent, &config(), 3, 0).await;
        assert!(
            matches!(outcome, ReviewOutcome::Approved),
            "a failed send_instruction must not abort the loop"
        );
        assert_eq!(agent.instructions().len(), 1, "the attempt was still made");
    })
}

#[test]
fn review_loop_polls_while_agent_is_running() {
    block_on(async {
        let agent = ScriptedAgent::new(vec![
            Step::Status(AgentStatus::Running { pid: 4321 }),
            Step::Status(AgentStatus::Pending),
            Step::Status(AgentStatus::Completed {
                result: result(true, ""),
            }),
        ]);
        let outcome = run_review_loop(&work_package(), "job-1", &agent, &config(), 5, 0).await;
        assert!(matches!(outcome, ReviewOutcome::Approved));
        assert_eq!(
            agent.poll_count(),
            3,
            "running and pending each consume a poll"
        );
    })
}

#[test]
fn review_loop_treats_status_query_error_as_a_retry() {
    block_on(async {
        let agent = ScriptedAgent::new(vec![
            Step::Error("adapter unavailable".to_string()),
            Step::Status(AgentStatus::Completed {
                result: result(true, ""),
            }),
        ]);
        let outcome = run_review_loop(&work_package(), "job-1", &agent, &config(), 3, 0).await;
        assert!(
            matches!(outcome, ReviewOutcome::Approved),
            "a query error must be retried, not fatal"
        );
        assert_eq!(agent.poll_count(), 2);
    })
}

#[test]
fn review_loop_reports_max_cycles_with_last_feedback() {
    block_on(async {
        let agent = ScriptedAgent::new(vec![Step::Status(AgentStatus::Completed {
            result: result(false, "persistent test failure"),
        })]);
        let outcome = run_review_loop(&work_package(), "job-1", &agent, &config(), 3, 0).await;
        match outcome {
            ReviewOutcome::MaxCyclesReached {
                cycles,
                last_feedback,
            } => {
                assert_eq!(cycles, 3, "all three cycles consumed");
                assert_eq!(
                    last_feedback, "persistent test failure",
                    "last failure is surfaced to the caller"
                );
            }
            other => panic!("expected MaxCyclesReached, got {other:?}"),
        }
        assert_eq!(agent.poll_count(), 3);
        assert_eq!(
            agent.instructions().len(),
            2,
            "cycles 1 and 2 re-instruct, cycle 3 does not"
        );
    })
}

#[test]
fn review_loop_max_cycles_with_no_feedback_reports_empty() {
    block_on(async {
        let agent = ScriptedAgent::new(vec![Step::Status(AgentStatus::Pending)]);
        let outcome = run_review_loop(&work_package(), "job-1", &agent, &config(), 2, 0).await;
        match outcome {
            ReviewOutcome::MaxCyclesReached {
                cycles,
                last_feedback,
            } => {
                assert_eq!(cycles, 2);
                assert!(
                    last_feedback.is_empty(),
                    "pending polls produce no feedback"
                );
            }
            other => panic!("expected MaxCyclesReached, got {other:?}"),
        }
    })
}

#[test]
fn review_loop_with_zero_cycles_returns_immediately() {
    block_on(async {
        let agent = ScriptedAgent::new(vec![Step::Status(AgentStatus::Pending)]);
        let outcome = run_review_loop(&work_package(), "job-1", &agent, &config(), 0, 0).await;
        match outcome {
            ReviewOutcome::MaxCyclesReached {
                cycles,
                last_feedback,
            } => {
                assert_eq!(cycles, 0);
                assert!(last_feedback.is_empty());
            }
            other => panic!("expected MaxCyclesReached, got {other:?}"),
        }
        assert_eq!(agent.poll_count(), 0, "no cycles means no polls");
    })
}

/// The loop must poll the job id it was handed. A mismatch would be invisible
/// from the returned `ReviewOutcome` alone, since the outcome does not carry
/// the id, so it is asserted directly on the port.
#[test]
fn review_loop_polls_only_the_requested_job_id() {
    block_on(async {
        let agent = ScriptedAgent::new(vec![Step::Status(AgentStatus::Running { pid: 4242 })]);
        let outcome = run_review_loop(
            &work_package(),
            "the-one-real-job-id",
            &agent,
            &config(),
            3,
            0,
        )
        .await;

        assert!(
            matches!(outcome, ReviewOutcome::MaxCyclesReached { .. }),
            "expected the budget to be spent, got {outcome:?}"
        );

        let queried = agent.queried_job_ids();
        assert_eq!(queried.len(), 3, "expected one poll per cycle");
        assert!(
            queried.iter().all(|id| id == "the-one-real-job-id"),
            "loop polled something other than the requested id: {queried:?}"
        );
    })
}

/// `send_instruction` must also address the same job id, otherwise a
/// corrective instruction would be delivered to a different job.
#[test]
fn review_loop_sends_corrections_to_the_requested_job_id() {
    block_on(async {
        let agent = ScriptedAgent::new(vec![
            Step::Status(AgentStatus::Completed {
                result: result(false, "lint error"),
            }),
            Step::Status(AgentStatus::Completed {
                result: result(true, ""),
            }),
        ]);

        let outcome = run_review_loop(
            &work_package(),
            "correction-target",
            &agent,
            &config(),
            2,
            0,
        )
        .await;

        assert!(
            matches!(outcome, ReviewOutcome::Approved),
            "got {outcome:?}"
        );
        assert_eq!(agent.instructions().len(), 1);
    })
}

/// The printed log truncates stderr at 200 chars, but the instruction handed to
/// the agent must carry all of it. An implementation that reused the truncated
/// slice would still satisfy a `contains` check on a short message, so this
/// uses stderr long enough that truncation would be visible.
#[test]
fn review_loop_sends_untruncated_stderr_to_the_agent() {
    block_on(async {
        let long = "E".repeat(600);
        let agent = ScriptedAgent::new(vec![
            Step::Status(AgentStatus::Completed {
                result: result(false, &long),
            }),
            Step::Status(AgentStatus::Completed {
                result: result(true, ""),
            }),
        ]);

        let outcome = run_review_loop(&work_package(), "job-long", &agent, &config(), 2, 0).await;

        assert!(
            matches!(outcome, ReviewOutcome::Approved),
            "got {outcome:?}"
        );

        let instruction = &agent.instructions()[0];
        assert!(
            instruction.contains(&long),
            "all 600 stderr chars must reach the agent, instruction was {} chars",
            instruction.len()
        );
    })
}
