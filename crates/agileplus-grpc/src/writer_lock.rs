// SPDX-License-Identifier: MIT OR Apache-2.0
//! Crash-recoverable ownership record for one canonical AgilePlus core writer.
//!
//! This is deliberately a project-core lock, not yet a universal SQLite lock.
//! Direct CLI writers must adopt the same boundary in a later migration slice.

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const LOCK_VERSION: u32 = 2;
const LOCK_FILE_NAME: &str = "core-writer.lock";
const GUARD_FILE_NAME: &str = "core-writer.guard";
const MAX_ACQUIRE_ATTEMPTS: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct LockOwner {
    version: u32,
    pid: u32,
    executable: Option<String>,
    token: String,
    acquired_at_unix_nanos: u128,
    canonical_repo_root: String,
}

/// Holds exclusive ownership of one project's canonical gRPC core process.
///
/// A persistent kernel-locked guard serializes acquisition, recovery and cleanup.
/// The guard inode is never renamed or deleted. If a process crashes, a later
/// contender checks whether the recorded PID still exists before quarantining a
/// stale record and retrying. PID reuse is intentionally conservative: a reused
/// live PID blocks automatic recovery rather than allowing ownership theft.
pub struct ProjectWriterLock {
    path: PathBuf,
    token: String,
    file: Option<File>,
    // Keep the kernel lock through owner-record cleanup. Closing releases it on crash.
    _guard: File,
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
        let guard = acquire_guard(&state_dir)?;

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
                        _guard: guard,
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

fn acquire_guard(state_dir: &Path) -> Result<File, String> {
    let path = state_dir.join(GUARD_FILE_NAME);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
            return Err("core writer guard is not a regular file; refusing ownership".into());
        }
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(format!("cannot inspect core writer guard: {error}")),
    }
    let guard = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|error| format!("cannot open core writer guard: {error}"))?;
    fs2::FileExt::try_lock_exclusive(&guard).map_err(|error| {
        if error.kind() == ErrorKind::WouldBlock
            || error.raw_os_error() == fs2::lock_contended_error().raw_os_error()
        {
            "AgilePlus project already has a live core writer or recovery contender".into()
        } else {
            format!("cannot lock core writer guard; refusing ownership: {error}")
        }
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let opened = guard
            .metadata()
            .map_err(|error| format!("cannot inspect open guard: {error}"))?;
        let current = fs::symlink_metadata(&path)
            .map_err(|error| format!("cannot inspect guard path: {error}"))?;
        if current.file_type().is_symlink()
            || opened.dev() != current.dev()
            || opened.ino() != current.ino()
        {
            return Err("core writer guard path changed; refusing ownership".into());
        }
    }
    Ok(guard)
}

impl Drop for ProjectWriterLock {
    fn drop(&mut self) {
        drop(self.file.take());
        if let Ok(contents) = fs::read_to_string(&self.path)
            && let Ok(owner) = serde_json::from_str::<LockOwner>(&contents)
            && owner.token == self.token
        {
            let _ = fs::remove_file(&self.path);
        }
        // Closing our descriptor alone need not release flock while a child
        // fork or duplicate retains the same open file description. Release
        // authority explicitly, after cleanup and before closing the guard.
        // Closing alone may leave the lock held by a forked/duplicated descriptor.
        // Release only after owner-record cleanup, while the guard is still held.
        let _ = fs2::FileExt::unlock(&self._guard);
    }
}

impl LockOwner {
    fn for_current_process(canonical_repo_root: &Path) -> Result<Self, String> {
        let pid = std::process::id();
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
            executable,
            token: format!("{pid}:{acquired_at_unix_nanos}"),
            acquired_at_unix_nanos,
            canonical_repo_root,
        })
    }
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
    // Version 1 is a legacy PID-only owner: probe it before migration. New
    // records use version 2 so legacy binaries cannot recover a guarded owner.
    if owner.version != 1 && owner.version != LOCK_VERSION {
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
    if process_is_live(owner.pid)? {
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

#[cfg(unix)]
fn process_is_live(pid: u32) -> Result<bool, String> {
    let pid_text = pid.to_string();
    let output = Command::new("ps")
        .args(["-p", &pid_text, "-o", "pid="])
        .output()
        .map_err(|error| format!("cannot probe lock owner pid {pid} with ps: {error}"))?;
    if output.status.code() == Some(1) && output.stdout.is_empty() && output.stderr.is_empty() {
        return Ok(false); // procps reports a verified absent PID this way.
    }
    if !output.status.success() {
        return Err(format!(
            "ps failed while probing lock owner pid {pid}; refusing recovery: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim() == pid_text {
        Ok(true)
    } else {
        Err(format!(
            "ps returned ambiguous output for lock owner pid {pid}; refusing recovery"
        ))
    }
}

#[cfg(windows)]
fn process_is_live(pid: u32) -> Result<bool, String> {
    let filter = format!("PID eq {pid}");
    let output = Command::new("tasklist")
        .args(["/FI", &filter, "/FO", "CSV", "/NH"])
        .output()
        .map_err(|error| format!("cannot probe lock owner pid {pid} with tasklist: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "tasklist failed while probing lock owner pid {pid}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let quoted_pid = format!("\"{pid}\"");
    Ok(String::from_utf8_lossy(&output.stdout).contains(&quoted_pid))
}

#[cfg(not(any(unix, windows)))]
fn process_is_live(pid: u32) -> Result<bool, String> {
    Err(format!(
        "cannot safely recover AgilePlus core writer lock for pid {pid} on this platform"
    ))
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
    fn owner_release_unlocks_a_descriptor_inherited_during_spawn() {
        let scratch = Scratch::new("inherited-descriptor");
        let first = ProjectWriterLock::acquire(scratch.path()).expect("first owner");
        // A fork can temporarily retain this open file description before exec
        // closes CLOEXEC descriptors. Keep a duplicate to deterministically model it.
        let inherited = first
            ._guard
            .try_clone()
            .expect("duplicate guard descriptor");
        drop(first);
        let replacement = ProjectWriterLock::acquire(scratch.path())
            .expect("owner release must explicitly unlock despite an inherited descriptor");
        assert!(replacement.path().exists());
        drop(inherited);
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
            pid: i32::MAX as u32,
            executable: Some("/definitely/not/a/process".into()),
            token: "stale".into(),
            acquired_at_unix_nanos: 1,
            canonical_repo_root: canonical.to_string_lossy().into_owned(),
        };
        fs::write(&path, serde_json::to_vec(&stale).unwrap()).unwrap();

        let recovered = ProjectWriterLock::acquire(scratch.path()).expect("recover stale lock");
        assert!(recovered.path().exists());
        let quarantined = fs::read_dir(&state).unwrap().flatten().any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("core-writer.lock.stale.")
        });
        assert!(
            quarantined,
            "stale ownership record must remain as evidence"
        );
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
