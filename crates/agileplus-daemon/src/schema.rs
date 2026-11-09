// SPDX-License-Identifier: MIT OR Apache-2.0
//! Versioned SQLite schema for the device registry.

pub(crate) const SCHEMA_VERSION: u32 = 1;
pub(crate) const REGISTRY_FILE_NAME: &str = "device-registry.sqlite3";
pub(crate) const INITIAL_SCHEMA: &str = r#"
CREATE TABLE registry_metadata (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    schema_version INTEGER NOT NULL,
    revision INTEGER NOT NULL CHECK (revision >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE registered_projects (
    project_uid TEXT PRIMARY KEY,
    canonical_repo_root TEXT NOT NULL UNIQUE,
    database_path TEXT NOT NULL UNIQUE,
    grpc_endpoint TEXT,
    desired_state TEXT NOT NULL CHECK (desired_state IN ('running', 'stopped')),
    observed_state TEXT NOT NULL CHECK (
        observed_state IN ('starting', 'healthy', 'degraded', 'stopped', 'crashed')
    ),
    child_pid INTEGER CHECK (child_pid IS NULL OR child_pid > 0),
    child_revision TEXT,
    last_started_at TEXT,
    last_healthy_at TEXT,
    restart_count INTEGER NOT NULL DEFAULT 0 CHECK (restart_count >= 0),
    registration_revision INTEGER NOT NULL CHECK (registration_revision > 0),
    updated_revision INTEGER NOT NULL CHECK (updated_revision >= registration_revision),
    last_error TEXT
);

CREATE INDEX idx_registered_projects_desired_state
    ON registered_projects(desired_state);
"#;
