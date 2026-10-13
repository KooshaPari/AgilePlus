// SPDX-License-Identifier: MIT OR Apache-2.0
//! Immutable stable identities layered over local integer aggregate keys.

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Row, TransactionBehavior, params};
use uuid::Uuid;

use agileplus_domain::{
    domain::stable_identity::{StableEntityIdentity, StableEntityKind},
    error::DomainError,
};

fn map_err(error: rusqlite::Error) -> DomainError {
    match &error {
        rusqlite::Error::SqliteFailure(code, _)
            if matches!(
                code.code,
                rusqlite::ErrorCode::ConstraintViolation
            ) =>
        {
            DomainError::Conflict(format!("stable identity constraint failed: {error}"))
        }
        _ => DomainError::Storage(error.to_string()),
    }
}

fn conversion_error(
    index: usize,
    value_type: rusqlite::types::Type,
    message: impl Into<String>,
) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        index,
        value_type,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message.into(),
        )),
    )
}

fn row_to_identity(row: &Row<'_>) -> rusqlite::Result<StableEntityIdentity> {
    let kind_text: String = row.get(0)?;
    let entity_id: i64 = row.get(1)?;
    let uid_text: String = row.get(2)?;
    let created_at_text: String = row.get(3)?;
    let tombstoned_at_text: Option<String> = row.get(4)?;

    let kind = StableEntityKind::parse(&kind_text).map_err(|error| {
        conversion_error(
            0,
            rusqlite::types::Type::Text,
            format!("invalid stable entity kind {kind_text:?}: {error}"),
        )
    })?;
    let uid = Uuid::parse_str(&uid_text).map_err(|error| {
        conversion_error(
            2,
            rusqlite::types::Type::Text,
            format!("invalid stable entity UID {uid_text:?}: {error}"),
        )
    })?;
    let created_at = created_at_text.parse::<DateTime<Utc>>().map_err(|error| {
        conversion_error(
            3,
            rusqlite::types::Type::Text,
            format!("invalid stable identity creation time: {error}"),
        )
    })?;
    let tombstoned_at = tombstoned_at_text
        .map(|value| {
            value.parse::<DateTime<Utc>>().map_err(|error| {
                conversion_error(
                    4,
                    rusqlite::types::Type::Text,
                    format!("invalid stable identity tombstone time: {error}"),
                )
            })
        })
        .transpose()?;

    let mut identity = StableEntityIdentity::new(kind, entity_id, uid, created_at).map_err(|error| {
        conversion_error(
            2,
            rusqlite::types::Type::Text,
            format!("stable identity invariant failed: {error}"),
        )
    })?;
    identity.tombstoned_at = tombstoned_at;
    Ok(identity)
}

/// Look up the stable identity assigned to one local aggregate row.
pub fn get_by_entity(
    connection: &Connection,
    entity_kind: StableEntityKind,
    entity_id: i64,
) -> Result<Option<StableEntityIdentity>, DomainError> {
    connection
        .query_row(
            "SELECT entity_kind, entity_id, uid, created_at, tombstoned_at
             FROM stable_entity_ids
             WHERE entity_kind=?1 AND entity_id=?2",
            params![entity_kind.as_str(), entity_id],
            row_to_identity,
        )
        .optional()
        .map_err(map_err)
}

/// Resolve a stable UID without assuming its local integer key.
pub fn get_by_uid(
    connection: &Connection,
    uid: Uuid,
) -> Result<Option<StableEntityIdentity>, DomainError> {
    connection
        .query_row(
            "SELECT entity_kind, entity_id, uid, created_at, tombstoned_at
             FROM stable_entity_ids WHERE uid=?1",
            [uid.to_string()],
            row_to_identity,
        )
        .optional()
        .map_err(map_err)
}

/// Assign a caller-supplied UUIDv7 to an existing aggregate row.
///
/// This is used by controlled imports and tests. Ordinary creation and legacy
/// backfill generate UUIDv7 values internally.
pub fn assign_identity(
    connection: &Connection,
    entity_kind: StableEntityKind,
    entity_id: i64,
    uid: Uuid,
    created_at: DateTime<Utc>,
) -> Result<StableEntityIdentity, DomainError> {
    let identity = StableEntityIdentity::new(entity_kind, entity_id, uid, created_at)?;
    if !entity_exists(connection, entity_kind, entity_id)? {
        return Err(DomainError::NotFound(format!(
            "{} row {entity_id}",
            entity_kind.as_str()
        )));
    }
    connection
        .execute(
            "INSERT INTO stable_entity_ids (
                entity_kind, entity_id, uid, created_at, tombstoned_at
             ) VALUES (?1, ?2, ?3, ?4, NULL)",
            params![
                entity_kind.as_str(),
                entity_id,
                uid.to_string(),
                created_at.to_rfc3339(),
            ],
        )
        .map_err(map_err)?;
    Ok(identity)
}

/// Return the existing live identity or assign one to an existing legacy row.
pub fn ensure_identity(
    connection: &Connection,
    entity_kind: StableEntityKind,
    entity_id: i64,
) -> Result<StableEntityIdentity, DomainError> {
    if let Some(existing) = get_by_entity(connection, entity_kind, entity_id)? {
        if existing.tombstoned_at.is_some() {
            return Err(DomainError::Conflict(format!(
                "{} row {entity_id} has a tombstoned identity {}",
                entity_kind.as_str(),
                existing.uid
            )));
        }
        return Ok(existing);
    }
    assign_identity(
        connection,
        entity_kind,
        entity_id,
        Uuid::now_v7(),
        Utc::now(),
    )
}

/// Backfill identities for all existing project, feature, and work-package rows.
///
/// The whole backfill commits atomically. Re-running it is idempotent.
pub fn backfill_missing(connection: &mut Connection) -> Result<usize, DomainError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(map_err)?;
    let mut inserted = 0_usize;
    for entity_kind in [
        StableEntityKind::Project,
        StableEntityKind::Feature,
        StableEntityKind::WorkPackage,
    ] {
        let ids = missing_ids(&transaction, entity_kind)?;
        for entity_id in ids {
            assign_identity(
                &transaction,
                entity_kind,
                entity_id,
                Uuid::now_v7(),
                Utc::now(),
            )?;
            inserted += 1;
        }
    }
    transaction.commit().map_err(map_err)?;
    Ok(inserted)
}

/// Verify that every live aggregate has exactly one live stable identity and
/// that no live identity references a missing row.
pub fn validate_complete(connection: &Connection) -> Result<(), DomainError> {
    for entity_kind in [
        StableEntityKind::Project,
        StableEntityKind::Feature,
        StableEntityKind::WorkPackage,
    ] {
        let table = entity_table(entity_kind);
        let missing_sql = format!(
            "SELECT COUNT(*) FROM {table} e
             WHERE NOT EXISTS (
                 SELECT 1 FROM stable_entity_ids s
                 WHERE s.entity_kind=?1 AND s.entity_id=e.id
                   AND s.tombstoned_at IS NULL
             )"
        );
        let missing: i64 = connection
            .query_row(&missing_sql, [entity_kind.as_str()], |row| row.get(0))
            .map_err(map_err)?;
        if missing != 0 {
            return Err(DomainError::Storage(format!(
                "{missing} live {} rows lack stable identities",
                entity_kind.as_str()
            )));
        }

        let orphan_sql = format!(
            "SELECT COUNT(*) FROM stable_entity_ids s
             WHERE s.entity_kind=?1 AND s.tombstoned_at IS NULL
               AND NOT EXISTS (SELECT 1 FROM {table} e WHERE e.id=s.entity_id)"
        );
        let orphans: i64 = connection
            .query_row(&orphan_sql, [entity_kind.as_str()], |row| row.get(0))
            .map_err(map_err)?;
        if orphans != 0 {
            return Err(DomainError::Storage(format!(
                "{orphans} live {} identities reference missing rows",
                entity_kind.as_str()
            )));
        }
    }
    Ok(())
}

fn missing_ids(
    connection: &Connection,
    entity_kind: StableEntityKind,
) -> Result<Vec<i64>, DomainError> {
    let table = entity_table(entity_kind);
    let sql = format!(
        "SELECT e.id FROM {table} e
         WHERE NOT EXISTS (
             SELECT 1 FROM stable_entity_ids s
             WHERE s.entity_kind=?1 AND s.entity_id=e.id
         )
         ORDER BY e.id"
    );
    let mut statement = connection.prepare(&sql).map_err(map_err)?;
    statement
        .query_map([entity_kind.as_str()], |row| row.get(0))
        .map_err(map_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_err)
}

fn entity_exists(
    connection: &Connection,
    entity_kind: StableEntityKind,
    entity_id: i64,
) -> Result<bool, DomainError> {
    let table = entity_table(entity_kind);
    let sql = format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id=?1)");
    connection
        .query_row(&sql, [entity_id], |row| row.get(0))
        .map_err(map_err)
}

const fn entity_table(entity_kind: StableEntityKind) -> &'static str {
    match entity_kind {
        StableEntityKind::Project => "projects",
        StableEntityKind::Feature => "features",
        StableEntityKind::WorkPackage => "work_packages",
    }
}
