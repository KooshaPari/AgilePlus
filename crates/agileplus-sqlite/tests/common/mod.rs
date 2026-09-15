//! Shared test helpers for integration tests.

use agileplus_sqlite::SqliteStorageAdapter;

/// Create an in-memory SQLite adapter with all migrations applied.
pub fn make_adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("failed to create in-memory adapter")
}

/// Seed a feature row directly (used as prerequisite for audit/evidence/WP tests).
pub fn seed_feature(conn: &rusqlite::Connection, id: i64, slug: &str) {
    conn.execute(
        "INSERT OR IGNORE INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'created', X'00', 'main', datetime('now'), datetime('now'))",
        rusqlite::params![id, slug, format!("Feature {id}")],
    )
    .expect("failed to seed feature");
}

/// Seed a project row directly.
pub fn seed_project(conn: &rusqlite::Connection, id: i64, slug: &str, name: &str) {
    conn.execute(
        "INSERT OR IGNORE INTO projects (id, slug, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, '', datetime('now'), datetime('now'))",
        rusqlite::params![id, slug, name],
    )
    .expect("failed to seed project");
}

/// Seed an epic row directly.
pub fn seed_epic(conn: &rusqlite::Connection, id: i64, project_id: i64) {
    conn.execute(
        "INSERT OR IGNORE INTO epics (id, project_id, title, description, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, '', 'backlog', datetime('now'), datetime('now'))",
        rusqlite::params![id, project_id, format!("Epic {id}")],
    )
    .expect("failed to seed epic");
}
