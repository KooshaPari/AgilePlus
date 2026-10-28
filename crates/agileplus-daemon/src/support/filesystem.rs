// SPDX-License-Identifier: MIT OR Apache-2.0
//! Device-registry filesystem and Git-root validation.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::model::RegistryError;

pub(crate) fn prepare_registry_directory(path: &Path) -> Result<(), RegistryError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| RegistryError::Io {
        context: format!(
            "cannot create device-registry directory {}",
            parent.display()
        ),
        source,
    })?;
    let metadata = fs::metadata(parent).map_err(|source| RegistryError::Io {
        context: format!(
            "cannot inspect device-registry directory {}",
            parent.display()
        ),
        source,
    })?;
    if metadata.is_dir() {
        Ok(())
    } else {
        Err(RegistryError::Corrupt(format!(
            "registry parent {} is not a directory",
            parent.display()
        )))
    }
}

pub(crate) fn validate_existing_registry_file(path: &Path) -> Result<(), RegistryError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| RegistryError::Io {
        context: format!("cannot inspect device registry {}", path.display()),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(RegistryError::Corrupt(format!(
            "registry path {} is not a regular file",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(unix)]
pub(crate) fn harden_registry_directory(path: &Path) -> Result<(), RegistryError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|source| {
        RegistryError::Io {
            context: format!(
                "cannot restrict device-registry directory {}",
                path.display()
            ),
            source,
        }
    })
}

#[cfg(not(unix))]
pub(crate) fn harden_registry_directory(_path: &Path) -> Result<(), RegistryError> {
    Ok(())
}

#[cfg(unix)]
pub(crate) fn harden_registry_permissions(path: &Path) -> Result<(), RegistryError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|source| {
        RegistryError::Io {
            context: format!("cannot restrict device registry {}", path.display()),
            source,
        }
    })
}

#[cfg(not(unix))]
pub(crate) fn harden_registry_permissions(_path: &Path) -> Result<(), RegistryError> {
    Ok(())
}

pub(crate) fn canonical_git_root(project_root: &Path) -> Result<PathBuf, RegistryError> {
    let canonical = fs::canonicalize(project_root).map_err(|source| {
        RegistryError::InvalidProjectRoot {
            path: project_root.to_path_buf(),
            detail: source.to_string(),
        }
    })?;
    if !canonical.is_dir() {
        return Err(RegistryError::InvalidProjectRoot {
            path: canonical,
            detail: "path is not a directory".into(),
        });
    }

    let output = Command::new("git")
        .args(["-C"])
        .arg(&canonical)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|source| RegistryError::InvalidProjectRoot {
            path: canonical.clone(),
            detail: format!("cannot execute git root discovery: {source}"),
        })?;
    if !output.status.success() {
        return Err(RegistryError::InvalidProjectRoot {
            path: canonical,
            detail: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    let discovered = String::from_utf8(output.stdout).map_err(|error| {
        RegistryError::InvalidProjectRoot {
            path: canonical.clone(),
            detail: format!("git returned a non-UTF-8 worktree root: {error}"),
        }
    })?;
    let discovered = fs::canonicalize(discovered.trim()).map_err(|source| {
        RegistryError::InvalidProjectRoot {
            path: canonical.clone(),
            detail: format!("cannot canonicalize discovered Git root: {source}"),
        }
    })?;
    if discovered != canonical {
        return Err(RegistryError::InvalidProjectRoot {
            path: canonical,
            detail: format!(
                "path resolves inside Git worktree {} rather than to its root",
                discovered.display()
            ),
        });
    }
    Ok(discovered)
}

pub(crate) fn path_text(path: &Path) -> Result<String, RegistryError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| RegistryError::NonUtf8Path(path.to_path_buf()))
}
