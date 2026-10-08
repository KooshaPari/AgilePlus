// SPDX-License-Identifier: MIT OR Apache-2.0
//! Ordered embedded schema migrations; existing migration bodies remain unchanged.

use agileplus_domain::error::DomainError;
use rusqlite::{Connection, Result as SqlResult};

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_create_features",
        include_str!("001_create_features.sql"),
    ),
    (
        "002_create_work_packages",
        include_str!("002_create_work_packages.sql"),
    ),
    (
        "003_create_governance_contracts",
        include_str!("003_create_governance_contracts.sql"),
    ),
    (
        "004_create_audit_log",
        include_str!("004_create_audit_log.sql"),
    ),
    (
        "005_create_evidence",
        include_str!("005_create_evidence.sql"),
    ),
    (
        "006_create_policy_rules",
        include_str!("006_create_policy_rules.sql"),
    ),
    ("007_create_metrics", include_str!("007_create_metrics.sql")),
    (
        "008_create_wp_dependencies",
        include_str!("008_create_wp_dependencies.sql"),
    ),
    ("009_create_indexes", include_str!("009_create_indexes.sql")),
    ("010_create_events", include_str!("010_create_events.sql")),
    (
        "011_create_snapshots",
        include_str!("011_create_snapshots.sql"),
    ),
    (
        "012_create_sync_mappings",
        include_str!("012_create_sync_mappings.sql"),
    ),
    (
        "013_create_api_keys",
        include_str!("013_create_api_keys.sql"),
    ),
    (
        "014_create_device_nodes",
        include_str!("014_create_device_nodes.sql"),
    ),
    ("015_modules_cycles", include_str!("015_modules_cycles.sql")),
    (
        "016_create_backlog_items",
        include_str!("016_create_backlog_items.sql"),
    ),
    (
        "017_create_projects",
        include_str!("017_create_projects.sql"),
    ),
    ("018_create_users", include_str!("018_create_users.sql")),
    ("019_create_epics", include_str!("019_create_epics.sql")),
    ("020_create_stories", include_str!("020_create_stories.sql")),
    (
        "021_add_requirement_id",
        include_str!("021_add_requirement_id.sql"),
    ),
    (
        "022_create_trace_links",
        include_str!("022_create_trace_links.sql"),
    ),
    (
        "022_story_wp_cycle_links",
        include_str!("022_story_wp_cycle_links.sql"),
    ),
    (
        "023_create_worklog_entries",
        include_str!("023_create_worklog_entries.sql"),
    ),
    (
        "024_l2_38_worklog_trace_gate_run_scope",
        include_str!("024_l2_38_worklog_trace_gate_run_scope.sql"),
    ),
    (
        "025_create_intent_graph",
        include_str!("025_create_intent_graph.sql"),
    ),
    (
        "025_governance_channel_iteration",
        include_str!("025_governance_channel_iteration.sql"),
    ),
    (
        "025_intent_graph_views",
        include_str!("025_intent_graph_views.sql"),
    ),
    ("026_feature_labels", include_str!("026_feature_labels.sql")),
    (
        "027_execution_records",
        include_str!("027_execution_records.sql"),
    ),
    (
        "028_one_active_assignment",
        include_str!("028_one_active_assignment.sql"),
    ),
    (
        "029_assignment_criteria",
        include_str!("029_assignment_criteria.sql"),
    ),
    (
        "030_atomic_acceptance",
        include_str!("030_atomic_acceptance.sql"),
    ),
    ("031_promotion_saga", include_str!("031_promotion_saga.sql")),
];

fn find_up_body_start(sql: &str) -> Option<usize> {
    let bytes = sql.as_bytes();
    let mut i = 0;
    while i + 5 <= bytes.len() {
        if &bytes[i..i + 5] == b"-- UP"
            && (i + 5 == bytes.len() || !bytes[i + 5].is_ascii_lowercase())
        {
            let mut j = i + 5;
            if j < bytes.len() && bytes[j] == b':' {
                j += 1;
            }
            while j < bytes.len() && bytes[j] != b'\n' {
                j += 1;
            }
            if j < bytes.len() {
                j += 1;
            }
            return Some(j);
        }
        i += 1;
    }
    None
}

fn parse_up(sql: &str) -> &str {
    if let Some(up_start) = find_up_body_start(sql) {
        if let Some(down_start) = sql[up_start..].find("-- DOWN") {
            return sql[up_start..up_start + down_start].trim();
        }
        return sql[up_start..].trim();
    }
    sql.trim()
}

fn parse_down(sql: &str) -> &str {
    if let Some(down_start) = sql.find("-- DOWN") {
        return sql[down_start + 7..].trim();
    }
    ""
}

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

pub struct MigrationRunner<'a> {
    conn: &'a Connection,
}

impl<'a> MigrationRunner<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn ensure_meta_table(&self) -> Result<(), DomainError> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS _migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                applied_at TEXT NOT NULL
            );",
            )
            .map_err(map_err)
    }

    fn is_applied(&self, name: &str) -> SqlResult<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn run_all(&self) -> Result<(), DomainError> {
        self.ensure_meta_table()?;
        for (name, sql) in MIGRATIONS {
            if self.is_applied(name).map_err(map_err)? {
                continue;
            }
            self.conn
                .execute_batch(parse_up(sql))
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
            return Ok(());
        };
        if let Some((_, sql)) = MIGRATIONS.iter().find(|(n, _)| *n == name.as_str()) {
            let down = parse_down(sql);
            if !down.is_empty() {
                self.conn
                    .execute_batch(down)
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
mod tests;
