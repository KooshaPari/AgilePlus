// SPDX-License-Identifier: MIT OR Apache-2.0
//! Device-registry integrity and monotonic revision helpers.

use chrono::{DateTime, Utc};
use rusqlite::{Connection, Transaction};

use crate::model::RegistryError;

pub(crate) fn validate_integrity(connection: &Connection) -> Result<(), RegistryError> {
    let mut statement = connection
        .prepare("PRAGMA quick_check")
        .map_err(|source| RegistryError::Sqlite {
            context: "cannot prepare device-registry integrity check".into(),
            source,
        })?;
    let messages = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|source| RegistryError::Sqlite {
            context: "cannot execute device-registry integrity check".into(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| RegistryError::Sqlite {
            context: "cannot decode device-registry integrity result".into(),
            source,
        })?;
    if messages.len() == 1 && messages[0] == "ok" {
        Ok(())
    } else {
        Err(RegistryError::Corrupt(format!(
            "SQLite quick_check returned: {}",
            messages.join("; ")
        )))
    }
}

pub(crate) fn bump_revision(
    transaction: &Transaction<'_>,
    now: DateTime<Utc>,
) -> Result<i64, RegistryError> {
    let current: i64 = transaction
        .query_row(
            "SELECT revision FROM registry_metadata WHERE singleton=1",
            [],
            |row| row.get(0),
        )
        .map_err(|source| RegistryError::Sqlite {
            context: "cannot read registry revision".into(),
            source,
        })?;
    let next = current
        .checked_add(1)
        .ok_or(RegistryError::RevisionOverflow)?;
    let changed = transaction
        .execute(
            "UPDATE registry_metadata SET revision=?1, updated_at=?2 WHERE singleton=1",
            rusqlite::params![next, now.to_rfc3339()],
        )
        .map_err(|source| RegistryError::Sqlite {
            context: "cannot advance registry revision".into(),
            source,
        })?;
    if changed != 1 {
        return Err(RegistryError::Corrupt(format!(
            "registry revision update changed {changed} rows"
        )));
    }
    Ok(next)
}

pub(crate) fn read_revision(connection: &Connection) -> Result<u64, RegistryError> {
    let revision: i64 = connection
        .query_row(
            "SELECT revision FROM registry_metadata WHERE singleton=1",
            [],
            |row| row.get(0),
        )
        .map_err(|source| RegistryError::Sqlite {
            context: "cannot read registry revision".into(),
            source,
        })?;
    u64::try_from(revision)
        .map_err(|_| RegistryError::Corrupt("registry revision is negative".into()))
}
