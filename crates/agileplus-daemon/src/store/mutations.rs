// SPDX-License-Identifier: MIT OR Apache-2.0
//! Versioned project-registration mutations.

use std::path::Path;

use chrono::Utc;
use rusqlite::{OptionalExtension, TransactionBehavior};
use uuid::Uuid;

use super::DeviceRegistry;
use crate::model::{DesiredState, RegisteredProject, RegistryError};
use crate::support::{
    bump_revision, canonical_git_root, get_project_from_connection, path_text,
};

impl DeviceRegistry {
    /// Register a canonical Git project with a generated stable identity.
    ///
    /// # Errors
    ///
    /// Returns an error when the project root is invalid, duplicates an existing
    /// canonical root, or the transaction cannot be committed.
    pub fn register_project(
        &mut self,
        project_root: &Path,
    ) -> Result<RegisteredProject, RegistryError> {
        self.register_project_with_uid(Uuid::now_v7(), project_root)
    }

    /// Register a canonical Git project with a caller-provided stable identity.
    ///
    /// This entry point exists for controlled imports and deterministic tests.
    /// It never overwrites an existing identity or root.
    ///
    /// # Errors
    ///
    /// Returns an error when the root or identity conflicts with registry state,
    /// or when the transaction cannot be committed.
    pub fn register_project_with_uid(
        &mut self,
        project_uid: Uuid,
        project_root: &Path,
    ) -> Result<RegisteredProject, RegistryError> {
        let canonical_repo_root = canonical_git_root(project_root)?;
        let database_path = canonical_repo_root
            .join(".agileplus")
            .join("agileplus.db");
        let root_text = path_text(&canonical_repo_root)?;
        let database_text = path_text(&database_path)?;
        let project_uid_text = project_uid.to_string();
        let now = Utc::now();

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot begin project-registration transaction".into(),
                source,
            })?;

        if let Some(existing_uid) = transaction
            .query_row(
                "SELECT project_uid FROM registered_projects WHERE canonical_repo_root=?1",
                [&root_text],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot check duplicate project root".into(),
                source,
            })?
        {
            let existing_uid = Uuid::parse_str(&existing_uid).map_err(|error| {
                RegistryError::Corrupt(format!(
                    "stored project UID {existing_uid:?} is invalid: {error}"
                ))
            })?;
            return Err(RegistryError::DuplicateRoot {
                path: canonical_repo_root,
                existing_uid,
            });
        }

        let uid_exists: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM registered_projects WHERE project_uid=?1)",
                [&project_uid_text],
                |row| row.get(0),
            )
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot check project UID collision".into(),
                source,
            })?;
        if uid_exists {
            return Err(RegistryError::UidCollision(project_uid));
        }

        let revision = bump_revision(&transaction, now)?;
        transaction
            .execute(
                "INSERT INTO registered_projects (
                    project_uid, canonical_repo_root, database_path, grpc_endpoint,
                    desired_state, observed_state, child_pid, child_revision,
                    last_started_at, last_healthy_at, restart_count,
                    registration_revision, updated_revision, last_error
                 ) VALUES (?1, ?2, ?3, NULL, 'stopped', 'stopped', NULL, NULL,
                           NULL, NULL, 0, ?4, ?4, NULL)",
                rusqlite::params![project_uid_text, root_text, database_text, revision],
            )
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot insert project registry record".into(),
                source,
            })?;
        transaction
            .commit()
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot commit project registration".into(),
                source,
            })?;

        self.get_project(project_uid)?
            .ok_or_else(|| RegistryError::Corrupt("committed project record disappeared".into()))
    }

    /// Remove one registry entry without deleting repository or project-local data.
    ///
    /// # Errors
    ///
    /// Returns an error when the identity is unknown or the transaction fails.
    pub fn unregister_project(
        &mut self,
        project_uid: Uuid,
    ) -> Result<RegisteredProject, RegistryError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot begin project-unregistration transaction".into(),
                source,
            })?;
        let project = get_project_from_connection(&transaction, project_uid)?
            .ok_or(RegistryError::ProjectNotFound(project_uid))?;
        let changed = transaction
            .execute(
                "DELETE FROM registered_projects WHERE project_uid=?1",
                [project_uid.to_string()],
            )
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot remove project registry record".into(),
                source,
            })?;
        if changed != 1 {
            return Err(RegistryError::Corrupt(format!(
                "unregister changed {changed} rows for {project_uid}"
            )));
        }
        bump_revision(&transaction, Utc::now())?;
        transaction
            .commit()
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot commit project unregistration".into(),
                source,
            })?;
        Ok(project)
    }

    /// Change the operator-requested state for one registered project.
    ///
    /// # Errors
    ///
    /// Returns an error when the identity is unknown or the transaction fails.
    pub fn set_desired_state(
        &mut self,
        project_uid: Uuid,
        desired_state: DesiredState,
    ) -> Result<RegisteredProject, RegistryError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot begin desired-state transaction".into(),
                source,
            })?;
        if get_project_from_connection(&transaction, project_uid)?.is_none() {
            return Err(RegistryError::ProjectNotFound(project_uid));
        }
        let revision = bump_revision(&transaction, Utc::now())?;
        let changed = transaction
            .execute(
                "UPDATE registered_projects
                 SET desired_state=?1, updated_revision=?2
                 WHERE project_uid=?3",
                rusqlite::params![desired_state.as_str(), revision, project_uid.to_string()],
            )
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot update desired project state".into(),
                source,
            })?;
        if changed != 1 {
            return Err(RegistryError::Corrupt(format!(
                "desired-state update changed {changed} rows for {project_uid}"
            )));
        }
        transaction
            .commit()
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot commit desired project state".into(),
                source,
            })?;
        self.get_project(project_uid)?
            .ok_or_else(|| RegistryError::Corrupt("updated project record disappeared".into()))
    }
}
