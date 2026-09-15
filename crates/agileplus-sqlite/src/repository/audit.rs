//! Audit repository — CRUD for the `audit_log` table.

use rusqlite::{Connection, Row, params};

use agileplus_domain::{
    domain::audit::{AuditEntry, EvidenceRef},
    error::DomainError,
};

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn row_to_audit_entry(row: &Row<'_>) -> rusqlite::Result<AuditEntry> {
    let id: i64 = row.get(0)?;
    let feature_id: i64 = row.get(1)?;
    let wp_id: Option<i64> = row.get(2)?;
    let timestamp_s: String = row.get(3)?;
    let actor: String = row.get(4)?;
    let transition: String = row.get(5)?;
    let evidence_refs_json: String = row.get(6)?;
    let prev_hash_bytes: Vec<u8> = row.get(7)?;
    let hash_bytes: Vec<u8> = row.get(8)?;

    let timestamp = timestamp_s
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    let evidence_refs: Vec<EvidenceRef> =
        serde_json::from_str(&evidence_refs_json).unwrap_or_default();

    let mut prev_hash = [0u8; 32];
    if prev_hash_bytes.len() == 32 {
        prev_hash.copy_from_slice(&prev_hash_bytes);
    }

    let mut hash = [0u8; 32];
    if hash_bytes.len() == 32 {
        hash.copy_from_slice(&hash_bytes);
    }

    Ok(AuditEntry {
        id,
        feature_id,
        wp_id,
        timestamp,
        actor,
        transition,
        evidence_refs,
        prev_hash,
        hash,
        event_id: None,
        archived_to: None,
    })
}

/// Append an audit entry. Performs a defense-in-depth chain check.
pub fn append_audit_entry(conn: &Connection, entry: &AuditEntry) -> Result<i64, DomainError> {
    // Defense-in-depth: verify prev_hash matches latest entry's hash
    let latest = get_latest_audit_entry(conn, entry.feature_id)?;
    if let Some(ref latest) = latest {
        if entry.prev_hash != latest.hash {
            return Err(DomainError::Storage(
                "audit chain broken: prev_hash does not match latest entry hash".into(),
            ));
        }
    } else {
        // First entry: prev_hash should be all zeros
        if entry.prev_hash != [0u8; 32] {
            return Err(DomainError::Storage(
                "audit chain broken: first entry prev_hash must be all zeros".into(),
            ));
        }
    }

    let evidence_refs_json = serde_json::to_string(&entry.evidence_refs)
        .map_err(|e| DomainError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO audit_log
         (feature_id, wp_id, timestamp, actor, transition, evidence_refs, prev_hash, hash)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            entry.feature_id,
            entry.wp_id,
            entry.timestamp.to_rfc3339(),
            entry.actor,
            entry.transition,
            evidence_refs_json,
            entry.prev_hash.as_slice(),
            entry.hash.as_slice(),
        ],
    )
    .map_err(map_err)?;

    Ok(conn.last_insert_rowid())
}

pub fn get_audit_trail(conn: &Connection, feature_id: i64) -> Result<Vec<AuditEntry>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id,feature_id,wp_id,timestamp,actor,transition,evidence_refs,prev_hash,hash
             FROM audit_log WHERE feature_id = ?1 ORDER BY id ASC",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![feature_id], row_to_audit_entry)
        .map_err(map_err)?;

    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn get_latest_audit_entry(
    conn: &Connection,
    feature_id: i64,
) -> Result<Option<AuditEntry>, DomainError> {
    conn.query_row(
        "SELECT id,feature_id,wp_id,timestamp,actor,transition,evidence_refs,prev_hash,hash
         FROM audit_log WHERE feature_id = ?1 ORDER BY id DESC LIMIT 1",
        params![feature_id],
        row_to_audit_entry,
    )
    .optional()
    .map_err(map_err)
}

/// Extension trait to add `.optional()` on rusqlite query results.
trait OptionalExt<T> {
    fn optional(self) -> rusqlite::Result<Option<T>>;
}

impl<T> OptionalExt<T> for rusqlite::Result<T> {
    fn optional(self) -> rusqlite::Result<Option<T>> {
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
    use crate::SqliteStorageAdapter;

    fn make_entry(feature_id: i64, prev_hash: [u8; 32], hash_val: [u8; 32]) -> AuditEntry {
        AuditEntry {
            id: 0,
            feature_id,
            wp_id: None,
            timestamp: chrono::Utc::now(),
            actor: "test-agent".to_string(),
            transition: "Created->Specified".to_string(),
            evidence_refs: vec![],
            prev_hash,
            hash: hash_val,
            event_id: None,
            archived_to: None,
        }
    }

    #[test]
    fn append_first_entry_with_zero_prev_hash() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let entry = make_entry(1, [0u8; 32], [1u8; 32]);
        let id = append_audit_entry(&conn, &entry).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn append_entry_rejects_non_zero_prev_hash_on_first() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let entry = make_entry(1, [0xaa; 32], [1u8; 32]);
        let result = append_audit_entry(&conn, &entry);
        assert!(result.is_err());
    }

    #[test]
    fn append_chained_entries() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let e1 = make_entry(1, [0u8; 32], [1u8; 32]);
        append_audit_entry(&conn, &e1).unwrap();
        let e2 = make_entry(1, [1u8; 32], [2u8; 32]);
        append_audit_entry(&conn, &e2).unwrap();
        let trail = get_audit_trail(&conn, 1).unwrap();
        assert_eq!(trail.len(), 2);
    }

    #[test]
    fn append_entry_rejects_broken_chain() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let e1 = make_entry(1, [0u8; 32], [1u8; 32]);
        append_audit_entry(&conn, &e1).unwrap();
        let e2 = make_entry(1, [0xff; 32], [2u8; 32]);
        let result = append_audit_entry(&conn, &e2);
        assert!(result.is_err());
    }

    #[test]
    fn get_audit_trail_empty_for_unknown() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(get_audit_trail(&conn, 999).unwrap().is_empty());
    }

    #[test]
    fn get_latest_returns_most_recent() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        append_audit_entry(&conn, &make_entry(1, [0u8; 32], [1u8; 32])).unwrap();
        append_audit_entry(&conn, &make_entry(1, [1u8; 32], [2u8; 32])).unwrap();
        let latest = get_latest_audit_entry(&conn, 1).unwrap().unwrap();
        assert_eq!(latest.hash, [2u8; 32]);
    }

    #[test]
    fn get_latest_none_for_empty() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(get_latest_audit_entry(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn trail_filters_by_feature_id() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        append_audit_entry(&conn, &make_entry(1, [0u8; 32], [1u8; 32])).unwrap();
        append_audit_entry(&conn, &make_entry(2, [0u8; 32], [1u8; 32])).unwrap();
        assert_eq!(get_audit_trail(&conn, 1).unwrap().len(), 1);
        assert_eq!(get_audit_trail(&conn, 2).unwrap().len(), 1);
    }
}
