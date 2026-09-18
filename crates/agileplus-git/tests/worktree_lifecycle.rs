//! Worktree behavior for the `VcsPort` implementation.
//!
//! The happy path (create -> list -> cleanup) is covered elsewhere; this
//! file pins the boundary behavior: occupied target paths, unregistered
//! or already-vanished worktrees, and the listing shape for repos with
//! no commits yet and for detached-HEAD worktrees.
//!
//! Worktrees are created as siblings of the temp repo, so each test uses
//! a process-unique slug and removes what it created.

use agileplus_domain::ports::VcsPort;
use agileplus_git::GitVcsAdapter;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const ZERO_SHA: &str = "0000000000000000000000000000000000000000";

fn git(dir: &Path, args: &[&str]) -> std::process::Output {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("spawn git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} failed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

/// Temp repo with one commit on `main`.
fn make_repo() -> (TempDir, GitVcsAdapter) {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main"]);
    git(dir.path(), &["config", "user.email", "t@example.com"]);
    git(dir.path(), &["config", "user.name", "tester"]);
    std::fs::write(dir.path().join("f.txt"), "base\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "init"]);
    let adapter = GitVcsAdapter::new(dir.path().to_path_buf());
    (dir, adapter)
}

fn canonical(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// Trimmed stdout of a `git` invocation that must succeed.
fn git_out(dir: &Path, args: &[&str]) -> String {
    String::from_utf8_lossy(&git(dir, args).stdout)
        .trim()
        .to_string()
}

/// Process-unique slug so a leftover worktree from an aborted run cannot
/// make a later run fail with "already exists". Each test passes its own
/// prefix, so slugs are distinct even when tests run concurrently.
fn unique_slug(prefix: &str) -> String {
    format!("{prefix}-{}", std::process::id())
}

// ---------------------------------------------------------------------------
// create_worktree
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_worktree_fails_when_target_path_is_occupied() {
    let (_dir, adapter) = make_repo();
    let slug = unique_slug("occupied");
    let target = adapter.worktree_path(&slug, "wp-1");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("keep.txt"), "keep\n").unwrap();

    let err = adapter
        .create_worktree(&slug, "wp-1")
        .await
        .expect_err("create must refuse an occupied path");
    assert!(
        err.to_string().contains("already exists"),
        "expected an 'already exists' error, got: {err}"
    );

    // The failure is non-destructive and leaves no worktree registered.
    assert!(target.join("keep.txt").is_file());
    let list = adapter.list_worktrees().await.unwrap();
    assert!(!list.iter().any(|w| w.path == canonical(&target)));

    // Once the blocker is gone the same call succeeds.
    std::fs::remove_dir_all(&target).unwrap();
    let created = adapter.create_worktree(&slug, "wp-1").await.unwrap();
    assert!(created.is_dir());
    adapter.cleanup_worktree(&created).await.unwrap();
    assert!(!created.exists());
}

#[tokio::test]
async fn create_worktree_on_unborn_head_repo_creates_a_real_worktree() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main"]);
    let adapter = GitVcsAdapter::new(dir.path().to_path_buf());
    let slug = unique_slug("unborn");

    let wt = adapter
        .create_worktree(&slug, "wp-9")
        .await
        .expect("worktree add works while HEAD is unborn");
    assert!(wt.join(".git").exists(), "worktree must have a .git link");

    let entry = adapter
        .list_worktrees()
        .await
        .unwrap()
        .into_iter()
        .find(|w| w.path == wt)
        .expect("fresh worktree should be listed");
    assert_eq!(entry.branch, format!("feat/{slug}/wp-9"));
    assert_eq!(entry.commit, ZERO_SHA);

    adapter.cleanup_worktree(&wt).await.unwrap();
    assert!(!wt.exists());
}

// ---------------------------------------------------------------------------
// cleanup_worktree
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cleanup_worktree_rejects_directory_that_is_not_a_worktree() {
    let (dir, adapter) = make_repo();
    let ghost = dir.path().join(".worktrees").join("ghost");
    std::fs::create_dir_all(&ghost).unwrap();
    std::fs::write(ghost.join("note.txt"), "not a worktree\n").unwrap();

    let err = adapter.cleanup_worktree(&ghost).await.unwrap_err();
    assert!(
        err.to_string().contains("is not a working tree"),
        "expected a 'not a working tree' error, got: {err}"
    );
    assert!(
        ghost.join("note.txt").is_file(),
        "a rejected cleanup must not delete anything"
    );
}

#[tokio::test]
async fn cleanup_worktree_removes_worktree_with_uncommitted_changes() {
    let (_dir, adapter) = make_repo();
    let slug = unique_slug("dirty");
    let wt = adapter.create_worktree(&slug, "wp-2").await.unwrap();
    std::fs::write(wt.join("f.txt"), "modified\n").unwrap();
    std::fs::write(wt.join("untracked.txt"), "untracked\n").unwrap();

    adapter
        .cleanup_worktree(&wt)
        .await
        .expect("--force cleanup ignores dirty state");
    assert!(!wt.exists());

    let list = adapter.list_worktrees().await.unwrap();
    assert!(!list.iter().any(|w| w.path == wt));
}

#[tokio::test]
async fn cleanup_worktree_prunes_entry_whose_directory_vanished() {
    let (dir, adapter) = make_repo();
    let slug = unique_slug("stale");
    let wt = adapter.create_worktree(&slug, "wp-3").await.unwrap();

    // Simulate an operator (or a crash) deleting the directory out from under git.
    std::fs::remove_dir_all(&wt).unwrap();

    // The registration survives, so the worktree is still reported...
    let listed = adapter.list_worktrees().await.unwrap();
    assert!(
        listed
            .iter()
            .any(|w| w.path == wt && w.branch == format!("feat/{slug}/wp-3")),
        "stale worktree should still be listed: {listed:?}"
    );
    let raw =
        String::from_utf8_lossy(&git(dir.path(), &["worktree", "list", "--porcelain"]).stdout)
            .into_owned();
    assert!(
        raw.contains("prunable"),
        "git marks the entry prunable: {raw}"
    );

    // ...and cleanup clears it, after which the path is unknown to git.
    adapter.cleanup_worktree(&wt).await.unwrap();
    let err = adapter.cleanup_worktree(&wt).await.unwrap_err();
    assert!(
        err.to_string().contains("is not a working tree"),
        "second cleanup should fail, got: {err}"
    );
    let listed = adapter.list_worktrees().await.unwrap();
    assert!(!listed.iter().any(|w| w.path == wt));
}

// ---------------------------------------------------------------------------
// list_worktrees
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_worktrees_reports_unborn_head_repo_with_zero_commit() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main"]);
    let adapter = GitVcsAdapter::new(dir.path().to_path_buf());

    let listed = adapter.list_worktrees().await.unwrap();
    assert_eq!(listed.len(), 1, "only the main worktree exists: {listed:?}");
    let entry = &listed[0];
    assert_eq!(entry.path, canonical(dir.path()));
    assert_eq!(entry.branch, "main");
    assert_eq!(entry.commit, ZERO_SHA);
    assert!(entry.feature_slug.is_empty());
    assert!(entry.wp_id.is_empty());
}

#[tokio::test]
async fn list_worktrees_reports_detached_worktree_without_branch() {
    let (dir, adapter) = make_repo();
    let head = git_out(dir.path(), &["rev-parse", "HEAD"]);

    let holder = tempfile::tempdir().unwrap();
    let det = holder.path().join("detached");
    git(
        dir.path(),
        &[
            "worktree",
            "add",
            "-q",
            "--detach",
            &det.to_string_lossy(),
            "HEAD",
        ],
    );

    let listed = adapter.list_worktrees().await.unwrap();
    let entry = listed
        .iter()
        .find(|w| w.path == canonical(&det))
        .unwrap_or_else(|| panic!("detached worktree missing from {listed:?}"));
    assert_eq!(entry.commit, head, "detached worktree sits on HEAD");
    assert!(
        entry.branch.is_empty(),
        "a detached worktree has no branch name, got {:?}",
        entry.branch
    );

    let main_entry = listed
        .iter()
        .find(|w| w.path == canonical(dir.path()))
        .expect("main worktree listed");
    assert_eq!(main_entry.branch, "main");

    adapter.cleanup_worktree(&det).await.unwrap();
}
