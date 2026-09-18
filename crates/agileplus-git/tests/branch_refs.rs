//! Branch-reference behavior for the `VcsPort` implementation.
//!
//! Covers local create / checkout / delete failure modes plus the
//! remote-tracking half of the branch surface (`list_branches(.., true)`
//! and push-based `delete_branch(.., Some(remote))`), which the
//! happy-path suites exercise only for local refs.
//!
//! Every test runs against throwaway repositories under a `TempDir`; no
//! shared environment or global git config is touched.

use agileplus_domain::ports::VcsPort;
use agileplus_git::GitVcsAdapter;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

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

fn commit_file(dir: &Path, name: &str, body: &str, msg: &str) {
    std::fs::write(dir.join(name), body).unwrap();
    git(dir, &["add", name]);
    git(dir, &["commit", "-q", "-m", msg]);
}

/// Trimmed stdout of a `git` invocation that must succeed.
fn git_out(dir: &Path, args: &[&str]) -> String {
    String::from_utf8_lossy(&git(dir, args).stdout)
        .trim()
        .to_string()
}

/// Sorted local branch names, for order-insensitive assertions.
async fn local_names(adapter: &GitVcsAdapter) -> Vec<String> {
    let mut names: Vec<String> = adapter
        .list_branches(None, false)
        .await
        .unwrap()
        .into_iter()
        .map(|b| b.name)
        .collect();
    names.sort();
    names
}

// ---------------------------------------------------------------------------
// create_branch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_branch_rejects_existing_name() {
    let (_dir, adapter) = make_repo();
    adapter.create_branch("feat/dup", "main").await.unwrap();

    let err = adapter.create_branch("feat/dup", "main").await.unwrap_err();
    assert!(
        err.to_string().contains("already exists"),
        "expected an exists error, got: {err}"
    );

    // The rejected call must not have disturbed the existing ref.
    let branches = adapter
        .list_branches(Some("feat/dup"), false)
        .await
        .unwrap();
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "feat/dup");
    assert!(!branches[0].is_remote);
}

#[tokio::test]
async fn create_branch_rejects_unresolvable_base_and_creates_nothing() {
    let (_dir, adapter) = make_repo();

    let err = adapter
        .create_branch("feat/from-ghost", "no-such-base")
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("no-such-base"),
        "error should name the bad base: {err}"
    );

    let names = local_names(&adapter).await;
    assert!(
        !names.iter().any(|n| n == "feat/from-ghost"),
        "no branch should exist after a failed create: {names:?}"
    );
    assert_eq!(names, vec!["main".to_string()]);
}

// ---------------------------------------------------------------------------
// checkout_branch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn checkout_unknown_branch_errors_and_keeps_head() {
    let (dir, adapter) = make_repo();
    adapter.create_branch("feat/keep", "main").await.unwrap();
    adapter.checkout_branch("feat/keep").await.unwrap();

    let err = adapter.checkout_branch("no-such-branch").await.unwrap_err();
    assert!(
        err.to_string().contains("pathspec 'no-such-branch'"),
        "expected a pathspec error, got: {err}"
    );

    let head = git_out(dir.path(), &["rev-parse", "--abbrev-ref", "HEAD"]);
    assert_eq!(head, "feat/keep", "failed checkout must not move HEAD");
}

// ---------------------------------------------------------------------------
// delete_branch (local)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_missing_local_branch_errors() {
    let (_dir, adapter) = make_repo();

    let err = adapter
        .delete_branch("ghost", false, None)
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("cannot locate local branch"),
        "expected a lookup error, got: {err}"
    );
    assert_eq!(local_names(&adapter).await, vec!["main".to_string()]);
}

/// libgit2's branch delete has no merge-state check, so the `force`
/// argument does not change the outcome: an unmerged branch goes away
/// either way while `main` survives.
#[tokio::test]
async fn delete_unmerged_branch_succeeds_for_both_force_values() {
    let (dir, adapter) = make_repo();
    for (branch, force) in [("feat/unmerged-safe", false), ("feat/unmerged-force", true)] {
        git(dir.path(), &["checkout", "-q", "-b", branch]);
        commit_file(dir.path(), "work.txt", "unmerged\n", "unmerged work");
        git(dir.path(), &["checkout", "-q", "main"]);

        adapter
            .delete_branch(branch, force, None)
            .await
            .unwrap_or_else(|e| panic!("delete {branch} (force={force}) failed: {e}"));
    }

    assert_eq!(local_names(&adapter).await, vec!["main".to_string()]);
}

// ---------------------------------------------------------------------------
// list_branches
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_branches_pattern_with_no_match_is_empty() {
    let (_dir, adapter) = make_repo();
    adapter.create_branch("feat/alpha", "main").await.unwrap();

    let none = adapter.list_branches(Some("fix/*"), false).await.unwrap();
    assert!(none.is_empty(), "fix/* should match nothing: {none:?}");

    let local = adapter.list_branches(None, false).await.unwrap();
    assert!(local.iter().all(|b| !b.is_remote));
    assert_eq!(local.len(), 2, "main + feat/alpha: {local:?}");
}

#[tokio::test]
async fn list_remote_branches_without_a_remote_is_empty() {
    let (_dir, adapter) = make_repo();
    adapter
        .create_branch("feat/local-only", "main")
        .await
        .unwrap();

    let remotes = adapter.list_branches(None, true).await.unwrap();
    assert!(
        remotes.is_empty(),
        "no remote configured means no remote-tracking refs: {remotes:?}"
    );
    assert!(!local_names(&adapter).await.is_empty());
}

// ---------------------------------------------------------------------------
// Remote branches: listing and push-based deletion
// ---------------------------------------------------------------------------

/// Repo with a bare `origin` that already has `main` and `feat/pushed`.
fn repo_with_origin() -> (TempDir, TempDir, GitVcsAdapter) {
    let origin = tempfile::tempdir().unwrap();
    git(origin.path(), &["init", "-q", "--bare"]);

    let (dir, adapter) = make_repo();
    git(
        dir.path(),
        &["remote", "add", "origin", &origin.path().to_string_lossy()],
    );
    git(dir.path(), &["push", "-q", "origin", "main"]);
    git(dir.path(), &["branch", "feat/pushed"]);
    git(dir.path(), &["push", "-q", "origin", "feat/pushed"]);
    // A local branch that was never pushed.
    git(dir.path(), &["branch", "feat/never-pushed"]);

    (dir, origin, adapter)
}

fn upstream_refs(origin: &Path) -> Vec<String> {
    let out = git(origin, &["ls-remote", "--heads", "."]);
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.split_once('\t').map(|(_, r)| r.to_string()))
        .collect()
}

#[tokio::test]
async fn remote_tracking_refs_are_listed_after_push() {
    let (_dir, _origin, adapter) = repo_with_origin();

    let remotes = adapter.list_branches(None, true).await.unwrap();
    let names: Vec<&str> = remotes.iter().map(|b| b.name.as_str()).collect();
    assert!(names.contains(&"origin/main"), "{names:?}");
    assert!(names.contains(&"origin/feat/pushed"), "{names:?}");
    assert!(
        !names.contains(&"origin/feat/never-pushed"),
        "unpushed branch must not appear as remote: {names:?}"
    );
    assert!(
        remotes.iter().all(|b| b.is_remote),
        "remote listing must flag is_remote: {remotes:?}"
    );

    let local_main = adapter
        .list_branches(Some("main"), false)
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("local main");
    let remote_main = remotes
        .iter()
        .find(|b| b.name == "origin/main")
        .expect("origin/main");
    assert_eq!(remote_main.commit, local_main.commit);
    assert_eq!(remote_main.commit.len(), 40);
}

#[tokio::test]
async fn delete_branch_with_remote_pushes_ref_deletion() {
    let (_dir, origin, adapter) = repo_with_origin();
    assert!(
        upstream_refs(origin.path())
            .iter()
            .any(|r| r == "refs/heads/feat/pushed")
    );

    adapter
        .delete_branch("feat/pushed", false, Some("origin"))
        .await
        .unwrap();

    let refs = upstream_refs(origin.path());
    assert!(
        !refs.iter().any(|r| r == "refs/heads/feat/pushed"),
        "remote ref should be deleted upstream: {refs:?}"
    );
    assert!(refs.iter().any(|r| r == "refs/heads/main"));

    // The local branch is intentionally untouched by a remote delete.
    assert!(
        local_names(&adapter)
            .await
            .iter()
            .any(|n| n == "feat/pushed")
    );
    let remotes = adapter.list_branches(None, true).await.unwrap();
    assert!(
        !remotes.iter().any(|b| b.name == "origin/feat/pushed"),
        "tracking ref should be pruned with the upstream: {remotes:?}"
    );
}

#[tokio::test]
async fn delete_remote_branch_that_does_not_exist_errors() {
    let (_dir, origin, adapter) = repo_with_origin();

    let err = adapter
        .delete_branch("feat/nope", false, Some("origin"))
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("remote ref does not exist"),
        "expected upstream deletion error, got: {err}"
    );
    assert!(
        upstream_refs(origin.path())
            .iter()
            .any(|r| r == "refs/heads/main")
    );
}

#[tokio::test]
async fn delete_remote_branch_with_unknown_remote_errors() {
    let (_dir, adapter) = make_repo();

    let err = adapter
        .delete_branch("main", false, Some("origin"))
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("does not appear to be a git repository"),
        "expected a missing-remote error, got: {err}"
    );
    assert_eq!(local_names(&adapter).await, vec!["main".to_string()]);
}
