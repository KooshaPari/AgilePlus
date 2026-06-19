// SPDX-License-Identifier: MIT OR Apache-2.0
//! Agent port — trait + types for dispatching AI agent tasks.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// The kind of agent to dispatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    /// A code-writing / implementation agent.
    Codex,
    /// A review / analysis agent.
    Review,
    /// A generic reasoning agent.
    Generic,
}

/// Status of a running or completed agent task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

/// Configuration for an agent task dispatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub kind: AgentKind,
    /// Maximum wall-clock seconds to wait for the task.
    pub timeout_secs: u64,
    /// Additional model or provider hints (opaque string).
    pub hint: Option<String>,
}

/// A handle to a dispatched agent task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub status: AgentStatus,
    /// Captured stdout / output from the agent.
    pub output: Option<String>,
}

/// Port for dispatching agent tasks and polling their status.
#[async_trait]
pub trait AgentPort: Send + Sync {
    /// Dispatch an agent task with the given prompt and config.
    async fn dispatch(&self, prompt: &str, config: &AgentConfig) -> Result<AgentTask, DomainError>;

    /// Poll the status of a previously dispatched task.
    async fn poll(&self, task_id: &str) -> Result<AgentTask, DomainError>;

    /// Cancel a running task.
    async fn cancel(&self, task_id: &str) -> Result<(), DomainError>;
}
