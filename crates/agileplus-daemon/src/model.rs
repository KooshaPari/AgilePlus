// SPDX-License-Identifier: MIT OR Apache-2.0
//! Device-registry data model and fail-closed errors.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Desired lifecycle state persisted by the device registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesiredState {
    /// The future supervisor should keep the project core running.
    Running,
    /// The future supervisor should keep the project core stopped.
    Stopped,
}

impl DesiredState {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Stopped => "stopped",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, RegistryError> {
        match value {
            "running" => Ok(Self::Running),
            "stopped" => Ok(Self::Stopped),
            other => Err(RegistryError::Corrupt(format!(
                "unknown desired state {other:?}"
            ))),
        }
    }
}

/// Last observed runtime state for one registered project core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservedState {
    /// The child process has started but is not ready.
    Starting,
    /// The child process passed readiness checks.
    Healthy,
    /// The child process is running but cannot satisfy readiness.
    Degraded,
    /// No child process is expected or observed.
    Stopped,
    /// The last child process exited unexpectedly.
    Crashed,
}

impl ObservedState {
    pub(crate) fn parse(value: &str) -> Result<Self, RegistryError> {
        match value {
            "starting" => Ok(Self::Starting),
            "healthy" => Ok(Self::Healthy),
            "degraded" => Ok(Self::Degraded),
            "stopped" => Ok(Self::Stopped),
            "crashed" => Ok(Self::Crashed),
            other => Err(RegistryError::Corrupt(format!(
                "unknown observed state {other:?}"
            ))),
        }
    }
}

/// Device-local routing and lifecycle record for one canonical Git project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredProject {
    /// Stable AgilePlus project identity on this device.
    pub project_uid: Uuid,
    /// Canonical Git worktree root, with symlinks resolved at registration.
    pub canonical_repo_root: PathBuf,
    /// Derived project-local AgilePlus database path.
    pub database_path: PathBuf,
    /// Future per-project core endpoint allocated by the supervisor.
    pub grpc_endpoint: Option<String>,
    /// Operator-requested process state.
    pub desired_state: DesiredState,
    /// Last state reported by the future supervisor.
    pub observed_state: ObservedState,
    /// Last observed child process identifier.
    pub child_pid: Option<u32>,
    /// Revision of the supervised core binary.
    pub child_revision: Option<String>,
    /// Last child start timestamp.
    pub last_started_at: Option<DateTime<Utc>>,
    /// Last successful readiness timestamp.
    pub last_healthy_at: Option<DateTime<Utc>>,
    /// Number of recorded restart attempts.
    pub restart_count: u32,
    /// Registry revision at initial registration.
    pub registration_revision: u64,
    /// Registry revision that last changed this record.
    pub updated_revision: u64,
    /// Last bounded runtime diagnostic.
    pub last_error: Option<String>,
}

/// Current registry metadata without exposing the SQLite connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrySummary {
    /// Registry file path.
    pub path: PathBuf,
    /// Supported on-disk schema version.
    pub schema_version: u32,
    /// Monotonic registry mutation revision.
    pub revision: u64,
    /// Number of registered projects.
    pub project_count: usize,
}

/// Fail-closed registry errors.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// Filesystem operation failed.
    #[error("{context}: {source}")]
    Io {
        /// Operation that failed.
        context: String,
        /// Original I/O error.
        #[source]
        source: std::io::Error,
    },
    /// SQLite operation failed.
    #[error("{context}: {source}")]
    Sqlite {
        /// Operation that failed.
        context: String,
        /// Original SQLite error.
        #[source]
        source: rusqlite::Error,
    },
    /// Existing registry content failed structural or semantic validation.
    #[error("device registry is corrupt or incomplete: {0}")]
    Corrupt(String),
    /// Existing registry uses a schema this binary does not understand.
    #[error("unsupported device-registry schema version {found}; expected {expected}")]
    UnsupportedSchema {
        /// Supported schema version.
        expected: u32,
        /// Observed schema version.
        found: u32,
    },
    /// Platform did not expose a per-user local data directory.
    #[error("platform user-data directory is unavailable")]
    DataDirectoryUnavailable,
    /// Project path was not a canonical Git worktree root.
    #[error("{path} is not an acceptable Git project root: {detail}")]
    InvalidProjectRoot {
        /// Rejected path.
        path: PathBuf,
        /// Validation failure.
        detail: String,
    },
    /// A required platform path could not be represented without loss.
    #[error("path is not valid UTF-8: {0}")]
    NonUtf8Path(PathBuf),
    /// Canonical root already belongs to another registry record.
    #[error("project root {path} is already registered as {existing_uid}")]
    DuplicateRoot {
        /// Canonical duplicate root.
        path: PathBuf,
        /// Existing stable identity.
        existing_uid: Uuid,
    },
    /// Caller-supplied identity already belongs to another root.
    #[error("project UID {0} is already registered")]
    UidCollision(Uuid),
    /// Requested project identity does not exist.
    #[error("project UID {0} is not registered")]
    ProjectNotFound(Uuid),
    /// Monotonic registry revision overflowed SQLite's signed integer range.
    #[error("device-registry revision overflow")]
    RevisionOverflow,
}
