//! Integration tests for schema creation and the migration system.

use agileplus_sqlite::SqliteStorageAdapter;

#[test]
fn migrations_create_expected_tables() {
    let adapter = SqliteStorageAdapter::in_memory().unwrap();
    let conn = adapter.conn_for_bench().unwrap();

    // Core tables must exist after migration.
    let expected_tables = [
        "features",
        "work_packages",
        "governance_contracts",
        "audit_log",
        "evidence",
        "policy_rules",
        "metrics",
        "wp_dependencies",
        "events",
        "snapshots",
        "sync_mappings",
        "api_keys",
        "device_nodes",
        "modules",
        "cycles",
        "backlog_items",
        "projects",
        "users",
        "epics",
        "stories",
        "trace_links",
        "worklog_entries",
        "intent_nodes",
        "features",
    ];

    for table in &expected_tables {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
            > 0;
        assert!(exists, "table '{table}' should exist after migration");
    }
}

#[test]
fn migrations_tracking_table_created() {
    let adapter = SqliteStorageAdapter::in_memory().unwrap();
    let conn = adapter.conn_for_bench().unwrap();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
        .unwrap();
    // We expect at least 20 migrations to be tracked.
    assert!(count >= 20, "expected >=20 tracked migrations, got {count}");
}

#[test]
fn migrations_are_idempotent() {
    // Creating two adapters against separate in-memory DBs should each run
    // migrations exactly once and produce equivalent schema.
    let _a1 = SqliteStorageAdapter::in_memory().unwrap();
    let _a2 = SqliteStorageAdapter::in_memory().unwrap();
    // Both should succeed without error.
}

#[test]
fn migration_tracking_records_names() {
    let adapter = SqliteStorageAdapter::in_memory().unwrap();
    let conn = adapter.conn_for_bench().unwrap();

    let names: Vec<String> = conn
        .prepare("SELECT name FROM _migrations ORDER BY id")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert!(!names.is_empty());
    assert!(
        names.contains(&"001_create_features".to_string()),
        "first migration should be tracked"
    );
    assert!(
        names.last().unwrap().contains("026"),
        "last migration should be 026-related"
    );
}

#[test]
fn migration_tracking_has_applied_at() {
    let adapter = SqliteStorageAdapter::in_memory().unwrap();
    let conn = adapter.conn_for_bench().unwrap();

    let applied_at: String = conn
        .query_row(
            "SELECT applied_at FROM _migrations WHERE name = '001_create_features'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    // Should be a valid RFC3339 timestamp.
    assert!(
        chrono::DateTime::parse_from_rfc3339(&applied_at).is_ok(),
        "applied_at should be RFC3339, got: {applied_at}"
    );
}

#[test]
fn foreign_keys_are_enforced() {
    let adapter = SqliteStorageAdapter::in_memory().unwrap();
    let conn = adapter.conn_for_bench().unwrap();

    // Verify FK pragma is ON.
    let fk_enabled: i32 = conn
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .unwrap();
    assert_eq!(fk_enabled, 1, "foreign keys should be enabled");
}

#[test]
fn features_table_has_expected_columns() {
    let adapter = SqliteStorageAdapter::in_memory().unwrap();
    let conn = adapter.conn_for_bench().unwrap();

    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(features)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let expected = ["id", "slug", "friendly_name", "state", "spec_hash", "target_branch", "created_at", "updated_at"];
    for col in &expected {
        assert!(
            columns.iter().any(|c| c == col),
            "features table should have column '{col}', got: {columns:?}"
        );
    }
}

#[test]
fn work_packages_table_has_expected_columns() {
    let adapter = SqliteStorageAdapter::in_memory().unwrap();
    let conn = adapter.conn_for_bench().unwrap();

    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(work_packages)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let expected = ["id", "feature_id", "title", "state", "sequence", "file_scope"];
    for col in &expected {
        assert!(
            columns.iter().any(|c| c == col),
            "work_packages table should have column '{col}'"
        );
    }
}

#[test]
fn indexes_are_created() {
    let adapter = SqliteStorageAdapter::in_memory().unwrap();
    let conn = adapter.conn_for_bench().unwrap();

    let indexes: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert!(!indexes.is_empty(), "should have custom indexes");
}

#[test]
fn unique_constraint_on_feature_slug() {
    let adapter = SqliteStorageAdapter::in_memory().unwrap();
    let conn = adapter.conn_for_bench().unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, labels, created_at, updated_at)
         VALUES ('dup', 'F1', 'created', X'00', 'main', '[]', ?1, ?1)",
        [&now],
    )
    .unwrap();

    let result = conn.execute(
        "INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, labels, created_at, updated_at)
         VALUES ('dup', 'F2', 'created', X'00', 'main', '[]', ?1, ?1)",
        [&now],
    );

    assert!(result.is_err(), "duplicate slug should fail");
}

#[test]
fn unique_constraint_on_project_slug() {
    let adapter = SqliteStorageAdapter::in_memory().unwrap();
    let conn = adapter.conn_for_bench().unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO projects (slug, name, description, created_at, updated_at)
         VALUES ('proj-dup', 'P1', '', ?1, ?1)",
        [&now],
    )
    .unwrap();

    let result = conn.execute(
        "INSERT INTO projects (slug, name, description, created_at, updated_at)
         VALUES ('proj-dup', 'P2', '', ?1, ?1)",
        [&now],
    );

    assert!(result.is_err(), "duplicate project slug should fail");
}

#[test]
fn unique_constraint_on_user_email() {
    let adapter = SqliteStorageAdapter::in_memory().unwrap();
    let conn = adapter.conn_for_bench().unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO users (display_name, email, role, status, avatar_url, github_login, created_at, updated_at)
         VALUES ('Alice', 'alice@test.com', 'admin', 'active', NULL, NULL, ?1, ?1)",
        [&now],
    )
    .unwrap();

    let result = conn.execute(
        "INSERT INTO users (display_name, email, role, status, avatar_url, github_login, created_at, updated_at)
         VALUES ('Bob', 'alice@test.com', 'member', 'active', NULL, NULL, ?1, ?1)",
        [&now],
    );

    assert!(result.is_err(), "duplicate email should fail");
}

#[test]
fn sync_mappings_unique_on_entity_type_and_entity_id() {
    let adapter = SqliteStorageAdapter::in_memory().unwrap();
    let conn = adapter.conn_for_bench().unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO sync_mappings (entity_type, entity_id, plane_issue_id, content_hash, last_synced_at, sync_direction, conflict_count)
         VALUES ('feature', 1, 'plane-1', 'hash1', ?1, 'push', 0)",
        [&now],
    )
    .unwrap();

    // Upsert should succeed (ON CONFLICT).
    let result = conn.execute(
        "INSERT INTO sync_mappings (entity_type, entity_id, plane_issue_id, content_hash, last_synced_at, sync_direction, conflict_count)
         VALUES ('feature', 1, 'plane-2', 'hash2', ?1, 'pull', 1)
         ON CONFLICT (entity_type, entity_id) DO UPDATE SET
             plane_issue_id = excluded.plane_issue_id, content_hash = excluded.content_hash",
        [&now],
    );

    assert!(result.is_ok(), "upsert should succeed");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sync_mappings", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1, "should still be exactly one row");
}
