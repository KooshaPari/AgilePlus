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
    use agileplus_domain::domain::audit::AuditEntry;

    fn seed_feature(conn: &Connection, id: i64) {
        conn.execute(
            "INSERT OR IGNORE INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at) \
             VALUES (?1, ?2, ?3, 'created', X'00', 'main', datetime('now'), datetime('now'))",
            params![id, format!("feat-{id}"), format!("Feature {id}")],
        )
        .unwrap();
    }

    fn make_entry(feature_id: i64, actor: &str, prev_hash: [u8; 32], hash: [u8; 32]) -> AuditEntry {
        AuditEntry {
            id: 0,
            feature_id,
            wp_id: None,
            timestamp: chrono::Utc::now(),
            actor: actor.to_string(),
            transition: "created".to_string(),
            evidence_refs: vec![],
            prev_hash,
            hash,
            event_id: None,
            archived_to: None,
        }
    }

    #[test]
    fn test_append_and_get_trail() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let h1 = [1u8; 32];
        let entry = make_entry(1, "alice", [0u8; 32], h1);
        append_audit_entry(&conn, &entry).unwrap();
        let trail = get_audit_trail(&conn, 1).unwrap();
        assert_eq!(trail.len(), 1);
        assert_eq!(trail[0].actor, "alice");
        assert_eq!(trail[0].hash, h1);
    }

    #[test]
    fn test_chain_verification_succeeds() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let h1 = [1u8; 32];
        let h2 = [2u8; 32];
        append_audit_entry(&conn, &make_entry(1, "alice", [0u8; 32], h1)).unwrap();
        append_audit_entry(&conn, &make_entry(1, "bob", h1, h2)).unwrap();
        let trail = get_audit_trail(&conn, 1).unwrap();
        assert_eq!(trail.len(), 2);
        assert_eq!(trail[0].prev_hash, [0u8; 32]);
        assert_eq!(trail[1].prev_hash, h1);
    }

    #[test]
    fn test_wrong_prev_hash_rejected() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let h1 = [1u8; 32];
        append_audit_entry(&conn, &make_entry(1, "alice", [0u8; 32], h1)).unwrap();
        // Try to append with wrong prev_hash
        let bad = make_entry(1, "bob", [99u8; 32], [2u8; 32]);
        assert!(append_audit_entry(&conn, &bad).is_err());
    }

    #[test]
    fn test_first_entry_nonzero_prev_hash_rejected() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let bad = make_entry(1, "alice", [1u8; 32], [2u8; 32]);
        assert!(append_audit_entry(&conn, &bad).is_err());
    }

    #[test]
    fn test_get_latest_returns_last_entry() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let h1 = [1u8; 32];
        let h2 = [2u8; 32];
        append_audit_entry(&conn, &make_entry(1, "alice", [0u8; 32], h1)).unwrap();
        append_audit_entry(&conn, &make_entry(1, "bob", h1, h2)).unwrap();
        let latest = get_latest_audit_entry(&conn, 1).unwrap().unwrap();
        assert_eq!(latest.actor, "bob");
        assert_eq!(latest.hash, h2);
    }

    #[test]
    fn test_get_latest_nonexistent_returns_none() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        assert!(get_latest_audit_entry(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn test_trail_isolation_by_feature() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        seed_feature(&conn, 2);
        append_audit_entry(&conn, &make_entry(1, "alice", [0u8; 32], [1u8; 32])).unwrap();
        append_audit_entry(&conn, &make_entry(2, "bob", [0u8; 32], [2u8; 32])).unwrap();
        assert_eq!(get_audit_trail(&conn, 1).unwrap().len(), 1);
        assert_eq!(get_audit_trail(&conn, 2).unwrap().len(), 1);
    }

    /// Insert an audit row directly, bypassing the chain checks.
    fn insert_raw_entry(
        conn: &Connection,
        evidence_refs: &str,
        prev_hash: &[u8],
        hash: &[u8],
        timestamp: &str,
    ) {
        conn.execute(
            "INSERT INTO audit_log
             (feature_id, wp_id, timestamp, actor, transition, evidence_refs, prev_hash, hash)
             VALUES (1, NULL, ?1, 'agent', 'created', ?2, ?3, ?4)",
            params![timestamp, evidence_refs, prev_hash, hash],
        )
        .expect("raw audit insert");
    }

    #[test]
    fn short_hash_blobs_read_as_zeroed_32_bytes() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        insert_raw_entry(
            &conn,
            "[]",
            &[7u8, 7u8],
            &[9u8],
            &chrono::Utc::now().to_rfc3339(),
        );

        let trail = get_audit_trail(&conn, 1).unwrap();
        assert_eq!(trail.len(), 1);
        assert_eq!(trail[0].prev_hash, [0u8; 32]);
        assert_eq!(trail[0].hash, [0u8; 32]);
    }

    #[test]
    fn malformed_evidence_refs_reads_as_empty() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        insert_raw_entry(
            &conn,
            "{not-a-list}",
            &[0u8; 32],
            &[1u8; 32],
            &chrono::Utc::now().to_rfc3339(),
        );

        let trail = get_audit_trail(&conn, 1).unwrap();
        assert!(trail[0].evidence_refs.is_empty());
    }

    #[test]
    fn corrupt_timestamp_reads_as_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        insert_raw_entry(&conn, "[]", &[0u8; 32], &[1u8; 32], "not-a-timestamp");

        let err = get_audit_trail(&conn, 1).unwrap_err();
        assert!(
            matches!(err, agileplus_domain::error::DomainError::Storage(_)),
            "got {err:?}"
        );
    }
}
