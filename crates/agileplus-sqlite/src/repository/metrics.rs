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

    fn sample_metric(feature_id: Option<i64>, command: &str) -> Metric {
        Metric {
            id: 0,
            feature_id,
            command: command.to_string(),
            duration_ms: 1500,
            agent_runs: 3,
            review_cycles: 1,
            metadata: None,
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn record_and_get_metrics_by_feature() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        record_metric(&conn, &sample_metric(Some(1), "cargo test")).unwrap();
        record_metric(&conn, &sample_metric(Some(1), "cargo build")).unwrap();
        record_metric(&conn, &sample_metric(Some(2), "cargo test")).unwrap();
        let m1 = get_metrics_by_feature(&conn, 1).unwrap();
        assert_eq!(m1.len(), 2);
        let m2 = get_metrics_by_feature(&conn, 2).unwrap();
        assert_eq!(m2.len(), 1);
    }

    #[test]
    fn get_metrics_empty_for_unknown_feature() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(get_metrics_by_feature(&conn, 999).unwrap().is_empty());
    }

    #[test]
    fn record_metric_with_metadata() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let mut m = sample_metric(Some(1), "deploy");
        m.metadata = Some(serde_json::json!({"env": "prod"}));
        record_metric(&conn, &m).unwrap();
        let fetched = get_metrics_by_feature(&conn, 1).unwrap();
        assert_eq!(fetched.len(), 1);
        assert!(fetched[0].metadata.is_some());
    }

    #[test]
    fn record_metric_without_feature_id() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        record_metric(&conn, &sample_metric(None, "global-op")).unwrap();
        assert!(get_metrics_by_feature(&conn, 0).unwrap().is_empty());
    }

    #[test]
    fn metrics_ordered_by_timestamp() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let mut m1 = sample_metric(Some(1), "first");
        m1.timestamp = chrono::Utc::now() - chrono::Duration::hours(2);
        record_metric(&conn, &m1).unwrap();
        let mut m2 = sample_metric(Some(1), "second");
        m2.timestamp = chrono::Utc::now();
        record_metric(&conn, &m2).unwrap();
        let fetched = get_metrics_by_feature(&conn, 1).unwrap();
        assert_eq!(fetched[0].command, "first");
        assert_eq!(fetched[1].command, "second");
    }
}
