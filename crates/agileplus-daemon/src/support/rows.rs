// SPDX-License-Identifier: MIT OR Apache-2.0
//! Strict decoding and semantic validation for project registry rows.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::model::{DesiredState, ObservedState, RegisteredProject, RegistryError};

pub(crate) fn get_project_from_connection(
    connection: &Connection,
    project_uid: Uuid,
) -> Result<Option<RegisteredProject>, RegistryError> {
    connection
        .query_row(
            "SELECT project_uid, canonical_repo_root, database_path, grpc_endpoint,
                    desired_state, observed_state, child_pid, child_revision,
                    last_started_at, last_healthy_at, restart_count,
                    registration_revision, updated_revision, last_error
             FROM registered_projects WHERE project_uid=?1",
            [project_uid.to_string()],
            row_to_project,
        )
        .optional()
        .map_err(|source| RegistryError::Sqlite {
            context: format!("cannot read project {project_uid}"),
            source,
        })?
        .map(validate_project_invariants)
        .transpose()
}

pub(crate) fn list_projects_from_connection(
    connection: &Connection,
) -> Result<Vec<RegisteredProject>, RegistryError> {
    let mut statement = connection
        .prepare(
            "SELECT project_uid, canonical_repo_root, database_path, grpc_endpoint,
                    desired_state, observed_state, child_pid, child_revision,
                    last_started_at, last_healthy_at, restart_count,
                    registration_revision, updated_revision, last_error
             FROM registered_projects ORDER BY project_uid",
        )
        .map_err(|source| RegistryError::Sqlite {
            context: "cannot prepare project-registry query".into(),
            source,
        })?;
    statement
        .query_map([], row_to_project)
        .map_err(|source| RegistryError::Sqlite {
            context: "cannot query registered projects".into(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| RegistryError::Sqlite {
            context: "cannot decode registered projects".into(),
            source,
        })?
        .into_iter()
        .map(validate_project_invariants)
        .collect()
}

fn row_to_project(row: &Row<'_>) -> rusqlite::Result<RegisteredProject> {
    let project_uid_text: String = row.get(0)?;
    let desired_state_text: String = row.get(4)?;
    let observed_state_text: String = row.get(5)?;
    let child_pid: Option<i64> = row.get(6)?;
    let restart_count: i64 = row.get(10)?;
    let registration_revision: i64 = row.get(11)?;
    let updated_revision: i64 = row.get(12)?;

    Ok(RegisteredProject {
        project_uid: Uuid::parse_str(&project_uid_text).map_err(|error| {
            conversion_error(0, rusqlite::types::Type::Text, &project_uid_text, error)
        })?,
        canonical_repo_root: PathBuf::from(row.get::<_, String>(1)?),
        database_path: PathBuf::from(row.get::<_, String>(2)?),
        grpc_endpoint: row.get(3)?,
        desired_state: DesiredState::parse(&desired_state_text).map_err(|error| {
            conversion_error(4, rusqlite::types::Type::Text, &desired_state_text, error)
        })?,
        observed_state: ObservedState::parse(&observed_state_text).map_err(|error| {
            conversion_error(5, rusqlite::types::Type::Text, &observed_state_text, error)
        })?,
        child_pid: child_pid
            .map(u32::try_from)
            .transpose()
            .map_err(|error| {
                conversion_error(
                    6,
                    rusqlite::types::Type::Integer,
                    child_pid.unwrap_or_default(),
                    error,
                )
            })?,
        child_revision: row.get(7)?,
        last_started_at: parse_optional_datetime(row, 8)?,
        last_healthy_at: parse_optional_datetime(row, 9)?,
        restart_count: u32::try_from(restart_count).map_err(|error| {
            conversion_error(
                10,
                rusqlite::types::Type::Integer,
                restart_count,
                error,
            )
        })?,
        registration_revision: u64::try_from(registration_revision).map_err(|error| {
            conversion_error(
                11,
                rusqlite::types::Type::Integer,
                registration_revision,
                error,
            )
        })?,
        updated_revision: u64::try_from(updated_revision).map_err(|error| {
            conversion_error(
                12,
                rusqlite::types::Type::Integer,
                updated_revision,
                error,
            )
        })?,
        last_error: row.get(13)?,
    })
}

fn parse_optional_datetime(
    row: &Row<'_>,
    index: usize,
) -> rusqlite::Result<Option<DateTime<Utc>>> {
    let value: Option<String> = row.get(index)?;
    value
        .map(|text| {
            text.parse::<DateTime<Utc>>().map_err(|error| {
                conversion_error(index, rusqlite::types::Type::Text, text, error)
            })
        })
        .transpose()
}

fn conversion_error(
    index: usize,
    value_type: rusqlite::types::Type,
    value: impl std::fmt::Display,
    error: impl std::fmt::Display,
) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        index,
        value_type,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid value {value}: {error}"),
        )),
    )
}

fn validate_project_invariants(
    project: RegisteredProject,
) -> Result<RegisteredProject, RegistryError> {
    let expected_database = project
        .canonical_repo_root
        .join(".agileplus")
        .join("agileplus.db");
    if project.database_path != expected_database {
        return Err(RegistryError::Corrupt(format!(
            "project {} stores arbitrary database path {}; expected {}",
            project.project_uid,
            project.database_path.display(),
            expected_database.display()
        )));
    }
    Ok(project)
}
