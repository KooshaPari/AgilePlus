// SPDX-License-Identifier: MIT
//! E2E test helpers for the AgilePlus Desktop (Tauri) application.
//!
//! Provides shared utilities for CLI invocation, database schema
//! verification, and Tauri configuration assertions used by the
//! desktop lifecycle integration tests.

use std::path::PathBuf;
use tempfile::TempDir;

/// Return the absolute path to the workspace repository root.
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tests/e2e-desktop/")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

/// Return the path to `desktop/src-tauri/` relative to the workspace root.
pub fn desktop_tauri_dir() -> PathBuf {
    repo_root().join("desktop/src-tauri")
}

/// Return the path to the Tauri configuration file.
pub fn tauri_config_path() -> PathBuf {
    desktop_tauri_dir().join("tauri.conf.json")
}

/// Return the path to the icons directory.
pub fn icons_dir() -> PathBuf {
    desktop_tauri_dir().join("icons")
}

/// Create a temporary directory suitable for isolated CLI/database tests.
///
/// The returned `TempDir` is automatically cleaned up when dropped.
pub fn isolated_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// Initialize a SQLite database with the same schema the desktop app uses.
///
/// This mirrors `initialize_database` from `desktop/src-tauri/src/db.rs`
/// so E2E tests can verify schema correctness without linking the desktop crate.
pub fn init_desktop_schema(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS features (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            state TEXT NOT NULL DEFAULT 'created',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS work_packages (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            state TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (feature_id) REFERENCES features(id)
        );

        CREATE TABLE IF NOT EXISTS evidence (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            work_package_id TEXT,
            evidence_type TEXT NOT NULL,
            content TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (feature_id) REFERENCES features(id),
            FOREIGN KEY (work_package_id) REFERENCES work_packages(id)
        );
        ",
    )
}
