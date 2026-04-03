//! # Phenotype Process
//!
//! Process management and execution utilities for Phenotype ecosystem.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

// =============================================================================
// Errors
// =============================================================================

/// Process errors
#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Process not found: {0}")]
    NotFound(String),

    #[error("Failed to start process: {0}")]
    StartFailed(String),

    #[error("Process exited with code: {0}")]
    ExitCode(i32),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Timeout exceeded")]
    Timeout,
}

/// Result type for process operations
pub type ProcessResult<T> = Result<T, ProcessError>;

// =============================================================================
// Process Definition
// =============================================================================

/// Process definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDefinition {
    /// Process name
    pub name: String,
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Working directory
    pub cwd: Option<String>,
    /// Process priority
    pub priority: ProcessPriority,
}

/// Process priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessPriority {
    /// Low priority
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Real-time priority
    RealTime,
}

impl Default for ProcessPriority {
    fn default() -> Self {
        Self::Normal
    }
}

// =============================================================================
// Process Status
// =============================================================================

/// Process status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    /// Process is running
    Running,
    /// Process is sleeping
    Sleeping,
    /// Process has stopped
    Stopped,
    /// Process is zombie
    Zombie,
    /// Process has exited
    Exited,
}

/// Process state information
#[derive(Debug, Clone)]
pub struct ProcessState {
    /// Process ID
    pub pid: u32,
    /// Parent process ID
    pub ppid: u32,
    /// Process name
    pub name: String,
    /// Current status
    pub status: ProcessStatus,
    /// CPU usage percentage
    pub cpu_percent: f32,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Start time (unix timestamp)
    pub start_time: i64,
}

// =============================================================================
// Process Manager
// =============================================================================

/// Process manager for handling multiple processes
#[derive(Debug, Clone)]
pub struct ProcessManager {
    processes: HashMap<u32, ProcessState>,
}

impl ProcessManager {
    /// Create a new process manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    /// Register a process
    pub fn register(&mut self, state: ProcessState) {
        self.processes.insert(state.pid, state);
    }

    /// Get process by PID
    #[must_use]
    pub fn get(&self, pid: u32) -> Option<&ProcessState> {
        self.processes.get(&pid)
    }

    /// List all processes
    #[must_use]
    pub fn list(&self) -> Vec<&ProcessState> {
        self.processes.values().collect()
    }

    /// Find processes by name
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Vec<&ProcessState> {
        self.processes
            .values()
            .filter(|p| p.name.contains(name))
            .collect()
    }

    /// Count running processes
    #[must_use]
    pub fn running_count(&self) -> usize {
        self.processes
            .values()
            .filter(|p| p.status == ProcessStatus::Running)
            .count()
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_definition() {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "/usr/bin".to_string());

        let def = ProcessDefinition {
            name: "test".to_string(),
            command: "/bin/test".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
            env,
            cwd: Some("/tmp".to_string()),
            priority: ProcessPriority::Normal,
        };

        assert_eq!(def.name, "test");
        assert_eq!(def.args.len(), 2);
        assert_eq!(def.priority, ProcessPriority::Normal);
    }

    #[test]
    fn test_process_state() {
        let state = ProcessState {
            pid: 1234,
            ppid: 1,
            name: "test".to_string(),
            status: ProcessStatus::Running,
            cpu_percent: 10.5,
            memory_bytes: 1024,
            start_time: 1234567890,
        };

        assert_eq!(state.pid, 1234);
        assert_eq!(state.status, ProcessStatus::Running);
    }

    #[test]
    fn test_process_manager() {
        let mut manager = ProcessManager::new();

        let state = ProcessState {
            pid: 1,
            ppid: 0,
            name: "init".to_string(),
            status: ProcessStatus::Running,
            cpu_percent: 0.0,
            memory_bytes: 0,
            start_time: 0,
        };

        manager.register(state);

        assert_eq!(manager.running_count(), 1);
        assert!(manager.get(1).is_some());
        assert!(manager.get(999).is_none());
    }

    #[test]
    fn test_process_manager_find() {
        let mut manager = ProcessManager::new();

        manager.register(ProcessState {
            pid: 1,
            ppid: 0,
            name: "firefox".to_string(),
            status: ProcessStatus::Running,
            cpu_percent: 10.0,
            memory_bytes: 100,
            start_time: 0,
        });

        manager.register(ProcessState {
            pid: 2,
            ppid: 0,
            name: "chrome".to_string(),
            status: ProcessStatus::Running,
            cpu_percent: 20.0,
            memory_bytes: 200,
            start_time: 0,
        });

        let browsers = manager.find_by_name("fire");
        assert_eq!(browsers.len(), 1);
        assert_eq!(browsers[0].name, "firefox");
    }

    #[test]
    fn test_process_priority_default() {
        assert_eq!(ProcessPriority::default(), ProcessPriority::Normal);
    }
}
