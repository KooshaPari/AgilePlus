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

use std::collections::VecDeque;
use std::sync::Mutex;

use agileplus_cli::commands::review_loop::{run_review_loop, ReviewOutcome};
use agileplus_domain::domain::work_package::WorkPackage;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::agent::{
    AgentConfig, AgentKind, AgentPort, AgentResult, AgentStatus, AgentTask,
};

fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    tokio_test::block_on(fut)
}

fn work_package() -> WorkPackage {
    let mut wp = WorkPackage::new(1, "Review loop WP", 1, "passes review");
    wp.id = 42;
    wp
}

fn config() -> AgentConfig {
    AgentConfig {
        kind: AgentKind::ClaudeCode,
        max_review_cycles: 3,
        timeout_secs: 30,
        extra_args: vec![],
    }
}

fn result(success: bool, stderr: &str) -> AgentResult {
    AgentResult {
        success,
        pr_url: None,
        commits: vec![],
        stdout: String::new(),
        stderr: stderr.to_string(),
        exit_code: if success { 0 } else { 1 },
    }
}

/// A poll result: either a status or a query error.
#[derive(Clone)]
enum Step {
    Status(AgentStatus),
    Error(String),
}

/// Scripted `AgentPort` double. Each `query_status` pops the next step; once
/// the script is exhausted it repeats the last step, so multi-cycle loops can
/// be described without knowing exactly how many polls occur.
struct ScriptedAgent {
    steps: Mutex<VecDeque<Step>>,
    last: Mutex<Option<Step>>,
    polls: Mutex<usize>,
    instructions: Mutex<Vec<String>>,
    send_fails: bool,
}

impl ScriptedAgent {
    fn new(steps: Vec<Step>) -> Self {
        Self {
            steps: Mutex::new(steps.into()),
            last: Mutex::new(None),
            polls: Mutex::new(0),
            instructions: Mutex::new(vec![]),
            send_fails: false,
        }
    }

    fn with_send_failure(mut self) -> Self {
        self.send_fails = true;
        self
    }

    fn poll_count(&self) -> usize {
        *self.polls.lock().unwrap()
    }

    fn instructions(&self) -> Vec<String> {
        self.instructions.lock().unwrap().clone()
    }
}

/// Build a next step, remembering it as the repeat value once the script runs out.
fn advance(steps: &mut VecDeque<Step>, last: &mut Option<Step>) -> Step {
    let step = match steps.pop_front() {
        Some(step) => step,
        None => last.clone().expect("script must contain at least one step"),
    };
    if steps.is_empty() {
        *last = Some(clone_step(&step));
    }
    step
}

fn clone_step(step: &Step) -> Step {
    match step {
        Step::Status(s) => Step::Status(s.clone()),
        Step::Error(e) => Step::Error(e.clone()),
    }
}

impl AgentPort for ScriptedAgent {
    fn dispatch(
        &self,
        _task: AgentTask,
        _config: &AgentConfig,
    ) -> impl std::future::Future<Output = Result<AgentResult, DomainError>> + Send {
        async { Err(DomainError::NotFound("not used by review loop".into())) }
    }

    fn dispatch_async(
        &self,
        _task: AgentTask,
        _config: &AgentConfig,
    ) -> impl std::future::Future<Output = Result<String, DomainError>> + Send {
        async { Err(DomainError::NotFound("not used by review loop".into())) }
    }

    fn query_status(
        &self,
        _job_id: &str,
    ) -> impl std::future::Future<Output = Result<AgentStatus, DomainError>> + Send {
        let step = {
            *self.polls.lock().unwrap() += 1;
            let mut steps = self.steps.lock().unwrap();
            let mut last = self.last.lock().unwrap();
            advance(&mut steps, &mut last)
        };
        async move {
            match step {
                Step::Status(s) => Ok(s),
                Step::Error(e) => Err(DomainError::NotFound(e)),
            }
        }
    }

    fn cancel(
        &self,
        _job_id: &str,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        async { Ok(()) }
    }

    fn send_instruction(
        &self,
        _job_id: &str,
        instruction: &str,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let fails = self.send_fails;
        let recorded = instruction.to_string();
        self.instructions.lock().unwrap().push(recorded);
        async move {
            if fails {
                Err(DomainError::NotFound("send failed".into()))
            } else {
                Ok(())
            }
        }
    }
}

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
