//! Real-process counterexamples for lock recovery; every root and child is disposable.
#![cfg(unix)]

use agileplus_grpc::writer_lock::ProjectWriterLock;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct Scratch(PathBuf);
impl Scratch {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "agileplus-lock-regression-{name}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(path.join(".agileplus")).unwrap();
        fs::create_dir_all(path.join("bin")).unwrap();
        let script = path.join("bin/ps");
        fs::write(
            &script,
            r#"#!/bin/sh
if test "$LOCK_TEST_PROBE" = error; then
  printf 'process probe unavailable\n' >&2
  exit 2
fi
if test "$LOCK_TEST_ACTOR" = A; then
  touch "$LOCK_TEST_ROOT/probed-A"
  until test -f "$LOCK_TEST_ROOT/allow-A"; do sleep .01; done
fi
exit 1
"#,
        )
        .unwrap();
        fs::set_permissions(script, fs::Permissions::from_mode(0o700)).unwrap();
        Self(fs::canonicalize(path).unwrap())
    }
    fn owner(&self, pid: u32) -> Vec<u8> {
        let bytes = serde_json::to_vec(&serde_json::json!({
            "version": 1, "pid": pid, "executable": "/legacy/core", "token": "legacy-owner",
            "acquired_at_unix_nanos": 1, "canonical_repo_root": self.0.to_str().unwrap()
        }))
        .unwrap();
        fs::write(self.0.join(".agileplus/core-writer.lock"), &bytes).unwrap();
        bytes
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Probe {
    child: Child,
    root: PathBuf,
    actor: &'static str,
}
impl Probe {
    fn spawn(root: &Path, actor: &'static str, mode: &str) -> Self {
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "ownership_probe_child", "--nocapture"])
            .env("LOCK_TEST_ROOT", root)
            .env("LOCK_TEST_ACTOR", actor)
            .env("LOCK_TEST_PROBE", mode)
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    root.join("bin").display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        Self {
            child,
            root: root.to_path_buf(),
            actor,
        }
    }
    fn wait(&mut self) -> std::process::ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status;
            }
            assert!(Instant::now() < deadline, "contender did not exit");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        let _ = fs::write(self.root.join("allow-A"), b"");
        let _ = fs::write(self.root.join(format!("release-{}", self.actor)), b"");
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn wait_for(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "missing probe barrier: {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn ownership_probe_child() {
    let Some(root) = std::env::var_os("LOCK_TEST_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    let actor = std::env::var("LOCK_TEST_ACTOR").unwrap();
    match ProjectWriterLock::acquire(&root) {
        Ok(guard) => {
            fs::write(root.join(format!("acquired-{actor}")), b"").unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            while !root.join(format!("release-{actor}")).exists() {
                if actor == "B" {
                    break;
                }
                assert!(Instant::now() < deadline, "release timeout");
                std::thread::sleep(Duration::from_millis(10));
            }
            drop(guard);
        }
        Err(error) => {
            fs::write(root.join(format!("rejected-{actor}")), error).unwrap();
            std::process::exit(2);
        }
    }
}

#[test]
fn concurrent_recovery_cannot_take_over_an_in_progress_owner() {
    let scratch = Scratch::new("concurrent");
    scratch.owner(i32::MAX as u32);
    let mut first = Probe::spawn(&scratch.0, "A", "dead");
    wait_for(&scratch.0.join("probed-A"));
    let mut second = Probe::spawn(&scratch.0, "B", "dead");
    let contender_status = second.wait();
    fs::write(scratch.0.join("allow-A"), b"").unwrap();
    wait_for(&scratch.0.join("acquired-A"));
    fs::write(scratch.0.join("release-A"), b"").unwrap();
    assert!(first.wait().success());
    assert!(
        !contender_status.success(),
        "recovery must be exclusive before inspecting an old owner"
    );
    assert!(!scratch.0.join("acquired-B").exists());
}

#[test]
fn failed_owner_probe_preserves_legacy_ownership() {
    let scratch = Scratch::new("probe-error");
    let original = scratch.owner(std::process::id());
    let mut contender = Probe::spawn(&scratch.0, "B", "error");
    let status = contender.wait();
    assert!(
        !status.success(),
        "a failed probe must never authorize takeover"
    );
    assert_eq!(
        fs::read(scratch.0.join(".agileplus/core-writer.lock")).unwrap(),
        original
    );
}

#[test]
fn abrupt_owner_exit_releases_guard_and_quarantines_its_record() {
    let scratch = Scratch::new("crash");
    let mut first = Probe::spawn(&scratch.0, "A", "dead");
    wait_for(&scratch.0.join("acquired-A"));
    let original = fs::read(scratch.0.join(".agileplus/core-writer.lock")).unwrap();
    first.child.kill().unwrap();
    first.child.wait().unwrap();
    let mut replacement = Probe::spawn(&scratch.0, "B", "dead");
    assert!(
        replacement.wait().success(),
        "OS guard must release after a crash"
    );
    assert!(scratch.0.join("acquired-B").exists());
    assert!(
        fs::read_dir(scratch.0.join(".agileplus"))
            .unwrap()
            .flatten()
            .any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("core-writer.lock.stale.")
                    && fs::read(entry.path()).unwrap() == original
            }),
        "crashed ownership must remain as evidence"
    );
}
