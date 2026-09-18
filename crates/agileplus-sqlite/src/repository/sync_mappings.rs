//! Sync mapping repository functions.
//!
//! Traceability: WP06-T033

use chrono::Utc;
use rusqlite::{Connection, params};

use agileplus_domain::domain::sync_mapping::{SyncDirection, SyncMapping};
use agileplus_domain::error::DomainError;

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

/// Look up a sync mapping by entity_type and entity_id.
pub fn get_sync_mapping(
    conn: &Connection,
    entity_type: &str,
    entity_id: i64,
) -> Result<Option<SyncMapping>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, entity_type, entity_id, plane_issue_id, content_hash, \
             last_synced_at, sync_direction, conflict_count \
             FROM sync_mappings WHERE entity_type = ?1 AND entity_id = ?2",
        )
        .map_err(map_err)?;

    let result = stmt.query_row(params![entity_type, entity_id], |row| {
        Ok(SyncMapping {
            id: row.get(0)?,
            entity_type: row.get(1)?,
            entity_id: row.get(2)?,
            plane_issue_id: row.get(3)?,
            content_hash: row.get(4)?,
            last_synced_at: row.get::<_, String>(5).map(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now())
            })?,
            sync_direction: row.get::<_, String>(6).map(|s| match s.as_str() {
                "push" => SyncDirection::Push,
                "pull" => SyncDirection::Pull,
                _ => SyncDirection::Bidirectional,
            })?,
            conflict_count: row.get(7)?,
        })
    });

    match result {
        Ok(m) => Ok(Some(m)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

/// Insert or update a sync mapping (upsert on entity_type + entity_id).
pub fn upsert_sync_mapping(conn: &Connection, mapping: &SyncMapping) -> Result<(), DomainError> {
    conn.execute(
        "INSERT INTO sync_mappings (entity_type, entity_id, plane_issue_id, content_hash, \
         last_synced_at, sync_direction, conflict_count) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
         ON CONFLICT (entity_type, entity_id) DO UPDATE SET \
             plane_issue_id = excluded.plane_issue_id, \
             content_hash = excluded.content_hash, \
             last_synced_at = excluded.last_synced_at, \
             sync_direction = excluded.sync_direction, \
             conflict_count = excluded.conflict_count",
        params![
            mapping.entity_type,
            mapping.entity_id,
            mapping.plane_issue_id,
            mapping.content_hash,
            mapping.last_synced_at.to_rfc3339(),
            mapping.sync_direction.to_string(),
            mapping.conflict_count,
        ],
    )
    .map_err(map_err)?;
    Ok(())
}

/// Look up a sync mapping by entity_type and plane_issue_id.
pub fn get_sync_mapping_by_plane_id(
    conn: &Connection,
    entity_type: &str,
    plane_issue_id: &str,
) -> Result<Option<SyncMapping>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, entity_type, entity_id, plane_issue_id, content_hash, \
             last_synced_at, sync_direction, conflict_count \
             FROM sync_mappings WHERE entity_type = ?1 AND plane_issue_id = ?2",
        )
        .map_err(map_err)?;

    let result = stmt.query_row(params![entity_type, plane_issue_id], |row| {
        Ok(SyncMapping {
            id: row.get(0)?,
            entity_type: row.get(1)?,
            entity_id: row.get(2)?,
            plane_issue_id: row.get(3)?,
            content_hash: row.get(4)?,
            last_synced_at: row.get::<_, String>(5).map(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now())
            })?,
            sync_direction: row.get::<_, String>(6).map(|s| match s.as_str() {
                "push" => SyncDirection::Push,
                "pull" => SyncDirection::Pull,
                _ => SyncDirection::Bidirectional,
            })?,
            conflict_count: row.get(7)?,
        })
    });

    match result {
        Ok(m) => Ok(Some(m)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

/// Delete a sync mapping by entity_type and entity_id.
pub fn delete_sync_mapping(
    conn: &Connection,
    entity_type: &str,
    entity_id: i64,
) -> Result<(), DomainError> {
    conn.execute(
        "DELETE FROM sync_mappings WHERE entity_type = ?1 AND entity_id = ?2",
        params![entity_type, entity_id],
    )
    .map_err(map_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;

    #[test]
    fn test_upsert_and_get() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let m = SyncMapping::new("feature", 1, "PLN-123", "hash1");
        upsert_sync_mapping(&conn, &m).unwrap();
        let got = get_sync_mapping(&conn, "feature", 1).unwrap().unwrap();
        assert_eq!(got.plane_issue_id, "PLN-123");
        assert_eq!(got.entity_type, "feature");
    }

    #[test]
    fn test_upsert_updates_existing() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let mut m = SyncMapping::new("feature", 1, "PLN-123", "hash1");
        upsert_sync_mapping(&conn, &m).unwrap();
        m.content_hash = "hash2".to_string();
        m.plane_issue_id = "PLN-456".to_string();
        upsert_sync_mapping(&conn, &m).unwrap();
        let got = get_sync_mapping(&conn, "feature", 1).unwrap().unwrap();
        assert_eq!(got.plane_issue_id, "PLN-456");
        assert_eq!(got.content_hash, "hash2");
    }

    #[test]
    fn test_get_by_plane_id() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let m = SyncMapping::new("epic", 5, "PLN-789", "hash3");
        upsert_sync_mapping(&conn, &m).unwrap();
        let got = get_sync_mapping_by_plane_id(&conn, "epic", "PLN-789")
            .unwrap()
            .unwrap();
        assert_eq!(got.entity_id, 5);
    }

    #[test]
    fn test_delete() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let m = SyncMapping::new("feature", 10, "PLN-10", "h");
        upsert_sync_mapping(&conn, &m).unwrap();
        delete_sync_mapping(&conn, "feature", 10).unwrap();
        assert!(get_sync_mapping(&conn, "feature", 10).unwrap().is_none());
    }

    #[test]
    fn test_get_nonexistent() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(get_sync_mapping(&conn, "feature", 999).unwrap().is_none());
    }

    #[test]
    fn test_different_entity_types() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let m1 = SyncMapping::new("feature", 1, "PLN-F1", "h1");
        let m2 = SyncMapping::new("epic", 1, "PLN-E1", "h2");
        upsert_sync_mapping(&conn, &m1).unwrap();
        upsert_sync_mapping(&conn, &m2).unwrap();
        let g1 = get_sync_mapping(&conn, "feature", 1).unwrap().unwrap();
        let g2 = get_sync_mapping(&conn, "epic", 1).unwrap().unwrap();
        assert_eq!(g1.plane_issue_id, "PLN-F1");
        assert_eq!(g2.plane_issue_id, "PLN-E1");
    }

    #[test]
    fn unparseable_timestamp_and_unknown_direction_fall_back() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        conn.execute(
            "INSERT INTO sync_mappings
             (entity_type, entity_id, plane_issue_id, content_hash, last_synced_at, sync_direction, conflict_count)
             VALUES ('feature', 3, 'PLN-3', 'h', 'not-a-timestamp', 'sideways', 0)",
            [],
        )
        .unwrap();

        let before = Utc::now();
        let got = get_sync_mapping(&conn, "feature", 3).unwrap().unwrap();
        assert_eq!(got.sync_direction, SyncDirection::Bidirectional);
        assert!(
            got.last_synced_at >= before - chrono::Duration::seconds(1),
            "unparseable timestamp must fall back to now, got {}",
            got.last_synced_at
        );

        let by_plane = get_sync_mapping_by_plane_id(&conn, "feature", "PLN-3")
            .unwrap()
            .unwrap();
        assert_eq!(by_plane.sync_direction, SyncDirection::Bidirectional);
        assert!(by_plane.last_synced_at >= before - chrono::Duration::seconds(1));
    }
}
