// SPDX-License-Identifier: MIT OR Apache-2.0
//! Crash-recoverable ownership record for one canonical AgilePlus core writer.
//!
//! This is deliberately a project-core lock, not yet a universal SQLite lock.
//! Direct CLI writers must adopt the same boundary in a later migration slice.

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sysinfo::{Pid, System};

const LOCK_VERSION: u32 = 1;
const LOCK_FILE_NAME: &str = "core-writer.lock";
const MAX_ACQUIRE_ATTEMPTS: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct LockOwner {
    version: u32,
    pid: u32,
    process_start_time: u64,
    executable: Option<String>,
    token: String,
    acquired_at_unix_nanos: u128,
    canonical_repo_root: String,
}

/// Holds exclusive ownership of one project's canonical gRPC core process.
///
/// `create_new` provides the atomic winner selection. If a process crashes, a
/// later contender verifies the recorded PID, process start time, and executable
/// before quarantining a stale record and retrying.
pub struct ProjectWriterLock {
    path: PathBuf,
    token: String,
    file: Option<File>,
}

impl ProjectWriterLock {
    pub fn acquire(project_root: &Path) -> Result<Self, String> {
        let canonical_repo_root = fs::canonicalize(project_root).map_err(|error| {
            format!(
                "cannot canonicalize AgilePlus project root {}: {error}",
                project_root.display()
            )
        })?;
        let state_dir = canonical_repo_root.join(".agileplus");
        fs::create_dir_all(&state_dir).map_err(|error| {
            format!(
                "cannot create AgilePlus state directory {}: {error}",
                state_dir.display()
            )
        })?;
        let path = state_dir.join(LOCK_FILE_NAME);

        for _ in 0..MAX_ACQUIRE_ATTEMPTS {
            match OpenOptions::new()
                .create_new(true)
                .read(true)
                .write(true)
                .open(&path)
            {
                Ok(mut file) => {
                    let owner = LockOwner::for_current_process(&canonical_repo_root)?;
                    if let Err(error) = write_owner(&mut file, &owner) {
                        drop(file);
                        let _ = fs::remove_file(&path);
                        return Err(error);
                    }
                    return Ok(Self {
                        path,
                        token: owner.token,
                        file: Some(file),
                    });
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    if quarantine_if_stale(&path, &canonical_repo_root)? {
                        continue;
                    }
                    return Err(active_owner_error(&path)?);
                }
                Err(error) => {
                    return Err(format!(
                        "cannot create AgilePlus core writer lock {}: {error}",
                        path.display()
                    ));
                }
            }
        }

        Err(format!(
            "could not acquire AgilePlus core writer lock {} after concurrent recovery attempts",
            path.display()
        ))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ProjectWriterLock {
    fn drop(&mut self) {
        drop(self.file.take());
        let Ok(contents) = fs::read_to_string(&self.path) else {
            return;
        };
        let Ok(owner) = serde_json::from_str::<LockOwner>(&contents) else {
            return;
        };
        if owner.token == self.token {
            let _ = fs::remove_file(&self.path);
        }
    }
}

impl LockOwner {
    fn for_current_process(canonical_repo_root: &Path) -> Result<Self, String> {
        let pid = std::process::id();
        let process_start_time = process_identity(pid)
            .map(|identity| identity.start_time)
            .unwrap_or_default();
        let executable = std::env::current_exe()
            .ok()
            .and_then(|path| canonical_path_string(&path));
        let acquired_at_unix_nanos = unix_nanos()?;
        let canonical_repo_root = canonical_repo_root
            .to_str()
            .ok_or_else(|| "AgilePlus project root is not valid UTF-8".to_owned())?
            .to_owned();

        Ok(Self {
            version: LOCK_VERSION,
            pid,
            process_start_time,
            executable,
            token: format!("{pid}:{process_start_time}:{acquired_at_unix_nanos}"),
            acquired_at_unix_nanos,
            canonical_repo_root,
        })
    }
}

#[derive(Debug)]
struct ProcessIdentity {
    start_time: u64,
    executable: Option<String>,
}

fn process_identity(pid: u32) -> Option<ProcessIdentity> {
    let mut system = System::new_all();
    system.refresh_all();
    let process = system.process(Pid::from_u32(pid))?;
    Some(ProcessIdentity {
        start_time: process.start_time(),
        executable: process.exe().and_then(canonical_path_string),
    })
}

fn canonical_path_string(path: &Path) -> Option<String> {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_str()
        .map(str::to_owned)
}

fn write_owner(file: &mut File, owner: &LockOwner) -> Result<(), String> {
    let payload = serde_json::to_vec_pretty(owner)
        .map_err(|error| format!("cannot serialize AgilePlus core writer lock: {error}"))?;
    file.write_all(&payload)
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("cannot persist AgilePlus core writer lock: {error}"))
}

fn read_owner(path: &Path) -> Result<LockOwner, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "cannot inspect AgilePlus core writer lock {}: {error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "AgilePlus core writer lock {} is not a regular file; refusing recovery",
            path.display()
        ));
    }

    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "cannot read AgilePlus core writer lock {}: {error}",
            path.display()
        )
    })?;
    let owner: LockOwner = serde_json::from_str(&contents).map_err(|error| {
        format!(
            "AgilePlus core writer lock {} is malformed ({error}); refusing destructive recovery",
            path.display()
        )
    })?;
    if owner.version != LOCK_VERSION {
        return Err(format!(
            "AgilePlus core writer lock {} uses unsupported version {}; refusing recovery",
            path.display(),
            owner.version
        ));
    }
    Ok(owner)
}

fn active_owner_error(path: &Path) -> Result<String, String> {
    let owner = read_owner(path)?;
    Ok(format!(
        "AgilePlus project already has a live core writer (pid {}, acquired at {}); lock: {}",
        owner.pid,
        owner.acquired_at_unix_nanos,
        path.display()
    ))
}

fn quarantine_if_stale(path: &Path, canonical_repo_root: &Path) -> Result<bool, String> {
    let owner = read_owner(path)?;
    let expected_root = canonical_repo_root
        .to_str()
        .ok_or_else(|| "AgilePlus project root is not valid UTF-8".to_owned())?;
    if owner.canonical_repo_root != expected_root {
        return Err(format!(
            "AgilePlus core writer lock {} names a different project root; refusing recovery",
            path.display()
        ));
    }
    if owner_is_live(&owner) {
        return Ok(false);
    }

    let quarantine = path.with_file_name(format!(
        "{LOCK_FILE_NAME}.stale.{}.{}",
        owner.pid,
        unix_nanos()?
    ));
    match fs::rename(path, &quarantine) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(true),
        Err(error) => Err(format!(
            "cannot quarantine stale AgilePlus core writer lock {} to {}: {error}",
            path.display(),
            quarantine.display()
        )),
    }
}

fn owner_is_live(owner: &LockOwner) -> bool {
    let Some(identity) = process_identity(owner.pid) else {
        return false;
    };
    if owner.process_start_time != 0
        && identity.start_time != 0
        && owner.process_start_time != identity.start_time
    {
        return false;
    }
    if let (Some(expected), Some(observed)) = (&owner.executable, &identity.executable)
        && expected != observed
    {
        return false;
    }
    true
}

fn unix_nanos() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .map_err(|error| format!("system clock is before UNIX epoch: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = std::env::temp_dir().join(format!(
                "agileplus-writer-lock-{name}-{}-{suffix}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create scratch project root");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn exactly_one_live_core_owns_a_project() {
        let scratch = Scratch::new("exclusive");
        let first = ProjectWriterLock::acquire(scratch.path()).expect("first writer lock");
        let error = ProjectWriterLock::acquire(scratch.path())
            .err()
            .expect("second writer must be rejected");
        assert!(error.contains("live core writer"));
        assert!(first.path().exists());

        drop(first);
        let replacement =
            ProjectWriterLock::acquire(scratch.path()).expect("lock released after drop");
        assert!(replacement.path().exists());
    }

    #[test]
    fn stale_owner_is_quarantined_before_recovery() {
        let scratch = Scratch::new("stale");
        let canonical = fs::canonicalize(scratch.path()).unwrap();
        let state = canonical.join(".agileplus");
        fs::create_dir_all(&state).unwrap();
        let path = state.join(LOCK_FILE_NAME);
        let stale = LockOwner {
            version: LOCK_VERSION,
            pid: u32::MAX,
            process_start_time: 1,
            executable: Some("/definitely/not/a/process".into()),
            token: "stale".into(),
            acquired_at_unix_nanos: 1,
            canonical_repo_root: canonical.to_string_lossy().into_owned(),
        };
        fs::write(&path, serde_json::to_vec(&stale).unwrap()).unwrap();

        let recovered = ProjectWriterLock::acquire(scratch.path()).expect("recover stale lock");
        assert!(recovered.path().exists());
        let quarantined = fs::read_dir(&state)
            .unwrap()
            .flatten()
            .any(|entry| entry.file_name().to_string_lossy().starts_with("core-writer.lock.stale."));
        assert!(quarantined, "stale ownership record must remain as evidence");
    }

    #[test]
    fn malformed_lock_fails_closed() {
        let scratch = Scratch::new("malformed");
        let state = scratch.path().join(".agileplus");
        fs::create_dir_all(&state).unwrap();
        fs::write(state.join(LOCK_FILE_NAME), b"not-json").unwrap();

        let error = ProjectWriterLock::acquire(scratch.path())
            .err()
            .expect("malformed ownership must not be deleted");
        assert!(error.contains("malformed"));
        assert!(state.join(LOCK_FILE_NAME).exists());
    }
}
