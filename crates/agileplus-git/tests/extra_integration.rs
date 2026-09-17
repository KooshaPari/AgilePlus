//! Additional integration tests for the agileplus-git crate.
//!
//! Tests materialize rendering functions, project context discovery,
//! and adapter VcsPort operations that exercise the full stack.

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::work_package::{PrState, WorkPackage};
use agileplus_domain::ports::VcsPort;
use agileplus_git::GitVcsAdapter;
use git2::{Repository, Signature};
use std::path::Path;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test harness
// ---------------------------------------------------------------------------

fn setup_test_repo() -> (TempDir, GitVcsAdapter) {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = Repository::init(dir.path()).expect("git init");
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test User").unwrap();
    config.set_str("user.email", "test@test.com").unwrap();
    drop(config);

    make_commit(&repo, dir.path(), "README.md", "# Test\n", "Initial commit");
    let adapter = GitVcsAdapter::new(dir.path().to_path_buf());
    (dir, adapter)
}

fn make_commit(repo: &Repository, workdir: &Path, filename: &str, content: &str, message: &str) {
    let path = workdir.join(filename);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, content).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new(filename)).unwrap();
    index.write().unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let sig = Signature::now("Test User", "test@test.com").unwrap();
    let parents: Vec<git2::Commit> = match repo.head() {
        Ok(head) => vec![head.peel_to_commit().unwrap()],
        Err(_) => vec![],
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
        .unwrap();
}

fn sample_feature(slug: &str) -> Feature {
    let mut f = Feature::new(slug, &format!("Feature {slug}"), [0u8; 32], Some("main"));
    f.id = 1;
    f.plane_issue_id = Some("PLN-123".to_string());
    f.labels = vec!["backend".to_string(), "api".to_string()];
    f.created_at_commit = Some("abc123".to_string());
    f.last_modified_commit = Some("def456".to_string());
    f
}

fn sample_work_package(feature_id: i64, seq: i32) -> WorkPackage {
    let mut wp = WorkPackage::new(
        feature_id,
        &format!("Work Package {seq}"),
        seq,
        "Must pass tests",
    );
    // Use sequential IDs starting from 1 so files are 1.json, 2.json, etc.
    wp.id = seq as i64;
    wp.file_scope = vec![format!("src/module{seq}.rs")];
    wp.agent_id = Some("agent-alpha".to_string());
    wp.pr_url = Some("https://github.com/org/repo/pull/42".to_string());
    wp.pr_state = Some(PrState::Review);
    wp.worktree_path = Some("/tmp/wt/feat-1".to_string());
    wp.base_commit = Some("aaa111".to_string());
    wp.head_commit = Some("bbb222".to_string());
    wp
}

// ---------------------------------------------------------------------------
// materialize::render_meta_json tests
// ---------------------------------------------------------------------------

#[test]
fn render_meta_json_all_fields() {
    let feature = sample_feature("login");
    let value = agileplus_git::materialize::render_meta_json(&feature);

    assert_eq!(value["slug"], "login");
    assert_eq!(value["friendly_name"], "Feature login");
    assert_eq!(value["state"], "created");
    assert_eq!(value["target_branch"], "main");
    assert!(value["spec_hash"].as_str().unwrap().len() == 64); // 32 bytes * 2 hex chars
    assert!(value["created_at"].as_str().is_some());
    assert!(value["updated_at"].as_str().is_some());
    assert!(value["materialized_at"].as_str().is_some());
    assert_eq!(value["created_at_commit"], "abc123");
    assert_eq!(value["last_modified_commit"], "def456");
    assert_eq!(value["plane_issue_id"], "PLN-123");
    let labels = value["labels"].as_array().unwrap();
    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&serde_json::json!("backend")));
    assert!(labels.contains(&serde_json::json!("api")));
}

#[test]
fn render_meta_json_empty_labels() {
    let mut feature = sample_feature("empty-labels");
    feature.labels = vec![];
    let value = agileplus_git::materialize::render_meta_json(&feature);
    let labels = value["labels"].as_array().unwrap();
    assert!(labels.is_empty());
}

#[test]
fn render_meta_json_no_plane_issue() {
    let mut feature = sample_feature("no-plane");
    feature.plane_issue_id = None;
    let value = agileplus_git::materialize::render_meta_json(&feature);
    assert!(value["plane_issue_id"].is_null());
}

// ---------------------------------------------------------------------------
// materialize::render_status_md tests
// ---------------------------------------------------------------------------

#[test]
fn render_status_md_with_work_packages() {
    let feature = sample_feature("checkout");
    let wps = vec![sample_work_package(1, 1), sample_work_package(1, 2)];
    let md = agileplus_git::materialize::render_status_md(&feature, &wps);

    assert!(md.contains("# Feature checkout"));
    assert!(md.contains("**Slug**: `checkout`"));
    assert!(md.contains("**State**: created"));
    assert!(md.contains("**Target branch**: `main`"));
    assert!(md.contains("Work Packages"));
    assert!(md.contains("| 1 |"));
    assert!(md.contains("| 2 |"));
    assert!(md.contains("agent-alpha"));
    assert!(md.contains("Do not edit directly"));
}

#[test]
fn render_status_md_no_work_packages() {
    let feature = sample_feature("empty-wps");
    let md = agileplus_git::materialize::render_status_md(&feature, &[]);
    assert!(md.contains("*No work packages.*"));
}

#[test]
fn render_status_md_with_labels() {
    let feature = sample_feature("labeled");
    let md = agileplus_git::materialize::render_status_md(&feature, &[]);
    assert!(md.contains("**Labels**: backend, api"));
}

#[test]
fn render_status_md_without_labels() {
    let mut feature = sample_feature("no-labels");
    feature.labels = vec![];
    let md = agileplus_git::materialize::render_status_md(&feature, &[]);
    assert!(!md.contains("**Labels**:"));
}

// ---------------------------------------------------------------------------
// materialize::render_audit_line tests
// ---------------------------------------------------------------------------

#[test]
fn render_audit_line_produces_valid_json() {
    let feature = sample_feature("audit-test");
    let line = agileplus_git::materialize::render_audit_line(&feature, Some("commit-abc"));
    let parsed: serde_json::Value =
        serde_json::from_str(&line).expect("audit line should be valid JSON");
    assert_eq!(parsed["slug"], "audit-test");
    assert_eq!(parsed["commit"], "commit-abc");
    assert!(parsed["timestamp"].as_str().is_some());
}

#[test]
fn render_audit_line_without_commit() {
    let feature = sample_feature("no-commit");
    let line = agileplus_git::materialize::render_audit_line(&feature, None);
    let parsed: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert!(parsed["commit"].is_null());
}

// ---------------------------------------------------------------------------
// materialize::render_wp_json tests
// ---------------------------------------------------------------------------

#[test]
fn render_wp_json_all_fields() {
    let wp = sample_work_package(1, 3);
    let value = agileplus_git::materialize::render_wp_json(&wp);

    assert_eq!(value["feature_id"], 1);
    assert_eq!(value["title"], "Work Package 3");
    assert_eq!(value["sequence"], 3);
    assert_eq!(value["state"], "planned");
    assert_eq!(value["agent_id"], "agent-alpha");
    assert_eq!(value["pr_url"], "https://github.com/org/repo/pull/42");
    assert_eq!(value["pr_state"], "review");
    assert_eq!(value["base_commit"], "aaa111");
    assert_eq!(value["head_commit"], "bbb222");
    assert_eq!(value["worktree_path"], "/tmp/wt/feat-1");
    let scope = value["file_scope"].as_array().unwrap();
    assert_eq!(scope.len(), 1);
    assert_eq!(scope[0], "src/module3.rs");
    assert!(value["materialized_at"].as_str().is_some());
}

#[test]
fn render_wp_json_optional_fields_none() {
    let wp = WorkPackage {
        agent_id: None,
        pr_url: None,
        pr_state: None,
        worktree_path: None,
        base_commit: None,
        head_commit: None,
        plane_sub_issue_id: None,
        ..sample_work_package(1, 1)
    };
    let value = agileplus_git::materialize::render_wp_json(&wp);
    assert!(value["agent_id"].is_null());
    assert!(value["pr_url"].is_null());
    assert!(value["pr_state"].is_null());
    assert!(value["worktree_path"].is_null());
    assert!(value["base_commit"].is_null());
    assert!(value["head_commit"].is_null());
}

// ---------------------------------------------------------------------------
// materialize::materialize_feature integration
// ---------------------------------------------------------------------------

#[test]
fn materialize_feature_creates_files_on_disk() {
    let (dir, adapter) = setup_test_repo();
    let feature = sample_feature("login");
    let wp = sample_work_package(1, 1);

    agileplus_git::materialize::materialize_feature(&adapter, &feature, &[])
        .expect("materialize_feature");
    agileplus_git::materialize::materialize_work_package(&adapter, &feature.slug, &wp)
        .expect("materialize_work_package");

    let base = dir.path().join("docs").join("agileplus").join("login");
    assert!(base.join("meta.json").is_file(), "meta.json should exist");
    assert!(base.join("status.md").is_file(), "status.md should exist");
    assert!(
        base.join("audit.jsonl").is_file(),
        "audit.jsonl should exist"
    );
    let wp_file = base.join("work-packages").join("1.json");
    assert!(wp_file.is_file(), "work package JSON should exist");
}

#[test]
fn materialize_feature_multiple_work_packages() {
    let (dir, adapter) = setup_test_repo();
    let feature = sample_feature("multi-wp");
    let wp1 = sample_work_package(1, 1);
    let wp2 = sample_work_package(1, 2);
    let wp3 = sample_work_package(1, 3);

    agileplus_git::materialize::materialize_feature(&adapter, &feature, &[])
        .expect("materialize_feature");
    for wp in [&wp1, &wp2, &wp3] {
        agileplus_git::materialize::materialize_work_package(&adapter, &feature.slug, wp)
            .expect("materialize_work_package");
    }

    let wp_dir = dir
        .path()
        .join("docs")
        .join("agileplus")
        .join("multi-wp")
        .join("work-packages");
    assert!(wp_dir.join("1.json").is_file());
    assert!(wp_dir.join("2.json").is_file());
    assert!(wp_dir.join("3.json").is_file());
}

// ---------------------------------------------------------------------------
// materialize::commit_materialization
// ---------------------------------------------------------------------------

#[test]
fn commit_materialization_commits_files() {
    let (_dir, adapter) = setup_test_repo();
    let feature = sample_feature("committed");
    let wps = vec![sample_work_package(1, 1)];

    agileplus_git::materialize::materialize_feature(&adapter, &feature, &wps).expect("materialize");
    let oid = agileplus_git::materialize::commit_materialization(
        &adapter,
        "committed",
        Some("materialize committed"),
    )
    .expect("commit");
    assert!(!oid.is_empty(), "commit OID should not be empty");
}

// ---------------------------------------------------------------------------
// ProjectContext tests
// ---------------------------------------------------------------------------

#[test]
fn project_context_discover_from_subdir() {
    let dir = tempfile::tempdir().unwrap();
    git2::Repository::init(dir.path()).unwrap();
    let nested = dir.path().join("a").join("b").join("c");
    std::fs::create_dir_all(&nested).unwrap();

    let ctx = agileplus_git::ProjectContext::discover(&nested).unwrap();
    let expected = dir.path().canonicalize().unwrap();
    assert_eq!(ctx.repo_root(), expected);
}

#[test]
fn project_context_database_path() {
    let dir = tempfile::tempdir().unwrap();
    git2::Repository::init(dir.path()).unwrap();

    let ctx = agileplus_git::ProjectContext::discover(dir.path()).unwrap();
    let db = ctx.database_path();
    assert!(db.to_string_lossy().contains("agileplus.db"));
    assert!(db.to_string_lossy().contains(".agileplus"));
}

#[test]
fn project_context_fails_outside_repo() {
    let dir = tempfile::tempdir().unwrap();
    assert!(agileplus_git::ProjectContext::discover(dir.path()).is_err());
}

// ---------------------------------------------------------------------------
// GitVcsAdapter snapshot-like behavior
// ---------------------------------------------------------------------------

#[test]
fn adapter_open_via_repository_discover() {
    let (_dir, adapter) = setup_test_repo();
    let repo = git2::Repository::discover(adapter.repo_root()).unwrap();
    assert!(repo.head().is_ok());
}

#[test]
fn adapter_open_fails_on_non_repo() {
    let dir = tempfile::tempdir().unwrap();
    // Creating an adapter on a non-repo path; open via git2::Repository::discover should fail
    let result = git2::Repository::discover(dir.path());
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// VcsPort: create branch + checkout round trip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn vcs_create_checkout_list_delete_branch() {
    let (_dir, adapter) = setup_test_repo();

    adapter
        .create_branch("feat/roundtrip", "HEAD")
        .await
        .unwrap();
    adapter.checkout_branch("feat/roundtrip").await.unwrap();

    // Use git2 directly to verify HEAD (adapter.run_git is private)
    let repo = git2::Repository::open(adapter.repo_root()).unwrap();
    let head = repo.head().unwrap();
    let shorthand = head.shorthand().unwrap_or("");
    assert_eq!(shorthand, "feat/roundtrip");

    let branches = adapter
        .list_branches(Some("feat/roundtrip"), false)
        .await
        .unwrap();
    assert_eq!(branches.len(), 1);

    // Checkout back to main before deleting
    adapter.checkout_branch("main").await.unwrap();
    adapter
        .delete_branch("feat/roundtrip", false, None)
        .await
        .unwrap();

    let branches = adapter
        .list_branches(Some("feat/roundtrip"), false)
        .await
        .unwrap();
    assert!(branches.is_empty());
}

// ---------------------------------------------------------------------------
// VcsPort: merge with conflict round trip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn vcs_merge_conflict_reports_conflicting_files() {
    let (dir, adapter) = setup_test_repo();

    // Create two diverging branches
    adapter.create_branch("branch-a", "HEAD").await.unwrap();
    adapter.create_branch("branch-b", "HEAD").await.unwrap();

    // Branch A: modify README
    adapter.checkout_branch("branch-a").await.unwrap();
    let repo = Repository::open(dir.path()).unwrap();
    make_commit(&repo, dir.path(), "README.md", "version A\n", "A commit");
    drop(repo);

    // Branch B: modify README differently
    adapter.checkout_branch("branch-b").await.unwrap();
    let repo = Repository::open(dir.path()).unwrap();
    make_commit(&repo, dir.path(), "README.md", "version B\n", "B commit");
    drop(repo);

    // Merge A into main first
    adapter.checkout_branch("main").await.unwrap();
    adapter.merge_to_target("branch-a", "main").await.unwrap();

    // Now merge B (should conflict)
    let result = adapter.merge_to_target("branch-b", "main").await.unwrap();
    assert!(!result.success);
    assert!(result.conflicts.iter().any(|c| c.file_path == "README.md"));

    // Cleanup merge state via git2 directly (adapter.run_git is private)
    let repo = git2::Repository::open(adapter.repo_root()).unwrap();
    repo.cleanup_state().ok();
}

// ---------------------------------------------------------------------------
// VcsPort: worktree operations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn vcs_worktree_lifecycle() {
    let (_dir, adapter) = setup_test_repo();

    let wt_path = adapter.create_worktree("lifecycle", "wp-99").await.unwrap();
    assert!(wt_path.exists());
    assert!(wt_path.join(".git").exists());

    let worktrees = adapter.list_worktrees().await.unwrap();
    assert!(worktrees.iter().any(|w| w.path == wt_path));

    adapter.cleanup_worktree(&wt_path).await.unwrap();
    assert!(!wt_path.exists());

    let worktrees = adapter.list_worktrees().await.unwrap();
    assert!(!worktrees.iter().any(|w| w.path == wt_path));
}

// ---------------------------------------------------------------------------
// VcsPort: artifact operations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn vcs_artifact_write_read_exists_scan() {
    let (_dir, adapter) = setup_test_repo();

    // Nonexistent artifact
    assert!(
        !adapter
            .artifact_exists("test-feat", "spec.md")
            .await
            .unwrap()
    );
    assert!(adapter.read_artifact("test-feat", "spec.md").await.is_err());

    // Write
    adapter
        .write_artifact("test-feat", "spec.md", "# Spec\n")
        .await
        .unwrap();
    assert!(
        adapter
            .artifact_exists("test-feat", "spec.md")
            .await
            .unwrap()
    );

    // Read
    let content = adapter.read_artifact("test-feat", "spec.md").await.unwrap();
    assert_eq!(content, "# Spec\n");

    // Scan
    adapter
        .write_artifact("test-feat", "meta.json", r#"{"slug":"test"}"#)
        .await
        .unwrap();
    let scan = adapter.scan_feature_artifacts("test-feat").await.unwrap();
    assert!(scan.spec.is_some());
    assert!(scan.meta_json.is_some());
}

// ---------------------------------------------------------------------------
// VcsPort: scan_all_features via public API
// ---------------------------------------------------------------------------

#[test]
fn scan_all_features_via_adapter() {
    let (dir, adapter) = setup_test_repo();

    for slug in &["feat-a", "feat-b"] {
        let meta = dir
            .path()
            .join("docs")
            .join("agileplus")
            .join(slug)
            .join("meta.json");
        std::fs::create_dir_all(meta.parent().unwrap()).unwrap();
        std::fs::write(&meta, "{}").unwrap();
    }

    let slugs = agileplus_git::scan_all_features(&adapter).unwrap();
    assert_eq!(slugs, vec!["feat-a", "feat-b"]);
}
