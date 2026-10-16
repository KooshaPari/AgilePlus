// SPDX-License-Identifier: MIT OR Apache-2.0

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use rusqlite::Connection;
use tempfile::TempDir;
use uuid::Uuid;

use crate::model::{DesiredState, ObservedState, RegistryError, RegistrySummary};
use crate::schema::{REGISTRY_FILE_NAME, SCHEMA_VERSION};
use crate::store::DeviceRegistry;

fn git_project(name: &str) -> TempDir {
    let directory = tempfile::Builder::new()
        .prefix(&format!("agileplus-registry-{name}-"))
        .tempdir()
        .expect("create test project");
    let output = Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(directory.path())
        .output()
        .expect("run git init");
    assert!(output.status.success(), "git init failed");
    directory
}

fn registry_path(directory: &TempDir) -> PathBuf {
    directory.path().join("state").join(REGISTRY_FILE_NAME)
}

#[test]
fn new_registry_is_versioned_and_empty() {
    let directory = tempfile::tempdir().unwrap();
    let registry = DeviceRegistry::open(registry_path(&directory)).unwrap();
    assert_eq!(
        registry.summary().unwrap(),
        RegistrySummary {
            path: registry_path(&directory),
            schema_version: SCHEMA_VERSION,
            revision: 0,
            project_count: 0,
        }
    );
}

#[test]
fn generated_project_identity_is_uuid_v7() {
    let directory = tempfile::tempdir().unwrap();
    let project = git_project("uuid-v7");
    let mut registry = DeviceRegistry::open(registry_path(&directory)).unwrap();

    let registered = registry.register_project(project.path()).unwrap();
    assert_eq!(registered.project_uid.as_bytes()[6] >> 4, 7);
}

#[test]
fn register_list_and_desired_state_are_revisioned() {
    let directory = tempfile::tempdir().unwrap();
    let project = git_project("register");
    let mut registry = DeviceRegistry::open(registry_path(&directory)).unwrap();
    let project_uid = Uuid::parse_str("0199f2a5-4a64-7f90-8000-000000000001").unwrap();

    let registered = registry
        .register_project_with_uid(project_uid, project.path())
        .unwrap();
    assert_eq!(registered.project_uid, project_uid);
    assert_eq!(registered.desired_state, DesiredState::Stopped);
    assert_eq!(registered.observed_state, ObservedState::Stopped);
    assert_eq!(registered.registration_revision, 1);
    assert_eq!(registered.updated_revision, 1);
    assert_eq!(
        registered.database_path,
        registered
            .canonical_repo_root
            .join(".agileplus")
            .join("agileplus.db")
    );

    let running = registry
        .set_desired_state(project_uid, DesiredState::Running)
        .unwrap();
    assert_eq!(running.desired_state, DesiredState::Running);
    assert_eq!(running.registration_revision, 1);
    assert_eq!(running.updated_revision, 2);
    assert_eq!(registry.summary().unwrap().revision, 2);
    assert_eq!(registry.list_projects().unwrap(), vec![running]);
}

#[test]
fn duplicate_root_and_uid_collisions_fail_closed() {
    let directory = tempfile::tempdir().unwrap();
    let first = git_project("first");
    let second = git_project("second");
    let mut registry = DeviceRegistry::open(registry_path(&directory)).unwrap();
    let uid = Uuid::new_v4();
    registry
        .register_project_with_uid(uid, first.path())
        .unwrap();

    let duplicate_root = registry.register_project(first.path()).unwrap_err();
    assert!(matches!(duplicate_root, RegistryError::DuplicateRoot { .. }));
    let duplicate_uid = registry
        .register_project_with_uid(uid, second.path())
        .unwrap_err();
    assert!(matches!(duplicate_uid, RegistryError::UidCollision(_)));
    assert_eq!(registry.summary().unwrap().revision, 1);
}

#[cfg(unix)]
#[test]
fn symlink_alias_is_rejected_as_duplicate_root() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let project = git_project("alias");
    let alias = directory.path().join("project-alias");
    symlink(project.path(), &alias).unwrap();
    let mut registry = DeviceRegistry::open(registry_path(&directory)).unwrap();
    registry.register_project(project.path()).unwrap();

    let error = registry.register_project(&alias).unwrap_err();
    assert!(matches!(error, RegistryError::DuplicateRoot { .. }));
}

#[cfg(unix)]
#[test]
fn explicit_registry_path_does_not_chmod_existing_parent() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let shared_parent = directory.path().join("shared-parent");
    fs::create_dir(&shared_parent).unwrap();
    fs::set_permissions(&shared_parent, fs::Permissions::from_mode(0o755)).unwrap();

    let path = shared_parent.join(REGISTRY_FILE_NAME);
    drop(DeviceRegistry::open(path).unwrap());

    let mode = fs::metadata(shared_parent).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o755);
}

#[test]
fn unregister_does_not_delete_project_local_state() {
    let directory = tempfile::tempdir().unwrap();
    let project = git_project("preserve");
    let state_dir = project.path().join(".agileplus");
    fs::create_dir_all(&state_dir).unwrap();
    let database = state_dir.join("agileplus.db");
    fs::write(&database, b"project-owned-state").unwrap();

    let mut registry = DeviceRegistry::open(registry_path(&directory)).unwrap();
    let registered = registry.register_project(project.path()).unwrap();
    let removed = registry
        .unregister_project(registered.project_uid)
        .unwrap();
    assert_eq!(removed.project_uid, registered.project_uid);
    assert!(database.exists());
    assert_eq!(fs::read(database).unwrap(), b"project-owned-state");
    assert_eq!(registry.summary().unwrap().revision, 2);
    assert!(registry.list_projects().unwrap().is_empty());
}

#[test]
fn non_git_or_nested_paths_are_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let mut registry = DeviceRegistry::open(registry_path(&directory)).unwrap();
    let error = registry.register_project(directory.path()).unwrap_err();
    assert!(matches!(error, RegistryError::InvalidProjectRoot { .. }));

    let project = git_project("nested");
    let nested = project.path().join("src");
    fs::create_dir_all(&nested).unwrap();
    let error = registry.register_project(&nested).unwrap_err();
    assert!(matches!(error, RegistryError::InvalidProjectRoot { .. }));
}

#[test]
fn damaged_existing_registry_is_not_reinitialized() {
    let directory = tempfile::tempdir().unwrap();
    let path = registry_path(&directory);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"not a sqlite database").unwrap();

    assert!(DeviceRegistry::open(&path).is_err());
    assert_eq!(fs::read(path).unwrap(), b"not a sqlite database");
}

#[test]
fn unsupported_schema_version_is_rejected_without_rewrite() {
    let directory = tempfile::tempdir().unwrap();
    let path = registry_path(&directory);
    drop(DeviceRegistry::open(&path).unwrap());
    let connection = Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 999).unwrap();
    connection
        .execute(
            "UPDATE registry_metadata SET schema_version=999 WHERE singleton=1",
            [],
        )
        .unwrap();
    drop(connection);

    let error = DeviceRegistry::open(&path)
        .err()
        .expect("unsupported registry must fail");
    assert!(matches!(
        error,
        RegistryError::UnsupportedSchema { found: 999, .. }
    ));
    let connection = Connection::open(path).unwrap();
    let version: u32 = connection
        .query_row(
            "SELECT schema_version FROM registry_metadata WHERE singleton=1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, 999);
}

#[test]
fn arbitrary_database_path_is_detected_on_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let project = git_project("tamper");
    let path = registry_path(&directory);
    let mut registry = DeviceRegistry::open(&path).unwrap();
    let registered = registry.register_project(project.path()).unwrap();
    drop(registry);

    let connection = Connection::open(&path).unwrap();
    connection
        .execute(
            "UPDATE registered_projects SET database_path='/tmp/foreign.db' WHERE project_uid=?1",
            [registered.project_uid.to_string()],
        )
        .unwrap();
    drop(connection);

    assert!(matches!(
        DeviceRegistry::open(path)
            .err()
            .expect("tampered registry must fail"),
        RegistryError::Corrupt(_)
    ));
}
