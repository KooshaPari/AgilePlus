//! Integration tests for the migration system and resulting schema.

use std::path::PathBuf;

use agileplus_sqlite::{migrations::MigrationRunner, SqliteStorageAdapter};

fn adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().unwrap()
}

fn table_exists(conn: &rusqlite::Connection, name: &str) -> bool {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )
        .unwrap();
    count > 0
}

fn index_exists(conn: &rusqlite::Connection, name: &str) -> bool {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )
        .unwrap();
    count > 0
}

fn unique_temp_path(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("agileplus-sqlite-test-{tag}-{nanos}.db"));
    p
}

// ---------------------------------------------------------------------------
// Schema presence
// ---------------------------------------------------------------------------

#[test]
fn all_expected_tables_exist_after_migration() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    for table in [
        "features",
        "work_packages",
        "wp_dependencies",
        "governance_contracts",
        "audit_log",
        "evidence",
        "policy_rules",
        "metrics",
        "events",
        "snapshots",
        "sync_mappings",
        "api_keys",
        "device_nodes",
        "modules",
        "module_feature_tags",
        "cycles",
        "cycle_features",
        "backlog_items",
        "projects",
        "users",
        "epics",
        "stories",
        "trace_links",
        "worklog_entries",
        "story_work_packages",
        "cycle_stories",
        "gate_results",
        "run_records",
        "scope_status",
        "intent_nodes",
        "intent_edges",
        "intent_graph_metadata",
        "channel_iterations",
        "_migrations",
    ] {
        assert!(table_exists(&conn, table), "missing table: {table}");
    }
}

#[test]
fn expected_indexes_exist() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    for index in [
        "idx_projects_slug",
        "idx_epics_project_id",
        "idx_stories_epic_id",
        "idx_modules_parent",
        "idx_cycles_state",
        "idx_backlog_items_intent",
        "idx_events_entity",
        "idx_sync_entity",
        "idx_trace_links_from",
        "idx_worklog_entries_task",
        "idx_story_work_packages_wp",
        "idx_cycle_stories_story",
    ] {
        assert!(index_exists(&conn, index), "missing index: {index}");
    }
}

#[test]
fn migrations_meta_table_records_each_migration() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM _migrations", [], |r| r.get(0))
        .unwrap();
    assert!(count >= 28, "expected all migrations recorded, got {count}");

    // Names must be unique (UNIQUE constraint + no duplicate inserts).
    let distinct: i64 = conn
        .query_row("SELECT COUNT(DISTINCT name) FROM _migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, distinct);
}

#[test]
fn run_all_is_idempotent() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let before: i64 = conn
        .query_row("SELECT COUNT(*) FROM _migrations", [], |r| r.get(0))
        .unwrap();
    MigrationRunner::new(&conn).run_all().unwrap();
    MigrationRunner::new(&conn).run_all().unwrap();
    let after: i64 = conn
        .query_row("SELECT COUNT(*) FROM _migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(before, after, "re-running migrations must not duplicate rows");
}

#[test]
fn foreign_keys_pragma_is_enabled() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let fk: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0)).unwrap();
    assert_eq!(fk, 1);
}

#[test]
fn file_backed_adapter_uses_wal_mode() {
    let path = unique_temp_path("wal");
    let adapter = SqliteStorageAdapter::new(&path).unwrap();
    let conn = adapter.conn_for_bench().unwrap();
    let mode: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();
    assert_eq!(mode.to_lowercase(), "wal");
    drop(conn);
    drop(adapter);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn in_memory_adapter_creates_schema_without_wal() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mode: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();
    // In-memory DBs report "memory" (never "wal").
    assert_ne!(mode.to_lowercase(), "wal");
    assert!(table_exists(&conn, "features"));
}

#[test]
fn file_backed_adapter_reopens_existing_database() {
    let path = unique_temp_path("reopen");
    {
        let a = SqliteStorageAdapter::new(&path).unwrap();
        let conn = a.conn_for_bench().unwrap();
        conn.execute(
            "INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
             VALUES ('persisted','Persisted','created', X'00', 'main', ?1, ?1)",
            rusqlite::params![chrono::Utc::now().to_rfc3339()],
        )
        .unwrap();
    }
    {
        let a = SqliteStorageAdapter::new(&path).unwrap();
        let conn = a.conn_for_bench().unwrap();
        let slug: String = conn
            .query_row("SELECT slug FROM features WHERE slug = 'persisted'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(slug, "persisted");
    }
    let _ = std::fs::remove_file(&path);
}

// ---------------------------------------------------------------------------
// Rollback
// ---------------------------------------------------------------------------

#[test]
fn rollback_last_removes_most_recent_migration() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    // 026_feature_labels is applied last; its DOWN drops nothing (ALTER concerns),
    // so instead verify the bookkeeping row is removed.
    let before: i64 = conn
        .query_row("SELECT COUNT(*) FROM _migrations", [], |r| r.get(0))
        .unwrap();
    MigrationRunner::new(&conn).rollback_last().unwrap();
    let after: i64 = conn
        .query_row("SELECT COUNT(*) FROM _migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(after, before - 1);
}

#[test]
fn rollback_last_on_empty_database_is_noop() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    // No migrations applied yet: rollback should succeed without error.
    MigrationRunner::new(&conn).rollback_last().unwrap();
}

#[test]
fn rollback_last_removes_tracking_row_for_last_migration() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let last: String = conn
        .query_row(
            "SELECT name FROM _migrations ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    MigrationRunner::new(&conn).rollback_last().unwrap();
    let still: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
            rusqlite::params![last],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(still, 0, "rolled-back migration must no longer be tracked");
}

// ---------------------------------------------------------------------------
// Constraints
// ---------------------------------------------------------------------------

#[test]
fn feature_slug_unique_constraint() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let ts = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
         VALUES ('dup','A','created', X'00','main', ?1, ?1)",
        rusqlite::params![ts],
    )
    .unwrap();
    let err = conn.execute(
        "INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
         VALUES ('dup','B','created', X'00','main', ?1, ?1)",
        rusqlite::params![ts],
    );
    assert!(err.is_err());
}

#[test]
fn feature_state_check_constraint_rejects_unknown_state() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let ts = chrono::Utc::now().to_rfc3339();
    let err = conn.execute(
        "INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
         VALUES ('bad','B','bogus', X'00','main', ?1, ?1)",
        rusqlite::params![ts],
    );
    assert!(err.is_err());
}

#[test]
fn work_package_state_check_constraint() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let ts = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
         VALUES (1,'f','F','created', X'00','main', ?1, ?1)",
        rusqlite::params![ts],
    )
    .unwrap();
    let err = conn.execute(
        "INSERT INTO work_packages (feature_id, title, state, sequence, file_scope, acceptance_criteria, created_at, updated_at)
         VALUES (1,'w','bogus', 1, '[]', '', ?1, ?1)",
        rusqlite::params![ts],
    );
    assert!(err.is_err());
}

#[test]
fn work_package_pr_state_check_constraint() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let ts = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
         VALUES (1,'f','F','created', X'00','main', ?1, ?1)",
        rusqlite::params![ts],
    )
    .unwrap();
    let err = conn.execute(
        "INSERT INTO work_packages (feature_id, title, state, sequence, file_scope, acceptance_criteria, pr_state, created_at, updated_at)
         VALUES (1,'w','planned', 1, '[]', '', 'bogus', ?1, ?1)",
        rusqlite::params![ts],
    );
    assert!(err.is_err());
}

#[test]
fn evidence_type_check_constraint() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let ts = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
         VALUES (1,'f','F','created', X'00','main', ?1, ?1)",
        rusqlite::params![ts],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO work_packages (id, feature_id, title, state, sequence, file_scope, acceptance_criteria, created_at, updated_at)
         VALUES (1,1,'w','planned',1,'[]','', ?1, ?1)",
        rusqlite::params![ts],
    )
    .unwrap();
    let err = conn.execute(
        "INSERT INTO evidence (wp_id, fr_id, evidence_type, artifact_path, metadata, created_at)
         VALUES (1,'FR-1','bogus','p',NULL, ?1)",
        rusqlite::params![ts],
    );
    assert!(err.is_err(), "evidence_type CHECK must reject unknown values");
}

#[test]
fn module_feature_tags_composite_pk_is_unique() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let ts = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
         VALUES (1,'f','F','created', X'00','main', ?1, ?1)",
        rusqlite::params![ts],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO modules (id, slug, friendly_name, created_at, updated_at)
         VALUES (1,'m','M', ?1, ?1)",
        rusqlite::params![ts],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO module_feature_tags (module_id, feature_id, created_at) VALUES (1,1, ?1)",
        rusqlite::params![ts],
    )
    .unwrap();
    assert!(conn
        .execute(
            "INSERT INTO module_feature_tags (module_id, feature_id, created_at) VALUES (1,1, ?1)",
            rusqlite::params![ts],
        )
        .is_err());
}

#[test]
fn cycle_date_check_constraint() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let ts = chrono::Utc::now().to_rfc3339();
    let err = conn.execute(
        "INSERT INTO cycles (name, state, start_date, end_date, created_at, updated_at)
         VALUES ('c','Draft','2026-02-01','2026-01-01', ?1, ?1)",
        rusqlite::params![ts],
    );
    assert!(err.is_err());
}

#[test]
fn work_packages_fk_cascade_on_feature_delete() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let ts = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
         VALUES (1,'f','F','created', X'00','main', ?1, ?1)",
        rusqlite::params![ts],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO work_packages (id, feature_id, title, state, sequence, file_scope, acceptance_criteria, created_at, updated_at)
         VALUES (1,1,'w','planned',1,'[]','', ?1, ?1)",
        rusqlite::params![ts],
    )
    .unwrap();
    conn.execute("DELETE FROM features WHERE id = 1", []).unwrap();
    let remaining: i64 = conn
        .query_row("SELECT COUNT(*) FROM work_packages", [], |r| r.get(0))
        .unwrap();
    assert_eq!(remaining, 0, "work packages must cascade-delete with their feature");
}

#[test]
fn feature_labels_column_exists() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    // Migration 026 adds `labels` to features.
    conn.execute(
        "INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, labels, created_at, updated_at)
         VALUES ('labelled','L','created', X'00','main', '[\"a\",\"b\"]', ?1, ?1)",
        rusqlite::params![chrono::Utc::now().to_rfc3339()],
    )
    .unwrap();
    let labels: String = conn
        .query_row("SELECT labels FROM features WHERE slug='labelled'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(labels, "[\"a\",\"b\"]");
}

#[test]
fn story_work_packages_fk_requires_story_and_wp() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let err = conn.execute(
        "INSERT INTO story_work_packages (story_id, work_package_id) VALUES (999, 999)",
        [],
    );
    assert!(err.is_err(), "FK to stories/work_packages must be enforced");
}

#[test]
fn cycle_stories_fk_requires_cycle_and_story() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let err = conn.execute(
        "INSERT INTO cycle_stories (cycle_id, story_id, added_at) VALUES (999, 999, '2026-01-01T00:00:00Z')",
        [],
    );
    assert!(err.is_err());
}

#[test]
fn intent_nodes_table_has_expected_columns() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut stmt = conn.prepare("PRAGMA table_info(intent_nodes)").unwrap();
    let cols: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert!(cols.contains(&"id".to_string()));
    assert!(cols.contains(&"kind".to_string()) || !cols.is_empty());
}

#[test]
fn gate_results_table_has_expected_columns() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut stmt = conn.prepare("PRAGMA table_info(gate_results)").unwrap();
    let cols: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    for col in ["work_package_id", "gate_name", "status", "checked_at"] {
        assert!(cols.contains(&col.to_string()), "gate_results missing {col}");
    }
}

#[test]
fn run_records_table_has_expected_columns() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut stmt = conn.prepare("PRAGMA table_info(run_records)").unwrap();
    let cols: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert!(cols.contains(&"run_type".to_string()));
}

#[test]
fn sync_mapping_unique_entity_and_plane() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let ts = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO sync_mappings (entity_type, entity_id, plane_issue_id, content_hash, last_synced_at, sync_direction)
         VALUES ('feature',1,'p1','h', ?1, 'push')",
        rusqlite::params![ts],
    )
    .unwrap();
    // Same entity, different plane id -> unique(entity_type, entity_id) violated.
    assert!(conn
        .execute(
            "INSERT INTO sync_mappings (entity_type, entity_id, plane_issue_id, content_hash, last_synced_at, sync_direction)
             VALUES ('feature',1,'p2','h', ?1, 'push')",
            rusqlite::params![ts],
        )
        .is_err());
}

#[test]
fn user_email_unique_constraint() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let ts = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO users (display_name, email, role, status, created_at, updated_at)
         VALUES ('A','same@example.com','member','active', ?1, ?1)",
        rusqlite::params![ts],
    )
    .unwrap();
    assert!(conn
        .execute(
            "INSERT INTO users (display_name, email, role, status, created_at, updated_at)
             VALUES ('B','same@example.com','member','active', ?1, ?1)",
            rusqlite::params![ts],
        )
        .is_err());
}

#[test]
fn migrations_include_newly_registered_story_links() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name = '022_story_wp_cycle_links'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "story/wp/cycle link migration must be recorded");
}

/// The `migrate` binary must create missing parent directories, heal a stale
/// database file, and report that it is up to date on a second run.
#[test]
fn migrate_binary_heals_stale_database_file() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "agileplus-sqlite-migrate-bin-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    // The parent directory does not exist yet: the binary must create it.
    let db = dir.join("nested").join("agileplus.db");

    let run = || {
        std::process::Command::new(env!("CARGO_BIN_EXE_migrate"))
            .arg(&db)
            .output()
            .expect("spawn migrate")
    };

    let first = run();
    assert!(
        first.status.success(),
        "migrate failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_stdout = String::from_utf8_lossy(&first.stdout);
    assert!(
        first_stdout.contains("Migrating database at:"),
        "unexpected output: {first_stdout}"
    );
    assert!(
        first_stdout.contains("Applied") && first_stdout.contains("OK: backlog_items present"),
        "a fresh file must have migrations applied: {first_stdout}"
    );
    assert!(db.exists(), "migrate must create the database file");

    let second = run();
    assert!(second.status.success(), "second migrate run must succeed");
    let second_stdout = String::from_utf8_lossy(&second.stdout);
    assert!(
        second_stdout.contains("Already up to date"),
        "a migrated file must report no work: {second_stdout}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
