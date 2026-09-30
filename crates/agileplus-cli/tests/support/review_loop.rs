// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared fixtures for the `run_review_loop` behaviour tests.
//!
//! Extracted from `tests/review_loop_flow.rs`, which outgrew the project's
//! 500-line file cap. These are the pieces both that file and any future
//! review-loop test need: a scripted `AgentPort` double and the small
//! constructors for the domain values the loop reads.
//!
//! The double replays a sequence of poll replies and records what it was asked,
//! so a test can assert on the sequence of agent interactions and not only on
//! the `ReviewOutcome` the loop finally returns.

use std::collections::VecDeque;
use std::sync::Mutex;

use agileplus_domain::domain::work_package::WorkPackage;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::agent::{
    AgentConfig, AgentKind, AgentPort, AgentResult, AgentStatus, AgentTask,
};

pub fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    tokio_test::block_on(fut)
}

pub fn work_package() -> WorkPackage {
    let mut wp = WorkPackage::new(1, "Review loop WP", 1, "passes review");
    wp.id = 42;
    wp
}

pub fn config() -> AgentConfig {
    AgentConfig {
        kind: AgentKind::ClaudeCode,
        max_review_cycles: 3,
        timeout_secs: 30,
        extra_args: vec![],
    }
}

pub fn result(success: bool, stderr: &str) -> AgentResult {
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
pub enum Step {
    Status(AgentStatus),
    Error(String),
}

/// Scripted `AgentPort` double. Each `query_status` pops the next step; once
/// the script is exhausted it repeats the last step, so multi-cycle loops can
/// be described without knowing exactly how many polls occur.
pub struct ScriptedAgent {
    steps: Mutex<VecDeque<Step>>,
    last: Mutex<Option<Step>>,
    polls: Mutex<usize>,
    queried_job_ids: Mutex<Vec<String>>,
    instructions: Mutex<Vec<String>>,
    send_fails: bool,
}

impl ScriptedAgent {
    pub fn new(steps: Vec<Step>) -> Self {
        Self {
            steps: Mutex::new(steps.into()),
            last: Mutex::new(None),
            polls: Mutex::new(0),
            queried_job_ids: Mutex::new(vec![]),
            instructions: Mutex::new(vec![]),
            send_fails: false,
        }
    }

    pub fn with_send_failure(mut self) -> Self {
        self.send_fails = true;
        self
    }

    pub fn poll_count(&self) -> usize {
        *self.polls.lock().unwrap()
    }

    /// Every job id the loop polled, in order.
    pub fn queried_job_ids(&self) -> Vec<String> {
        self.queried_job_ids.lock().unwrap().clone()
    }

    pub fn instructions(&self) -> Vec<String> {
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
    async fn dispatch(
        &self,
        _task: AgentTask,
        _config: &AgentConfig,
    ) -> Result<AgentResult, DomainError> {
        Err(DomainError::NotFound("not used by review loop".into()))
    }

    async fn dispatch_async(
        &self,
        _task: AgentTask,
        _config: &AgentConfig,
    ) -> Result<String, DomainError> {
        Err(DomainError::NotFound("not used by review loop".into()))
    }

    fn query_status(
        &self,
        job_id: &str,
    ) -> impl std::future::Future<Output = Result<AgentStatus, DomainError>> + Send {
        let step = {
            *self.polls.lock().unwrap() += 1;
            self.queried_job_ids
                .lock()
                .unwrap()
                .push(job_id.to_string());
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

    async fn cancel(&self, _job_id: &str) -> Result<(), DomainError> {
        Ok(())
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
