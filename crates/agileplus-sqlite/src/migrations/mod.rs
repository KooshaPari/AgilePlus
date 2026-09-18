// SPDX-License-Identifier: MIT OR Apache-2.0
//! Migration system for agileplus-sqlite.
//!
//! Migrations are embedded as SQL files and applied in order on startup.
//! Applied migrations are tracked in the `_migrations` meta table.

use rusqlite::{Connection, Result as SqlResult};

use agileplus_domain::error::DomainError;

// Embedded SQL migrations
const MIGRATION_001: &str = include_str!("001_create_features.sql");
const MIGRATION_002: &str = include_str!("002_create_work_packages.sql");
const MIGRATION_003: &str = include_str!("003_create_governance_contracts.sql");
const MIGRATION_004: &str = include_str!("004_create_audit_log.sql");
const MIGRATION_005: &str = include_str!("005_create_evidence.sql");
const MIGRATION_006: &str = include_str!("006_create_policy_rules.sql");
const MIGRATION_007: &str = include_str!("007_create_metrics.sql");
const MIGRATION_008: &str = include_str!("008_create_wp_dependencies.sql");
const MIGRATION_009: &str = include_str!("009_create_indexes.sql");
const MIGRATION_010: &str = include_str!("010_create_events.sql");
const MIGRATION_011: &str = include_str!("011_create_snapshots.sql");
const MIGRATION_012: &str = include_str!("012_create_sync_mappings.sql");
const MIGRATION_013: &str = include_str!("013_create_api_keys.sql");
const MIGRATION_014: &str = include_str!("014_create_device_nodes.sql");
const MIGRATION_015: &str = include_str!("015_modules_cycles.sql");
const MIGRATION_016: &str = include_str!("016_create_backlog_items.sql");
const MIGRATION_017: &str = include_str!("017_create_projects.sql");
const MIGRATION_018: &str = include_str!("018_create_users.sql");
const MIGRATION_019: &str = include_str!("019_create_epics.sql");
const MIGRATION_020: &str = include_str!("020_create_stories.sql");
const MIGRATION_021: &str = include_str!("021_add_requirement_id.sql");
const MIGRATION_022: &str = include_str!("022_create_trace_links.sql");
const MIGRATION_022_LINKS: &str = include_str!("022_story_wp_cycle_links.sql");
const MIGRATION_023: &str = include_str!("023_create_worklog_entries.sql");
const MIGRATION_024: &str = include_str!("024_l2_38_worklog_trace_gate_run_scope.sql");
const MIGRATION_025: &str = include_str!("025_create_intent_graph.sql");
const MIGRATION_025_GOV: &str = include_str!("025_governance_channel_iteration.sql");
const MIGRATION_025_VIEWS: &str = include_str!("025_intent_graph_views.sql");
const MIGRATION_026: &str = include_str!("026_feature_labels.sql");

/// All migrations in order: (name, up_sql, down_sql)
const MIGRATIONS: &[(&str, &str)] = &[
    ("001_create_features", MIGRATION_001),
    ("002_create_work_packages", MIGRATION_002),
    ("003_create_governance_contracts", MIGRATION_003),
    ("004_create_audit_log", MIGRATION_004),
    ("005_create_evidence", MIGRATION_005),
    ("006_create_policy_rules", MIGRATION_006),
    ("007_create_metrics", MIGRATION_007),
    ("008_create_wp_dependencies", MIGRATION_008),
    ("009_create_indexes", MIGRATION_009),
    ("010_create_events", MIGRATION_010),
    ("011_create_snapshots", MIGRATION_011),
    ("012_create_sync_mappings", MIGRATION_012),
    ("013_create_api_keys", MIGRATION_013),
    ("014_create_device_nodes", MIGRATION_014),
    ("015_modules_cycles", MIGRATION_015),
    ("016_create_backlog_items", MIGRATION_016),
    ("017_create_projects", MIGRATION_017),
    ("018_create_users", MIGRATION_018),
    ("019_create_epics", MIGRATION_019),
    ("020_create_stories", MIGRATION_020),
    ("021_add_requirement_id", MIGRATION_021),
    ("022_create_trace_links", MIGRATION_022),
    ("022_story_wp_cycle_links", MIGRATION_022_LINKS),
    ("023_create_worklog_entries", MIGRATION_023),
    ("024_l2_38_worklog_trace_gate_run_scope", MIGRATION_024),
    ("025_create_intent_graph", MIGRATION_025),
    ("025_governance_channel_iteration", MIGRATION_025_GOV),
    ("025_intent_graph_views", MIGRATION_025_VIEWS),
    ("026_feature_labels", MIGRATION_026),
];

/// Find the byte offset where the UP body starts, given a `-- UP` marker
/// (the `UP` token may be followed by `:`, whitespace, or anything). Returns
/// `None` if no UP marker is present.
fn find_up_body_start(sql: &str) -> Option<usize> {
    // Look for `-- UP` not followed by a lowercase letter (avoids matching
    // `-- UPGRADE` etc); allows `-- UP`, `-- UP:`, `-- UP --` ...
    let bytes = sql.as_bytes();
    let mut i = 0;
    while i + 5 <= bytes.len() {
        if &bytes[i..i + 5] == b"-- UP"
            && (i + 5 == bytes.len() || !bytes[i + 5].is_ascii_lowercase())
        {
            // Skip the marker + any trailing `:`, whitespace, or `--` (line comment).
            let mut j = i + 5;
            // Single trailing ':'
            if j < bytes.len() && bytes[j] == b':' {
                j += 1;
            }
            // Skip rest of the line (the marker comment)
            while j < bytes.len() && bytes[j] != b'\n' {
                j += 1;
            }
            // Skip the newline
            if j < bytes.len() {
                j += 1;
            }
            return Some(j);
        }
        i += 1;
    }
    None
}

/// Parse the UP section from a migration SQL file.
fn parse_up(sql: &str) -> &str {
    // Format is:
    //   -- UP [-- up to text]
    //   <sql>
    //   -- DOWN
    //   <sql>
    if let Some(up_start) = find_up_body_start(sql) {
        if let Some(down_start) = sql[up_start..].find("-- DOWN") {
            return sql[up_start..up_start + down_start].trim();
        }
        return sql[up_start..].trim();
    }
    sql.trim()
}

/// Parse the DOWN section from a migration SQL file.
fn parse_down(sql: &str) -> &str {
    if let Some(down_start) = sql.find("-- DOWN") {
        return sql[down_start + 7..].trim();
    }
    ""
}

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

/// Runs database schema migrations.
pub struct MigrationRunner<'a> {
    conn: &'a Connection,
}

impl<'a> MigrationRunner<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Create the migrations tracking table if it doesn't exist.
    fn ensure_meta_table(&self) -> Result<(), DomainError> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS _migrations (
                    id         INTEGER PRIMARY KEY AUTOINCREMENT,
                    name       TEXT    UNIQUE NOT NULL,
                    applied_at TEXT    NOT NULL
                );",
            )
            .map_err(map_err)
    }

    /// Check whether a migration has already been applied.
    fn is_applied(&self, name: &str) -> SqlResult<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Apply all pending migrations in order.
    pub fn run_all(&self) -> Result<(), DomainError> {
        self.ensure_meta_table()?;

        for (name, sql) in MIGRATIONS {
            if self.is_applied(name).map_err(map_err)? {
                continue;
            }

            let up_sql = parse_up(sql);
            self.conn
                .execute_batch(up_sql)
                .map_err(|e| DomainError::Storage(format!("migration {name} failed: {e}")))?;

            let now = chrono::Utc::now().to_rfc3339();
            self.conn
                .execute(
                    "INSERT INTO _migrations (name, applied_at) VALUES (?1, ?2)",
                    rusqlite::params![name, now],
                )
                .map_err(map_err)?;
        }

        Ok(())
    }

    /// Roll back the most recently applied migration.
    pub fn rollback_last(&self) -> Result<(), DomainError> {
        self.ensure_meta_table()?;

        let last_name: Option<String> = self
            .conn
            .query_row(
                "SELECT name FROM _migrations ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(map_err)?;

        let Some(name) = last_name else {
            return Ok(()); // Nothing to roll back
        };

        // Find the migration SQL
        let migration = MIGRATIONS.iter().find(|(n, _)| *n == name.as_str());
        if let Some((_, sql)) = migration {
            let down_sql = parse_down(sql);
            if !down_sql.is_empty() {
                self.conn
                    .execute_batch(down_sql)
                    .map_err(|e| DomainError::Storage(format!("rollback of {name} failed: {e}")))?;
            }
        }

        self.conn
            .execute(
                "DELETE FROM _migrations WHERE name = ?1",
                rusqlite::params![name],
            )
            .map_err(map_err)?;

        Ok(())
    }
}

/// Extension trait to add `.optional()` on rusqlite query results.
trait OptionalExt<T> {
    fn optional(self) -> SqlResult<Option<T>>;
}

impl<T> OptionalExt<T> for SqlResult<T> {
    fn optional(self) -> SqlResult<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_include_016_create_backlog_items() {
        assert!(
            MIGRATIONS
                .iter()
                .any(|(name, _)| *name == "016_create_backlog_items"),
            "016_create_backlog_items must stay registered so default DBs heal on open"
        );
    }

    #[test]
    fn run_all_creates_backlog_items_table() {
        let conn = Connection::open_in_memory().expect("in-memory");
        MigrationRunner::new(&conn).run_all().expect("migrate");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'backlog_items'",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert_eq!(count, 1, "backlog_items must exist after run_all");
    }

    #[test]
    fn parse_up_extracts_body_between_markers() {
        let sql = "-- UP\nCREATE TABLE t (id INTEGER);\n\n-- DOWN\nDROP TABLE t;\n";
        assert_eq!(parse_up(sql), "CREATE TABLE t (id INTEGER);");
    }

    #[test]
    fn parse_up_without_marker_returns_whole_text() {
        assert_eq!(parse_up("SELECT 1;"), "SELECT 1;");
    }

    #[test]
    fn parse_up_requires_uppercase_marker() {
        // A lowercase `-- up` is not recognized, so the entire text is the body.
        assert_eq!(parse_up("-- up\nSELECT 1;"), "-- up\nSELECT 1;");
    }

    #[test]
    fn parse_up_marker_accepts_colon_suffix() {
        let sql = "-- UP:\nSELECT 1;\n-- DOWN\nSELECT 2;";
        assert_eq!(parse_up(sql), "SELECT 1;");
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
            assert!(
                !up.is_empty(),
                "migration {name} has an empty UP body, so run_all would be a silent no-op"
            );
            assert!(
                !up.contains("-- DOWN"),
                "migration {name} leaked the DOWN section into its UP body"
            );
        }
    }

    #[test]
    fn rollback_last_deletes_tracking_row_for_unregistered_migration() {
        let conn = Connection::open_in_memory().expect("in-memory");
        let runner = MigrationRunner::new(&conn);
        runner.run_all().expect("migrate");
        // A tracking row whose name is not in MIGRATIONS: rollback must still
        // drop the row (there is simply no DOWN body to execute).
        conn.execute(
            "INSERT INTO _migrations (name, applied_at) VALUES ('999_not_registered', '2026-01-01T00:00:00Z')",
            [],
        )
        .expect("insert fake row");

        runner.rollback_last().expect("rollback");

        let remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE name = '999_not_registered'",
                [],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(remaining, 0, "unknown migration row must be removed");
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
            .expect("count all");
        assert_eq!(total as usize, MIGRATIONS.len());
    }

    #[test]
    fn run_all_reports_failing_migration_and_leaves_it_unapplied() {
        let conn = Connection::open_in_memory().expect("in-memory");
        // Pre-create a conflicting `features` table so `CREATE TABLE IF NOT
        // EXISTS features` is a no-op and the later index migration fails.
        conn.execute_batch("CREATE TABLE features (bogus TEXT);")
            .expect("conflicting table");

        let err = MigrationRunner::new(&conn)
            .run_all()
            .expect_err("migration must fail");
        let DomainError::Storage(message) = err else {
            panic!("expected DomainError::Storage, got {err:?}");
        };
        assert!(
            message.contains("migration 009_create_indexes failed"),
            "error must name the failing migration: {message}"
        );

        let earlier: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE name = '001_create_features'",
                [],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(earlier, 1, "migrations before the failure stay recorded");

        let failed: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE name = '009_create_indexes'",
                [],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(
            failed, 0,
            "a failed migration must not be recorded as applied"
        );
    }

    #[test]
    fn run_all_heals_a_database_missing_a_later_migration() {
        let conn = Connection::open_in_memory().expect("in-memory");
        let runner = MigrationRunner::new(&conn);
        runner.run_all().expect("migrate");

        // Simulate a database created by an older CLI that skipped 016.
        conn.execute_batch("DROP TABLE backlog_items;")
            .expect("drop backlog table");
        conn.execute(
            "DELETE FROM _migrations WHERE name = '016_create_backlog_items'",
            [],
        )
        .expect("forget 016");

        runner.run_all().expect("re-run must heal the schema");

        let tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'backlog_items'",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert_eq!(tables, 1, "backlog_items must be recreated");
        let recorded: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE name = '016_create_backlog_items'",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert_eq!(recorded, 1, "016 must be recorded again");
    }

    #[test]
    fn rollback_last_drops_tracking_row_but_keeps_add_column_changes() {
        let conn = Connection::open_in_memory().expect("in-memory");
        let runner = MigrationRunner::new(&conn);
        runner.run_all().expect("migrate");
        let (last_name, last_sql) = MIGRATIONS[MIGRATIONS.len() - 1];
        assert_eq!(last_name, "026_feature_labels");

        runner.rollback_last().expect("rollback");
        let applied: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
                rusqlite::params![last_name],
                |row| row.get(0),
            )
            .expect("query");
        assert_eq!(applied, 0, "{last_name} must no longer be recorded");

        // 026 only appends a column and its DOWN body is comment-only, so the
        // column survives the rollback (see the comment inside the migration file).
        let down = parse_down(last_sql);
        assert!(
            down.lines()
                .all(|line| line.trim().is_empty() || line.trim_start().starts_with("--")),
            "this test documents a DOWN body that only contains comments: {down:?}"
        );
        let has_labels: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('features') WHERE name = 'labels'",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert_eq!(
            has_labels, 1,
            "the appended column remains after an irreversible rollback"
        );
    }

    #[test]
    fn run_all_marks_all_migrations_applied_when_schema_is_clean() {
        let conn = Connection::open_in_memory().expect("in-memory");
        MigrationRunner::new(&conn).run_all().expect("migrate");
        let applied: i64 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
            .expect("count");
        assert_eq!(applied as usize, MIGRATIONS.len());
        let names: Vec<String> = conn
            .prepare("SELECT name, applied_at FROM _migrations ORDER BY id")
            .expect("prepare")
            .query_map([], |row| {
                let name: String = row.get(0)?;
                let applied_at: String = row.get(1)?;
                assert!(
                    applied_at.parse::<chrono::DateTime<chrono::Utc>>().is_ok(),
                    "applied_at for {name} must be RFC3339"
                );
                Ok(name)
            })
            .expect("query")
            .collect::<SqlResult<Vec<_>>>()
            .expect("collect");
        assert_eq!(names[0], MIGRATIONS[0].0);
        assert_eq!(names[names.len() - 1], MIGRATIONS[MIGRATIONS.len() - 1].0);
    }
}
