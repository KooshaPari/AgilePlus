// SPDX-License-Identifier: MIT OR Apache-2.0
//! Atomic, versioned, fail-closed device registry storage.

use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior};
use uuid::Uuid;

use crate::model::{RegisteredProject, RegistryError, RegistrySummary};
use crate::schema::{INITIAL_SCHEMA, REGISTRY_FILE_NAME, SCHEMA_VERSION};
use crate::support::{
    get_project_from_connection, harden_registry_directory, harden_registry_permissions,
    list_projects_from_connection, prepare_registry_directory, read_revision,
    validate_existing_registry_file, validate_integrity,
};

const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

mod mutations;

/// Persistent device registry backed by one private SQLite database.
pub struct DeviceRegistry {
    pub(crate) path: PathBuf,
    pub(crate) connection: Connection,
}

impl DeviceRegistry {
    /// Open or initialize the registry at the platform-default path.
    ///
    /// # Errors
    ///
    /// Returns an error when the platform data directory is unavailable or the
    /// registry cannot be opened and validated.
    pub fn open_default() -> Result<Self, RegistryError> {
        let path = default_registry_path()?;
        prepare_registry_directory(&path)?;
        let parent = path
            .parent()
            .ok_or_else(|| RegistryError::Corrupt("default registry has no parent".into()))?;
        harden_registry_directory(parent)?;
        Self::open(path)
    }

    /// Open or initialize a registry at an explicit path.
    ///
    /// Existing malformed, incomplete, symlinked, or unsupported registries are
    /// rejected without replacement. A missing file is initialized atomically by
    /// SQLite transactions. Explicit registry paths never change permissions on
    /// an already existing parent directory.
    ///
    /// # Errors
    ///
    /// Returns an error for filesystem, SQLite, integrity, or schema failures.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RegistryError> {
        let path = path.as_ref().to_path_buf();
        let existed = path.exists();

        if existed {
            validate_existing_registry_file(&path)?;
        } else {
            prepare_registry_directory(&path)?;
        }

        let flags = if existed {
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX
        } else {
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
        };
        let connection = Connection::open_with_flags(&path, flags).map_err(|source| {
            RegistryError::Sqlite {
                context: format!("cannot open device registry {}", path.display()),
                source,
            }
        })?;
        connection
            .busy_timeout(SQLITE_BUSY_TIMEOUT)
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot configure device-registry busy timeout".into(),
                source,
            })?;
        connection
            .execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot configure device-registry durability".into(),
                source,
            })?;

        let mut registry = Self { path, connection };
        if existed {
            registry.validate_existing()?;
        } else {
            registry.initialize()?;
            harden_registry_permissions(&registry.path)?;
        }
        Ok(registry)
    }

    /// Return the backing registry path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return registry version, revision, and record count.
    ///
    /// # Errors
    ///
    /// Returns an error when metadata or project rows cannot be decoded.
    pub fn summary(&self) -> Result<RegistrySummary, RegistryError> {
        let revision = read_revision(&self.connection)?;
        let project_count: i64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM registered_projects", [], |row| {
                row.get(0)
            })
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot count registered projects".into(),
                source,
            })?;
        let project_count = usize::try_from(project_count)
            .map_err(|_| RegistryError::Corrupt("negative project count".into()))?;
        Ok(RegistrySummary {
            path: self.path.clone(),
            schema_version: SCHEMA_VERSION,
            revision,
            project_count,
        })
    }

    /// Look up one registered project by stable identity.
    ///
    /// # Errors
    ///
    /// Returns an error when the stored row is malformed or cannot be queried.
    pub fn get_project(
        &self,
        project_uid: Uuid,
    ) -> Result<Option<RegisteredProject>, RegistryError> {
        get_project_from_connection(&self.connection, project_uid)
    }

    /// List all registered projects in stable identity order.
    ///
    /// # Errors
    ///
    /// Returns an error when any stored row is malformed or cannot be queried.
    pub fn list_projects(&self) -> Result<Vec<RegisteredProject>, RegistryError> {
        list_projects_from_connection(&self.connection)
    }

    fn initialize(&mut self) -> Result<(), RegistryError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot begin device-registry initialization".into(),
                source,
            })?;
        transaction
            .execute_batch(INITIAL_SCHEMA)
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot create device-registry schema".into(),
                source,
            })?;
        let now = Utc::now().to_rfc3339();
        transaction
            .execute(
                "INSERT INTO registry_metadata (
                    singleton, schema_version, revision, created_at, updated_at
                 ) VALUES (1, ?1, 0, ?2, ?2)",
                rusqlite::params![SCHEMA_VERSION, now],
            )
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot write device-registry metadata".into(),
                source,
            })?;
        transaction
            .pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot set device-registry user version".into(),
                source,
            })?;
        transaction
            .commit()
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot commit device-registry initialization".into(),
                source,
            })?;
        self.validate_existing()
    }

    fn validate_existing(&self) -> Result<(), RegistryError> {
        validate_integrity(&self.connection)?;
        let user_version: u32 = self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot read device-registry user version".into(),
                source,
            })?;
        if user_version != SCHEMA_VERSION {
            return Err(RegistryError::UnsupportedSchema {
                expected: SCHEMA_VERSION,
                found: user_version,
            });
        }

        let (schema_version, revision): (u32, i64) = self
            .connection
            .query_row(
                "SELECT schema_version, revision FROM registry_metadata WHERE singleton=1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|source| RegistryError::Sqlite {
                context: "cannot read device-registry metadata".into(),
                source,
            })?
            .ok_or_else(|| RegistryError::Corrupt("registry metadata row is missing".into()))?;
        if schema_version != SCHEMA_VERSION {
            return Err(RegistryError::UnsupportedSchema {
                expected: SCHEMA_VERSION,
                found: schema_version,
            });
        }
        if revision < 0 {
            return Err(RegistryError::Corrupt("registry revision is negative".into()));
        }

        let projects = list_projects_from_connection(&self.connection)?;
        let revision = u64::try_from(revision)
            .map_err(|_| RegistryError::Corrupt("registry revision is negative".into()))?;
        for project in projects {
            if project.registration_revision == 0
                || project.updated_revision < project.registration_revision
                || project.updated_revision > revision
            {
                return Err(RegistryError::Corrupt(format!(
                    "project {} has invalid revision bounds",
                    project.project_uid
                )));
            }
        }
        Ok(())
    }
}

/// Return the platform-default device registry path.
///
/// # Errors
///
/// Returns an error when the platform cannot resolve a per-user local data
/// directory.
pub fn default_registry_path() -> Result<PathBuf, RegistryError> {
    dirs_next::data_local_dir()
        .map(|base| base.join("AgilePlus").join(REGISTRY_FILE_NAME))
        .ok_or(RegistryError::DataDirectoryUnavailable)
}
