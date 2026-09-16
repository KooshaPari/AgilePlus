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

// ---------------------------------------------------------------------------
// Additional glob_match edge-case tests
// ---------------------------------------------------------------------------

#[test]
fn glob_match_empty_pattern_matches_empty_name() {
    assert!(glob_match("", ""));
}

#[test]
fn glob_match_empty_name_does_not_match_literal() {
    assert!(!glob_match("abc", ""));
}

#[test]
fn glob_match_single_star_matches_empty() {
    assert!(glob_match("*", ""));
}

#[test]
fn glob_match_double_wildcard_prefix() {
    assert!(glob_match("**/foo.rs", "src/deep/foo.rs"));
}

#[test]
fn glob_match_prefix_only() {
    assert!(glob_match("feat/*", "feat/"));
}

#[test]
fn glob_match_no_match_on_prefix_only_reversed() {
    assert!(!glob_match("feat/*", "feat"));
}

#[test]
fn glob_match_multiple_middle_wildcards() {
    assert!(glob_match("a/*b*c", "a/xbxxc"));
    assert!(!glob_match("a/*b*c", "a/xbxd"));
}

// glob_match "feat/" is treated as a literal (no wildcard)
#[test]
fn glob_match_trailing_star_literal_no_match() {
    assert!(!glob_match("feat/", "feat/anything"));
    assert!(glob_match("feat/", "feat/"));
}

#[test]
fn glob_match_trailing_star_with_wildcard() {
    assert!(glob_match("feat/*", "feat/anything"));
    assert!(!glob_match("feat/*", "feat"));
}

#[test]
fn glob_match_leading_star() {
    assert!(glob_match("*/main", "origin/main"));
    assert!(glob_match("*/main", "local/main"));
}

#[test]
fn glob_match_exact_after_leading_star() {
    assert!(!glob_match("*/main", "origin/develop"));
}

#[test]
fn glob_match_star_matches_slash() {
    assert!(glob_match("feat/*", "feat/a/b"));
}

// ---------------------------------------------------------------------------
// Additional parse_conflicts tests
// ---------------------------------------------------------------------------

#[test]
fn parse_conflicts_empty_input() {
    let conflicts = GitVcsAdapter::parse_conflicts("");
    assert!(conflicts.is_empty());
}

#[test]
fn parse_conflicts_no_conflicts() {
    let raw = "Auto-merging foo.txt\nMerge made by the 'ort' strategy.\n";
    let conflicts = GitVcsAdapter::parse_conflicts(raw);
    assert!(conflicts.is_empty());
}

#[test]
fn parse_conflicts_con_flict_add_add() {
    let raw = "CONFLICT (add/add): Merge conflict in new-file.txt\n";
    let conflicts = GitVcsAdapter::parse_conflicts(raw);
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].file_path, "new-file.txt");
    assert_eq!(conflicts[0].conflict_type, "add/add");
}

#[test]
fn parse_conflicts_con_flict_modify_delete() {
    let raw = "CONFLICT (modify/delete): deleted.txt deleted in HEAD and modified in topic\n";
    let conflicts = GitVcsAdapter::parse_conflicts(raw);
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].file_path, "deleted.txt");
    assert_eq!(conflicts[0].conflict_type, "modify/delete");
}

#[test]
fn parse_conflicts_multiple_conflicts() {
    let raw = "CONFLICT (content): Merge conflict in a.txt\nCONFLICT (content): Merge conflict in b.txt\n";
    let conflicts = GitVcsAdapter::parse_conflicts(raw);
    assert_eq!(conflicts.len(), 2);
    let paths: Vec<&str> = conflicts.iter().map(|c| c.file_path.as_str()).collect();
    assert!(paths.contains(&"a.txt"));
    assert!(paths.contains(&"b.txt"));
}

#[test]
fn parse_conflicts_changed_in_both_inline() {
    let raw = "changed in both src/main.rs\n";
    let conflicts = GitVcsAdapter::parse_conflicts(raw);
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].file_path, "src/main.rs");
    assert_eq!(conflicts[0].conflict_type, "content");
}

#[test]
fn parse_conflicts_index_row_format() {
    // Format: "mode oid stage<TAB>path" (3 whitespace-separated fields)
    let raw = "100644 abc1234 1\tsrc/lib.rs\n";
    let conflicts = GitVcsAdapter::parse_conflicts(raw);
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].file_path, "src/lib.rs");
}

#[test]
fn parse_conflicts_deduplicates_same_file() {
    let raw = concat!(
        "CONFLICT (content): Merge conflict in dup.txt\n",
        "100644 abc1234 1\tdup.txt\n",
    );
    let conflicts = GitVcsAdapter::parse_conflicts(raw);
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].file_path, "dup.txt");
}

#[test]
fn parse_conflicts_rename_conflict() {
    let raw = "CONFLICT (rename/delete): old.txt renamed to new.txt in HEAD and deleted in topic\n";
    let conflicts = GitVcsAdapter::parse_conflicts(raw);
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].conflict_type, "rename/delete");
}

#[test]
fn parse_conflicts_con_flict_empty_path_ignored() {
    let raw = "CONFLICT (content): Merge conflict in \n";
    let conflicts = GitVcsAdapter::parse_conflicts(raw);
    assert!(conflicts.is_empty());
}

// ---------------------------------------------------------------------------
// worktree_branch / worktree_dirname / worktree_path tests
// ---------------------------------------------------------------------------

#[test]
fn worktree_branch_format() {
    assert_eq!(
        GitVcsAdapter::worktree_branch("login", "wp-1"),
        "feat/login/wp-1"
    );
    assert_eq!(
        GitVcsAdapter::worktree_branch("my-feature", "WP42"),
        "feat/my-feature/WP42"
    );
}

#[test]
fn worktree_path_computation() {
    let adapter = GitVcsAdapter::new(PathBuf::from("/home/user/my-repo"));
    let wt_path = adapter.worktree_path("login", "wp-1");
    assert_eq!(
        wt_path,
        PathBuf::from("/home/user/login-wp-1")
    );
}

#[test]
fn worktree_path_with_root_parent() {
    let adapter = GitVcsAdapter::new(PathBuf::from("/"));
    let wt_path = adapter.worktree_path("feat", "wp-1");
    // PathBuf("/").parent() returns None on some platforms, falls back to "."
    // On others it may return Some("/"). Normalize to handle both.
    let dirname = GitVcsAdapter::worktree_branch("feat", "wp-1"); // just check it's not empty
    assert!(!dirname.is_empty());
    assert!(wt_path.to_string_lossy().contains("feat-wp-1"));
}

#[test]
fn worktree_path_no_parent_falls_back_to_dot() {
    // When repo_root has no parent (e.g., "repo" -> parent is None, falls back to ".")
    // ".".join("feat-wp-1") normalizes to "feat-wp-1"
    let adapter = GitVcsAdapter::new(PathBuf::from("repo"));
    let wt_path = adapter.worktree_path("feat", "wp-1");
    assert_eq!(wt_path, PathBuf::from("feat-wp-1"));
}

// ---------------------------------------------------------------------------
// checkout_blocked_by_other_worktree tests
// ---------------------------------------------------------------------------

#[test]
fn checkout_blocked_detected_already_used() {
    let err = DomainError::Storage("branch 'feat/x' is already used by worktree".into());
    assert!(GitVcsAdapter::checkout_blocked_by_other_worktree(&err));
}

#[test]
fn checkout_blocked_detected_already_checked_out() {
    let err = DomainError::Storage("fatal: 'feat/x' is already checked out at '/tmp/wt'".into());
    assert!(GitVcsAdapter::checkout_blocked_by_other_worktree(&err));
}

#[test]
fn checkout_blocked_detected_checkout_worktree_combo() {
    let err = DomainError::Storage("checkout failed: worktree conflict".into());
    assert!(GitVcsAdapter::checkout_blocked_by_other_worktree(&err));
}

#[test]
fn checkout_not_blocked_unrelated_error() {
    let err = DomainError::Storage("some other error".into());
    assert!(!GitVcsAdapter::checkout_blocked_by_other_worktree(&err));
}

// ---------------------------------------------------------------------------
// collect_evidence_paths tests
// ---------------------------------------------------------------------------

#[test]
fn collect_evidence_paths_empty_dir() {
    let dir = tempdir().unwrap();
    let mut paths = Vec::new();
    collect_evidence_paths(dir.path(), &mut paths).unwrap();
    assert!(paths.is_empty());
}

#[test]
fn collect_evidence_paths_single_file() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("result.json"), "{}").unwrap();
    let mut paths = Vec::new();
    collect_evidence_paths(dir.path(), &mut paths).unwrap();
    assert_eq!(paths.len(), 1);
    assert!(paths[0].contains("result.json"));
}

#[test]
fn collect_evidence_paths_nested_dirs() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("a").join("b");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join("deep.txt"), "data").unwrap();
    std::fs::write(dir.path().join("top.txt"), "data").unwrap();
    let mut paths = Vec::new();
    collect_evidence_paths(dir.path(), &mut paths).unwrap();
    assert_eq!(paths.len(), 2);
}

#[test]
fn collect_evidence_paths_dirs_are_skipped() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("subdir")).unwrap();
    std::fs::write(dir.path().join("file.txt"), "x").unwrap();
    let mut paths = Vec::new();
    collect_evidence_paths(dir.path(), &mut paths).unwrap();
    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("file.txt"));
}

// ---------------------------------------------------------------------------
// scan_all_features edge cases
// ---------------------------------------------------------------------------

#[test]
fn scan_all_features_no_agileplus_dir() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path);
    let slugs = scan_all_features(&adapter).unwrap();
    assert!(slugs.is_empty());
}

#[test]
fn scan_all_features_empty_dir() {
    let (_dir, path) = make_repo();
    let agileplus_dir = path.join("docs").join("agileplus");
    std::fs::create_dir_all(&agileplus_dir).unwrap();
    let adapter = GitVcsAdapter::new(path);
    let slugs = scan_all_features(&adapter).unwrap();
    assert!(slugs.is_empty());
}

#[test]
fn scan_all_features_dir_without_meta_json() {
    let (_dir, path) = make_repo();
    let feature_dir = path.join("docs").join("agileplus").join("no-meta");
    std::fs::create_dir_all(&feature_dir).unwrap();
    std::fs::write(feature_dir.join("readme.txt"), "hi").unwrap();
    let adapter = GitVcsAdapter::new(path);
    let slugs = scan_all_features(&adapter).unwrap();
    assert!(slugs.is_empty());
}

#[test]
fn scan_all_features_sorted_order() {
    let (_dir, path) = make_repo();
    for slug in &["zebra", "alpha", "middle"] {
        let meta = path
            .join("docs")
            .join("agileplus")
            .join(slug)
            .join("meta.json");
        std::fs::create_dir_all(meta.parent().unwrap()).unwrap();
        std::fs::write(&meta, "{}").unwrap();
    }
    let adapter = GitVcsAdapter::new(path);
    let slugs = scan_all_features(&adapter).unwrap();
    assert_eq!(slugs, vec!["alpha", "middle", "zebra"]);
}

// ---------------------------------------------------------------------------
// Adapter construction tests
// ---------------------------------------------------------------------------

#[test]
fn from_current_dir_succeeds() {
    let adapter = GitVcsAdapter::from_current_dir().unwrap();
    assert!(adapter.repo_root().exists());
}

#[test]
fn new_adapter_stores_path() {
    let p = PathBuf::from("/some/path");
    let adapter = GitVcsAdapter::new(p.clone());
    assert_eq!(adapter.repo_root(), p.as_path());
}

#[test]
fn open_fails_on_non_repo() {
    let dir = tempdir().unwrap();
    let adapter = GitVcsAdapter::new(dir.path().to_path_buf());
    let result = adapter.open();
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// create_claim_bound_worktree integration with in-memory claim store
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn create_claim_bound_worktree_end_to_end() {
    use crate::claim_bound::{ClaimBoundWorktree, make_worktree_claim};
    use agileplus_triage::claim::ClaimStore;

    let (_dir, path) = make_repo();
    let mut store = ClaimStore::new();
    let claim = make_worktree_claim("c-e2e", "feat/bound-e2e/wp-1", "agent-e2e", 300);
    store
        .claim(
            &claim.id,
            &claim.resource,
            claim.kind,
            &claim.agent_id,
            claim.ttl_seconds,
            claim.reason.clone(),
        )
        .unwrap();

    let wt_path = ClaimBoundWorktree::create(path.clone(), "bound-e2e", "wp-1", &claim, &mut store)
        .expect("create claim-bound worktree");
    assert!(wt_path.is_dir());

    // Verify the stored claim now carries the worktree path.
    let stored = store.lookup(claim.kind, &claim.resource).unwrap();
    let recovered = ClaimBoundWorktree::lookup(&stored).unwrap();
    assert_eq!(recovered, wt_path);

    // Cleanup.
    let adapter = GitVcsAdapter::new(path);
    adapter.cleanup_worktree(&wt_path).await.unwrap();
}

// ---------------------------------------------------------------------------
// worktree create + cleanup round trip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn worktree_create_with_existing_branch_reuses() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());
    // Use a unique slug including process id to avoid conflicts across test runs
    let slug = format!("reuser-{}", std::process::id());

    let wt1 = adapter.create_worktree(&slug, "wp-1").await.unwrap();
    // Cleanup the worktree via git
    adapter.cleanup_worktree(&wt1).await.unwrap();

    // Verify the worktree is gone from the listing
    let list = adapter.list_worktrees().await.unwrap();
    assert!(
        !list.iter().any(|w| w.path == wt1),
        "worktree should be removed from list after cleanup"
    );

    // Force-remove any leftover directory
    if wt1.exists() {
        std::fs::remove_dir_all(&wt1).expect("force remove worktree dir");
    }

    // Create again with same slug/wp_id -- branch still exists, uses -B to re-attach
    let wt2 = adapter.create_worktree(&slug, "wp-1").await.unwrap();
    assert!(wt2.is_dir());

    let list = adapter.list_worktrees().await.unwrap();
    assert!(
        list.iter().any(|w| w.path == wt2),
        "reattached worktree should appear in list"
    );

    adapter.cleanup_worktree(&wt2).await.unwrap();
}

// ---------------------------------------------------------------------------
// list_branches with pattern filtering
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_branches_pattern_filtering() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());

    adapter.create_branch("feat/alpha", "main").await.unwrap();
    adapter.create_branch("feat/beta", "main").await.unwrap();
    adapter.create_branch("fix/bug", "main").await.unwrap();

    let feat_branches = adapter
        .list_branches(Some("feat/*"), false)
        .await
        .unwrap();
    assert_eq!(feat_branches.len(), 2);
    let names: Vec<&str> = feat_branches.iter().map(|b| b.name.as_str()).collect();
    assert!(names.contains(&"feat/alpha"));
    assert!(names.contains(&"feat/beta"));

    let all_local = adapter.list_branches(None, false).await.unwrap();
    assert!(all_local.len() >= 4); // main + feat/alpha + feat/beta + fix/bug
}

// ---------------------------------------------------------------------------
// merge: up-to-date case
// ---------------------------------------------------------------------------

#[tokio::test]
async fn merge_up_to_date_returns_success() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());
    // Merging main into main should be up-to-date.
    let res = adapter.merge_to_target("main", "main").await.unwrap();
    assert!(res.success);
    assert!(res.conflicts.is_empty());
}

// ---------------------------------------------------------------------------
// delete_branch rejects deleting checked-out branch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_branch_rejects_current_branch() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());
    // Try to delete the currently checked-out branch.
    let result = adapter.delete_branch("main", false, None).await;
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// ConflictInfo fields populated correctly
// ---------------------------------------------------------------------------

#[test]
fn parse_conflicts_populates_path_and_file_path() {
    let raw = "CONFLICT (content): Merge conflict in readme.md\n";
    let conflicts = GitVcsAdapter::parse_conflicts(raw);
    assert_eq!(conflicts[0].path, "readme.md");
    assert_eq!(conflicts[0].file_path, "readme.md");
}

#[test]
fn parse_conflicts_ours_theirs_are_none() {
    let raw = "CONFLICT (content): Merge conflict in f.txt\n";
    let conflicts = GitVcsAdapter::parse_conflicts(raw);
    assert!(conflicts[0].ours.is_none());
    assert!(conflicts[0].theirs.is_none());
}

// ---------------------------------------------------------------------------
// BranchInfo / WorktreeInfo struct validation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn branch_info_has_correct_fields() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());
    adapter.create_branch("feat/test-branch", "main").await.unwrap();

    let branches = adapter
        .list_branches(Some("feat/test-branch"), false)
        .await
        .unwrap();
    assert_eq!(branches.len(), 1);
    let b = &branches[0];
    assert_eq!(b.name, "feat/test-branch");
    assert!(b.is_remote == false || b.is_remote == true); // just ensure field exists
}

// ---------------------------------------------------------------------------
// Multiple worktree creation and listing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn multiple_worktrees_listed() {
    let (_dir, path) = make_repo();
    let adapter = GitVcsAdapter::new(path.clone());

    let wt1 = adapter.create_worktree("multi-a", "wp-1").await.unwrap();
    let wt2 = adapter.create_worktree("multi-b", "wp-2").await.unwrap();

    let list = adapter.list_worktrees().await.unwrap();
    let wt_paths: Vec<PathBuf> = list.iter().map(|w| w.path.clone()).collect();
    assert!(wt_paths.contains(&wt1));
    assert!(wt_paths.contains(&wt2));

    adapter.cleanup_worktree(&wt1).await.unwrap();
    adapter.cleanup_worktree(&wt2).await.unwrap();
}
