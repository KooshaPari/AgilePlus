//! Metrics repository — CRUD for the `metrics` table.

use rusqlite::{Connection, params};

use agileplus_domain::{domain::metric::Metric, error::DomainError};

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

pub fn record_metric(conn: &Connection, metric: &Metric) -> Result<i64, DomainError> {
    let metadata_s = metric
        .metadata
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|e| DomainError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO metrics (feature_id, command, duration_ms, agent_runs, review_cycles, metadata, timestamp)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            metric.feature_id,
            metric.command,
            metric.duration_ms,
            metric.agent_runs,
            metric.review_cycles,
            metadata_s,
            metric.timestamp.to_rfc3339(),
        ],
    )
    .map_err(map_err)?;

    Ok(conn.last_insert_rowid())
}

pub fn get_metrics_by_feature(
    conn: &Connection,
    feature_id: i64,
) -> Result<Vec<Metric>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id,feature_id,command,duration_ms,agent_runs,review_cycles,metadata,timestamp
             FROM metrics WHERE feature_id = ?1 ORDER BY timestamp",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![feature_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, String>(7)?,
            ))
        })
        .map_err(map_err)?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?
        .into_iter()
        .map(
            |(
                id,
                feature_id,
                command,
                duration_ms,
                agent_runs,
                review_cycles,
                metadata_s,
                timestamp_s,
            )| {
                let metadata = metadata_s
                    .map(|s| serde_json::from_str(&s))
                    .transpose()
                    .map_err(|e: serde_json::Error| DomainError::Storage(e.to_string()))?;
                let timestamp = timestamp_s
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                Ok(Metric {
                    id,
                    feature_id,
                    command,
                    duration_ms,
                    agent_runs,
                    review_cycles,
                    metadata,
                    timestamp,
                })
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;
    use chrono::Utc;
    use rusqlite::params;

    fn seed_feature(conn: &Connection, id: i64) {
        conn.execute(
            "INSERT OR IGNORE INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at) \
             VALUES (?1, ?2, ?3, 'created', X'00', 'main', datetime('now'), datetime('now'))",
            params![id, format!("feat-{id}"), format!("Feature {id}")],
        )
        .unwrap();
    }

    fn make_metric(command: &str, feature_id: Option<i64>) -> Metric {
        Metric {
            id: 0,
            feature_id,
            command: command.to_string(),
            duration_ms: 1500,
            agent_runs: 3,
            review_cycles: 1,
            metadata: Some(serde_json::json!({"key": "value"})),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_record_and_get_metric() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 42);
        let m = make_metric("cargo test", Some(42));
        let id = record_metric(&conn, &m).unwrap();
        assert!(id > 0);
        let metrics = get_metrics_by_feature(&conn, 42).unwrap();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].command, "cargo test");
        assert_eq!(metrics[0].feature_id, Some(42));
    }

    #[test]
    fn test_get_metrics_empty_when_none() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let metrics = get_metrics_by_feature(&conn, 999).unwrap();
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_multiple_metrics_ordered_by_timestamp() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let mut m1 = make_metric("cmd1", Some(1));
        m1.timestamp = Utc::now();
        record_metric(&conn, &m1).unwrap();
        let mut m2 = make_metric("cmd2", Some(1));
        m2.timestamp = Utc::now();
        record_metric(&conn, &m2).unwrap();
        let metrics = get_metrics_by_feature(&conn, 1).unwrap();
        assert_eq!(metrics.len(), 2);
        assert!(metrics[0].timestamp <= metrics[1].timestamp);
    }

    #[test]
    fn test_metric_with_null_metadata() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 5);
        let mut m = make_metric("no-meta", Some(5));
        m.metadata = None;
        let id = record_metric(&conn, &m).unwrap();
        assert!(id > 0);
        let metrics = get_metrics_by_feature(&conn, 5).unwrap();
        assert!(metrics[0].metadata.is_none());
    }

    #[test]
    fn test_metric_filters_by_feature_id() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        seed_feature(&conn, 2);
        record_metric(&conn, &make_metric("a", Some(1))).unwrap();
        record_metric(&conn, &make_metric("b", Some(2))).unwrap();
        record_metric(&conn, &make_metric("c", Some(1))).unwrap();
        let f1 = get_metrics_by_feature(&conn, 1).unwrap();
        assert_eq!(f1.len(), 2);
        let f2 = get_metrics_by_feature(&conn, 2).unwrap();
        assert_eq!(f2.len(), 1);
    }
}
