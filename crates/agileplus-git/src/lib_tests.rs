// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tests for the Git VCS adapter and standalone helper functions.

use super::*;
use agileplus_domain::ports::VcsPort;
use std::process::Command as StdCommand;
use tempfile::tempdir;

/// Make a temp git repo with one initial commit on `main`.
fn make_repo() -> (tempfile::TempDir, PathBuf) {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    StdCommand::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&path)
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["config", "user.email", "t@example.com"])
        .current_dir(&path)
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["config", "user.name", "tester"])
        .current_dir(&path)
        .output()
        .unwrap();
    std::fs::write(path.join("README.md"), "hello\n").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-q", "-m", "init"])
        .current_dir(&path)
        .output()
        .unwrap();
    (dir, path)
}

#[test]
fn glob_match_basic() {
    assert!(glob_match("*", "anything"));
    assert!(glob_match("feat/*", "feat/login"));
    assert!(!glob_match("feat/*", "fix/login"));
    assert!(glob_match("*/main", "origin/main"));
    assert!(glob_match("feat/*/wp-1", "feat/login/wp-1"));
    assert!(!glob_match("feat/*/wp-1", "feat/login/wp-2"));
    assert!(glob_match("literal", "literal"));
    assert!(!glob_match("literal", "other"));
}

#[test]
fn parse_conflicts_reads_legacy_merge_tree_paths_with_spaces() {
    let raw = "changed in both\n  base   100644 abcdef docs/legacy conflict.txt\n  our    100644 012345 docs/legacy conflict.txt\n  their  100644 6789ab docs/legacy conflict.txt\n";

    let conflicts = GitVcsAdapter::parse_conflicts(raw);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].file_path, "docs/legacy conflict.txt");
    assert_eq!(conflicts[0].conflict_type, "content");
}

#[test]
fn parse_conflicts_reads_quoted_diff_paths_with_spaces() {
    let raw = format!(
        "diff --git \"a/docs/space conflict.txt\" \"b/docs/space conflict.txt\"\n{} HEAD\nours\n{}\ntheirs\n{} topic\n",
        "<".repeat(7),
        "=".repeat(7),
        ">".repeat(7),
    );

    let conflicts = GitVcsAdapter::parse_conflicts(&raw);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].file_path, "docs/space conflict.txt");
}

#[test]
fn parse_conflicts_decodes_quoted_diff_path_escapes() {
    let raw = format!(
        "diff --git \"a/docs/quote\\\"name.txt\" \"b/docs/quote\\\"name.txt\"\n{} HEAD\nours\n{}\ntheirs\n{} topic\n",
        "<".repeat(7),
        "=".repeat(7),
        ">".repeat(7),
    );

    let conflicts = GitVcsAdapter::parse_conflicts(&raw);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].file_path, "docs/quote\"name.txt");
}

#[test]
fn parse_conflicts_decodes_utf8_octal_quoted_paths() {
    let raw = format!(
        "diff --git \"a/docs/caf\\303\\251.txt\" \"b/docs/caf\\303\\251.txt\"\n{} HEAD\nours\n{}\ntheirs\n{} topic\n",
        "<".repeat(7),
        "=".repeat(7),
        ">".repeat(7),
    );

    let conflicts = GitVcsAdapter::parse_conflicts(&raw);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].file_path, "docs/café.txt");
}

#[tokio::test]
async fn create_and_list_worktree() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());
    let wt = adapter
        .create_worktree("adapter-login", "wp-1")
        .await
        .expect("create worktree");
    assert!(
        wt.is_dir(),
        "worktree dir was not created: {}",
        wt.display()
    );
    let list = adapter.list_worktrees().await.expect("list worktrees");
    let names: Vec<&str> = list.iter().map(|w| w.branch.as_str()).collect();
    assert!(
        names.iter().any(|n| n.contains("wp-1")),
        "expected wp-1 branch in worktree list, got {:?}",
        names
    );
    // Cleanup to avoid polluting temp dir for subsequent tests
    adapter
        .cleanup_worktree(&wt)
        .await
        .expect("cleanup worktree");
}
#[tokio::test]
async fn create_branch_lists_and_checkout() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());
    adapter
        .create_branch("feat/x", "main")
        .await
        .expect("create branch");
    let locals = adapter
        .list_branches(Some("feat/*"), false)
        .await
        .expect("list local branches");
    assert!(locals.iter().any(|b| b.name == "feat/x"));
    adapter.checkout_branch("feat/x").await.expect("checkout");
    let head = adapter
        .run_git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap();
    assert_eq!(head, "feat/x");
}

#[tokio::test]
async fn merge_no_conflict_to_target() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());
    // create a feature branch with a non-conflicting change
    adapter.create_branch("feat/ok", "main").await.unwrap();
    adapter.checkout_branch("feat/ok").await.unwrap();
    std::fs::write(path.join("newfile.txt"), "hi").unwrap();
    StdCommand::new("git")
        .args(["add", "newfile.txt"])
        .current_dir(&path)
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-q", "-m", "add newfile"])
        .current_dir(&path)
        .output()
        .unwrap();
    let res = adapter
        .merge_to_target("feat/ok", "main")
        .await
        .expect("merge");
    assert!(res.success, "merge should succeed: {:?}", res.message);
    assert!(res.commit.is_some());
}

#[tokio::test]
// Traces to: FR-006 / ship worktree-aware merge
async fn merge_when_target_checked_out_elsewhere() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());
    adapter.create_branch("feat/ok", "main").await.unwrap();
    adapter.checkout_branch("feat/ok").await.unwrap();
    std::fs::write(path.join("newfile.txt"), "hi").unwrap();
    StdCommand::new("git")
        .args(["add", "newfile.txt"])
        .current_dir(&path)
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-q", "-m", "add newfile"])
        .current_dir(&path)
        .output()
        .unwrap();

    // Lock `main` in a sibling worktree (canonical-style conflict).
    let lock_wt = path.parent().unwrap().join("lock-main-wt");
    StdCommand::new("git")
        .args(["worktree", "add", &lock_wt.to_string_lossy(), "main"])
        .current_dir(&path)
        .output()
        .expect("lock main in sibling worktree");

    let res = adapter
        .merge_to_target("feat/ok", "main")
        .await
        .expect("merge via temp worktree");
    assert!(
        res.success,
        "merge should succeed when main is locked elsewhere: {:?}",
        res.message
    );
    assert!(res.commit.is_some());

    // Cleanup locked worktree
    let _ = StdCommand::new("git")
        .args(["worktree", "remove", "--force", &lock_wt.to_string_lossy()])
        .current_dir(&path)
        .output();
}

#[tokio::test]
async fn merge_conflict_detected() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());
    // create a feature branch that edits README.md
    adapter
        .create_branch("feat/conflict", "main")
        .await
        .unwrap();
    adapter.checkout_branch("feat/conflict").await.unwrap();
    std::fs::write(path.join("README.md"), "from feature\n").unwrap();
    StdCommand::new("git")
        .args(["add", "README.md"])
        .current_dir(&path)
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-q", "-m", "feature edit"])
        .current_dir(&path)
        .output()
        .unwrap();
    // edit README.md differently on main
    adapter.checkout_branch("main").await.unwrap();
    std::fs::write(path.join("README.md"), "from main\n").unwrap();
    StdCommand::new("git")
        .args(["add", "README.md"])
        .current_dir(&path)
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-q", "-m", "main edit"])
        .current_dir(&path)
        .output()
        .unwrap();
    let res = adapter
        .merge_to_target("feat/conflict", "main")
        .await
        .expect("merge attempted");
    assert!(!res.success, "merge should have conflicted");
    // abort so the test repo is in a clean state for other tests
    let _ = adapter.run_git_status(&["merge", "--abort"]);
    let conflicts = adapter
        .detect_conflicts("feat/conflict", "main")
        .await
        .expect("detect conflicts");
    assert!(!conflicts.is_empty(), "expected at least one conflict");
    assert!(conflicts.iter().any(|c| c.file_path == "README.md"));
}

#[tokio::test]
async fn delete_local_and_remote_branch() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());
    adapter
        .create_branch("feat/ephemeral", "main")
        .await
        .unwrap();
    adapter
        .delete_branch("feat/ephemeral", false, None)
        .await
        .expect("delete local");
    let locals = adapter.list_branches(Some("feat/*"), false).await.unwrap();
    assert!(locals.iter().all(|b| b.name != "feat/ephemeral"));
}

#[tokio::test]
async fn artifact_round_trip() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());
    adapter
        .write_artifact("login", "spec.md", "# spec\n")
        .await
        .expect("write");
    let content = adapter
        .read_artifact("login", "spec.md")
        .await
        .expect("read");
    assert_eq!(content, "# spec\n");
    assert!(adapter.artifact_exists("login", "spec.md").await.unwrap());

    adapter
        .write_artifact("login", "meta.json", r#"{"slug":"login"}"#)
        .await
        .expect("write metadata");
    adapter
        .write_artifact("login", "audit/chain.jsonl", r#"{"event":"created"}"#)
        .await
        .expect("write audit chain");
    adapter
        .write_artifact("login", "evidence/run/result.json", r#"{"ok":true}"#)
        .await
        .expect("write nested evidence");

    let scan = adapter.scan_feature_artifacts("login").await.expect("scan");
    assert_eq!(scan.spec.as_deref(), Some("# spec\n"));
    assert_eq!(scan.meta_json.as_deref(), Some(r#"{"slug":"login"}"#));
    assert_eq!(scan.audit_chain.as_deref(), Some(r#"{"event":"created"}"#));
    assert_eq!(scan.evidence_paths.len(), 1);
}

#[tokio::test]
async fn cleanup_worktree_removes_it() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());
    let wt = adapter.create_worktree("x", "wp-1").await.unwrap();
    assert!(wt.is_dir());
    adapter.cleanup_worktree(&wt).await.expect("cleanup");
    assert!(!wt.exists(), "worktree should be removed");
}
