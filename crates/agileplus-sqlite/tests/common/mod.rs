//! Shared test helpers for integration tests.

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
