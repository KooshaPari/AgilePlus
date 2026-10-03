//! Retained migration oracles, moved out of mod.rs as the registry grows.
use super::*;

#[test]
fn migrations_include_016_create_backlog_items() {
    assert!(
        MIGRATIONS
            .iter()
            .any(|(name, _)| *name == "016_create_backlog_items")
    );
}
#[test]
fn run_all_creates_backlog_items_table() {
    let conn = Connection::open_in_memory().unwrap();
    MigrationRunner::new(&conn).run_all().unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='backlog_items'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}
#[test]
fn parse_up_extracts_body_between_markers() {
    assert_eq!(
        parse_up("-- UP\nCREATE TABLE t (id INTEGER);\n\n-- DOWN\nDROP TABLE t;\n"),
        "CREATE TABLE t (id INTEGER);"
    );
}
#[test]
fn parse_up_without_marker_returns_whole_text() {
    assert_eq!(parse_up("SELECT 1;"), "SELECT 1;");
}
#[test]
fn parse_up_requires_uppercase_marker() {
    assert_eq!(parse_up("-- up\nSELECT 1;"), "-- up\nSELECT 1;");
}
#[test]
fn parse_up_marker_accepts_colon_suffix() {
    assert_eq!(
        parse_up("-- UP:\nSELECT 1;\n-- DOWN\nSELECT 2;"),
        "SELECT 1;"
    );
}
#[test]
fn parse_up_body_runs_to_end_when_no_down_marker() {
    assert_eq!(
        parse_up("-- UP\nSELECT 1;\nSELECT 2;"),
        "SELECT 1;\nSELECT 2;"
    );
}
#[test]
fn parse_down_extracts_body_after_marker() {
    assert_eq!(
        parse_down("-- UP\nSELECT 1;\n-- DOWN\nDROP TABLE t;\n"),
        "DROP TABLE t;"
    );
}
#[test]
fn parse_down_without_marker_is_empty() {
    assert_eq!(parse_down("SELECT 1;"), "");
}
#[test]
fn every_registered_migration_has_a_nonempty_up_body() {
    for (name, sql) in MIGRATIONS {
        let up = parse_up(sql);
        assert!(!up.is_empty(), "{name}");
        assert!(!up.contains("-- DOWN"), "{name}");
    }
}
#[test]
fn rollback_last_deletes_tracking_row_for_unregistered_migration() {
    let conn = Connection::open_in_memory().unwrap();
    let runner = MigrationRunner::new(&conn);
    runner.run_all().unwrap();
    conn.execute("INSERT INTO _migrations(name,applied_at) VALUES ('999_not_registered','2026-01-01T00:00:00Z')", []).unwrap();
    runner.rollback_last().unwrap();
    let remaining: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name='999_not_registered'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(remaining, 0);
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM _migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(total as usize, MIGRATIONS.len());
}
#[test]
fn run_all_reports_failing_migration_and_leaves_it_unapplied() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE features (bogus TEXT);")
        .unwrap();
    let err = MigrationRunner::new(&conn).run_all().unwrap_err();
    let DomainError::Storage(message) = err else {
        panic!("expected storage error");
    };
    assert!(
        message.contains("migration 009_create_indexes failed"),
        "{message}"
    );
    let earlier: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name='001_create_features'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(earlier, 1);
    let failed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name='009_create_indexes'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(failed, 0);
}
#[test]
fn run_all_heals_a_database_missing_a_later_migration() {
    let conn = Connection::open_in_memory().unwrap();
    let runner = MigrationRunner::new(&conn);
    runner.run_all().unwrap();
    conn.execute_batch("DROP TABLE backlog_items;").unwrap();
    conn.execute(
        "DELETE FROM _migrations WHERE name='016_create_backlog_items'",
        [],
    )
    .unwrap();
    runner.run_all().unwrap();
    let tables: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='backlog_items'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(tables, 1);
    let recorded: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name='016_create_backlog_items'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(recorded, 1);
}
#[test]
fn rollback_of_026_drops_tracking_row_but_keeps_add_column_changes() {
    let conn = Connection::open_in_memory().unwrap();
    let runner = MigrationRunner::new(&conn);
    runner.run_all().unwrap();
    let index = MIGRATIONS
        .iter()
        .position(|(name, _)| *name == "026_feature_labels")
        .unwrap();
    for (name, _) in MIGRATIONS[index + 1..].iter().rev() {
        runner.rollback_last().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE name=?1",
                [name],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "{name}");
    }
    let (name_026, sql_026) = MIGRATIONS[index];
    runner.rollback_last().unwrap();
    let applied: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name=?1",
            [name_026],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(applied, 0);
    assert!(
        parse_down(sql_026)
            .lines()
            .all(|line| line.trim().is_empty() || line.trim_start().starts_with("--"))
    );
    let has_labels: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('features') WHERE name='labels'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(has_labels, 1, "irreversible 026 column must remain");
}
#[test]
fn run_all_marks_all_migrations_applied_when_schema_is_clean() {
    let conn = Connection::open_in_memory().unwrap();
    MigrationRunner::new(&conn).run_all().unwrap();
    let applied: i64 = conn
        .query_row("SELECT COUNT(*) FROM _migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(applied as usize, MIGRATIONS.len());
    let names: Vec<String> = conn
        .prepare("SELECT name,applied_at FROM _migrations ORDER BY id")
        .unwrap()
        .query_map([], |r| {
            let name: String = r.get(0)?;
            let applied: String = r.get(1)?;
            assert!(applied.parse::<chrono::DateTime<chrono::Utc>>().is_ok());
            Ok(name)
        })
        .unwrap()
        .collect::<SqlResult<Vec<_>>>()
        .unwrap();
    assert_eq!(names[0], MIGRATIONS[0].0);
    assert_eq!(names[names.len() - 1], MIGRATIONS[MIGRATIONS.len() - 1].0);
}
