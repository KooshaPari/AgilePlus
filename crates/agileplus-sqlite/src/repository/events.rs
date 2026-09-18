//! Event repository — CRUD for the `events` table.

use rusqlite::{Connection, Row, params};

use agileplus_domain::domain::event::Event;
use agileplus_domain::error::DomainError;

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn row_to_event(row: &Row<'_>) -> rusqlite::Result<Event> {
    let id: i64 = row.get(0)?;
    let entity_type: String = row.get(1)?;
    let entity_id: i64 = row.get(2)?;
    let event_type: String = row.get(3)?;
    let payload_str: String = row.get(4)?;
    let actor: String = row.get(5)?;
    let timestamp_str: String = row.get(6)?;
    let prev_hash_bytes: Vec<u8> = row.get(7)?;
    let hash_bytes: Vec<u8> = row.get(8)?;
    let sequence: i64 = row.get(9)?;

    let timestamp = timestamp_str
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                6,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    let payload: serde_json::Value = serde_json::from_str(&payload_str).unwrap_or_default();

    let mut prev_hash = [0u8; 32];
    if prev_hash_bytes.len() == 32 {
        prev_hash.copy_from_slice(&prev_hash_bytes);
    }
    let mut hash = [0u8; 32];
    if hash_bytes.len() == 32 {
        hash.copy_from_slice(&hash_bytes);
    }

    Ok(Event {
        id,
        entity_type,
        entity_id,
        event_type,
        payload,
        actor,
        timestamp,
        prev_hash,
        hash,
        sequence,
    })
}

const SELECT_COLS: &str =
    "id, entity_type, entity_id, event_type, payload, actor, timestamp, prev_hash, hash, sequence";

pub fn append_event(conn: &Connection, event: &Event) -> Result<i64, DomainError> {
    let payload_json = serde_json::to_string(&event.payload)
        .map_err(|e| DomainError::Storage(format!("serialize payload: {e}")))?;

    conn.execute(
        &format!(
            "INSERT INTO events ({SELECT_COLS}) VALUES (NULL, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
        ),
        params![
            event.entity_type,
            event.entity_id,
            event.event_type,
            payload_json,
            event.actor,
            event.timestamp.to_rfc3339(),
            &event.prev_hash[..],
            &event.hash[..],
            event.sequence,
        ],
    )
    .map_err(map_err)?;

    Ok(conn.last_insert_rowid())
}

pub fn get_events(
    conn: &Connection,
    entity_type: &str,
    entity_id: i64,
) -> Result<Vec<Event>, DomainError> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {SELECT_COLS} FROM events WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY sequence ASC"
        ))
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![entity_type, entity_id], row_to_event)
        .map_err(map_err)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(map_err)
}

pub fn get_events_since(
    conn: &Connection,
    entity_type: &str,
    entity_id: i64,
    sequence: i64,
) -> Result<Vec<Event>, DomainError> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {SELECT_COLS} FROM events WHERE entity_type = ?1 AND entity_id = ?2 AND sequence > ?3 ORDER BY sequence ASC"
        ))
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![entity_type, entity_id, sequence], row_to_event)
        .map_err(map_err)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(map_err)
}

pub fn get_events_by_range(
    conn: &Connection,
    entity_type: &str,
    entity_id: i64,
    from: &str,
    to: &str,
) -> Result<Vec<Event>, DomainError> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {SELECT_COLS} FROM events WHERE entity_type = ?1 AND entity_id = ?2 AND timestamp >= ?3 AND timestamp <= ?4 ORDER BY sequence ASC"
        ))
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![entity_type, entity_id, from, to], row_to_event)
        .map_err(map_err)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(map_err)
}

pub fn get_latest_sequence(
    conn: &Connection,
    entity_type: &str,
    entity_id: i64,
) -> Result<i64, DomainError> {
    let result: Option<i64> = conn
        .query_row(
            "SELECT MAX(sequence) FROM events WHERE entity_type = ?1 AND entity_id = ?2",
            params![entity_type, entity_id],
            |row| row.get(0),
        )
        .map_err(map_err)?;

    Ok(result.unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    use super::{
        append_event, get_events, get_events_by_range, get_events_since, get_latest_sequence,
    };
    use crate::SqliteStorageAdapter;
    use agileplus_domain::domain::event::Event;

    fn event(
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
        timestamp: chrono::DateTime<Utc>,
    ) -> Event {
        Event {
            id: 0,
            entity_type: entity_type.to_owned(),
            entity_id,
            event_type: "transitioned".to_owned(),
            payload: json!({"state": "validated", "sequence": sequence}),
            actor: "dashboard-agent".to_owned(),
            timestamp,
            prev_hash: [sequence as u8; 32],
            hash: [(sequence + 1) as u8; 32],
            sequence,
        }
    }

    #[test]
    fn append_and_get_events_preserves_payload_hashes_and_stream_order() {
        let adapter = SqliteStorageAdapter::in_memory().expect("in-memory storage");
        let conn = adapter.conn_for_bench().expect("database connection");
        let first_at = Utc
            .with_ymd_and_hms(2026, 8, 29, 10, 0, 0)
            .single()
            .unwrap();
        let second_at = Utc
            .with_ymd_and_hms(2026, 8, 29, 10, 1, 0)
            .single()
            .unwrap();

        append_event(&conn, &event("Feature", 42, 2, second_at)).expect("append second event");
        append_event(&conn, &event("Feature", 42, 1, first_at)).expect("append first event");
        append_event(&conn, &event("Feature", 99, 1, first_at)).expect("append other stream");

        let events = get_events(&conn, "Feature", 42).expect("read event stream");
        assert_eq!(
            events
                .iter()
                .map(|event| event.sequence)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            events[0].payload,
            json!({"state": "validated", "sequence": 1})
        );
        assert_eq!(events[1].prev_hash, [2; 32]);
        assert_eq!(events[1].hash, [3; 32]);
        assert_eq!(get_latest_sequence(&conn, "Feature", 42).unwrap(), 2);
    }

    #[test]
    fn since_and_range_queries_use_exclusive_sequence_and_inclusive_timestamps() {
        let adapter = SqliteStorageAdapter::in_memory().expect("in-memory storage");
        let conn = adapter.conn_for_bench().expect("database connection");
        let first_at = Utc
            .with_ymd_and_hms(2026, 8, 29, 10, 0, 0)
            .single()
            .unwrap();
        let second_at = Utc
            .with_ymd_and_hms(2026, 8, 29, 10, 1, 0)
            .single()
            .unwrap();
        let third_at = Utc
            .with_ymd_and_hms(2026, 8, 29, 10, 2, 0)
            .single()
            .unwrap();

        for (sequence, timestamp) in [(1, first_at), (2, second_at), (3, third_at)] {
            append_event(&conn, &event("WorkPackage", 7, sequence, timestamp))
                .expect("append event");
        }

        let since = get_events_since(&conn, "WorkPackage", 7, 1).expect("read newer events");
        assert_eq!(
            since.iter().map(|event| event.sequence).collect::<Vec<_>>(),
            [2, 3]
        );

        let range = get_events_by_range(
            &conn,
            "WorkPackage",
            7,
            &second_at.to_rfc3339(),
            &third_at.to_rfc3339(),
        )
        .expect("read time range");
        assert_eq!(
            range.iter().map(|event| event.sequence).collect::<Vec<_>>(),
            [2, 3]
        );
    }

    /// Insert an event row directly so a corrupted column can be read back.
    fn insert_raw_event(
        conn: &rusqlite::Connection,
        payload: &str,
        prev_hash: &[u8],
        hash: &[u8],
        timestamp: &str,
    ) {
        conn.execute(
            "INSERT INTO events (entity_type, entity_id, event_type, payload, actor, timestamp, prev_hash, hash, sequence)
             VALUES ('Feature', 5, 'transitioned', ?1, 'agent', ?2, ?3, ?4, 1)",
            rusqlite::params![payload, timestamp, prev_hash, hash],
        )
        .expect("raw event insert");
    }

    #[test]
    fn non_json_payload_reads_as_null_instead_of_failing() {
        let adapter = SqliteStorageAdapter::in_memory().expect("in-memory storage");
        let conn = adapter.conn_for_bench().expect("database connection");
        insert_raw_event(
            &conn,
            "definitely not json",
            &[0u8; 32],
            &[0u8; 32],
            &Utc.with_ymd_and_hms(2026, 8, 29, 10, 0, 0)
                .single()
                .unwrap()
                .to_rfc3339(),
        );

        let events = get_events(&conn, "Feature", 5).expect("read event stream");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].payload, serde_json::Value::Null);
    }

    #[test]
    fn short_hash_blobs_read_as_zeroed_32_bytes() {
        let adapter = SqliteStorageAdapter::in_memory().expect("in-memory storage");
        let conn = adapter.conn_for_bench().expect("database connection");
        insert_raw_event(
            &conn,
            "{}",
            &[1u8],
            &[2u8, 3u8],
            &Utc.with_ymd_and_hms(2026, 8, 29, 10, 0, 0)
                .single()
                .unwrap()
                .to_rfc3339(),
        );

        let events = get_events(&conn, "Feature", 5).expect("read event stream");
        assert_eq!(events[0].prev_hash, [0u8; 32]);
        assert_eq!(events[0].hash, [0u8; 32]);
    }

    #[test]
    fn corrupt_timestamp_column_is_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().expect("in-memory storage");
        let conn = adapter.conn_for_bench().expect("database connection");
        insert_raw_event(&conn, "{}", &[0u8; 32], &[0u8; 32], "not-a-timestamp");

        let err = get_events(&conn, "Feature", 5).unwrap_err();
        assert!(
            matches!(err, agileplus_domain::error::DomainError::Storage(_)),
            "got {err:?}"
        );
    }
}
