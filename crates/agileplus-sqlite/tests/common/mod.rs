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

/// Seed a fully valid feature row: RFC3339 timestamps and a 32-byte spec_hash,
/// so the row parses cleanly through `repository::features::row_to_feature`.
pub fn seed_feature_valid(conn: &rusqlite::Connection, id: i64, slug: &str) {
    conn.execute(
        "INSERT OR REPLACE INTO features
         (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'created', ?4, 'main', ?5, ?5)",
        rusqlite::params![
            id,
            slug,
            format!("Feature {slug}"),
            vec![0u8; 32],
            chrono::Utc::now().to_rfc3339()
        ],
    )
    .expect("failed to seed valid feature");
}

/// Seed a work package row (FK prerequisite for evidence / audit-log tests).
pub fn seed_work_package(conn: &rusqlite::Connection, id: i64, feature_id: i64) {
    conn.execute(
        "INSERT OR REPLACE INTO work_packages
         (id, feature_id, title, state, sequence, file_scope, acceptance_criteria, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'planned', 1, '[]', 'ac', ?4, ?4)",
        rusqlite::params![
            id,
            feature_id,
            format!("WP {id}"),
            chrono::Utc::now().to_rfc3339()
        ],
    )
    .expect("failed to seed work package");
}

/// Seed a user row.
pub fn seed_user(conn: &rusqlite::Connection, id: i64, email: &str) {
    conn.execute(
        "INSERT OR REPLACE INTO users
         (id, display_name, email, role, status, avatar_url, github_login, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'member', 'active', NULL, NULL, ?4, ?4)",
        rusqlite::params![id, format!("User {id}"), email, chrono::Utc::now().to_rfc3339()],
    )
    .expect("failed to seed user");
}
