// SPDX-License-Identifier: MIT OR Apache-2.0

mod database;
mod filesystem;
mod rows;

pub(crate) use database::{bump_revision, read_revision, validate_integrity};
pub(crate) use filesystem::{
    canonical_git_root, harden_registry_permissions, path_text, prepare_registry_directory,
    validate_existing_registry_file,
};
pub(crate) use rows::{get_project_from_connection, list_projects_from_connection};
