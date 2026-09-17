//! Document materialization: renders domain entities as git-tracked files.
//! One-way sync: SSOT -> git. Direct edits to materialized files are overwritten.
//!
//! Writes the following layout under `docs/agileplus/<slug>/`:
//!
//! ```text
//! docs/agileplus/<slug>/
//!   meta.json        -- Feature metadata (slug, name, state, git provenance)
//!   status.md        -- Human-readable status with WP table
//!   audit.jsonl      -- Append-only materialization audit log
//!   work-packages/
//!     <wp_id>.json   -- One file per WorkPackage
//! ```
//!
//! Traceability: L4 SSOT materialization layer

use std::{
    io::Write as IoWrite,
    path::{Path, PathBuf},
};

use agileplus_domain::{
    domain::{feature::Feature, work_package::WorkPackage},
    error::DomainError,
};
use chrono::Utc;
use serde_json::{Value, json};

use crate::GitVcsAdapter;

fn open_repo(adapter: &GitVcsAdapter) -> Result<(git2::Repository, PathBuf), DomainError> {
    let repo = git2::Repository::discover(adapter.repo_root()).map_err(|e| {
        DomainError::Storage(format!(
            "failed to open git repository at {}: {e}",
            adapter.repo_root().display()
        ))
    })?;
    let repo_root = repo.workdir().map(Path::to_path_buf).ok_or_else(|| {
        DomainError::Storage("materialization requires a non-bare worktree".into())
    })?;
    Ok((repo, repo_root))
}

fn git_error(error: git2::Error) -> DomainError {
    DomainError::Storage(error.to_string())
}

// ---------------------------------------------------------------------------
// Pure rendering functions (no I/O, easily testable)
// ---------------------------------------------------------------------------

/// Render `meta.json` content for a feature.
pub fn render_meta_json(feature: &Feature) -> Value {
    json!({
        "slug": feature.slug,
        "friendly_name": feature.friendly_name,
        "state": feature.state.to_string(),
        "target_branch": feature.target_branch,
        "spec_hash": feature.spec_hash.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        "created_at": feature.created_at.to_rfc3339(),
        "updated_at": feature.updated_at.to_rfc3339(),
        "created_at_commit": feature.created_at_commit,
        "last_modified_commit": feature.last_modified_commit,
        "labels": feature.labels,
        "plane_issue_id": feature.plane_issue_id,
        "materialized_at": Utc::now().to_rfc3339(),
    })
}

/// Render `status.md` content for a feature with its work packages.
pub fn render_status_md(feature: &Feature, work_packages: &[WorkPackage]) -> String {
    let mut out = String::new();

    out.push_str(&format!("# {}\n\n", feature.friendly_name));
    out.push_str(&format!("**Slug**: `{}`  \n", feature.slug));
    out.push_str(&format!("**State**: {}  \n", feature.state));
    out.push_str(&format!(
        "**Target branch**: `{}`  \n",
        feature.target_branch
    ));
    out.push_str(&format!(
        "**Updated**: {}  \n",
        feature.updated_at.to_rfc3339()
    ));

    if !feature.labels.is_empty() {
        out.push_str(&format!("**Labels**: {}  \n", feature.labels.join(", ")));
    }

    out.push('\n');
    out.push_str("---\n\n");

    if work_packages.is_empty() {
        out.push_str("*No work packages.*\n");
    } else {
        out.push_str("## Work Packages\n\n");
        out.push_str("| ID | Seq | Title | State | Agent | Branch |\n");
        out.push_str("|----|-----|-------|-------|-------|--------|\n");

        for wp in work_packages {
            let agent = wp.agent_id.as_deref().unwrap_or("-");
            let branch = wp.worktree_path.as_deref().unwrap_or("-");
            let state_str = format!("{:?}", wp.state).to_lowercase();
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                wp.id, wp.sequence, wp.title, state_str, agent, branch,
            ));
        }
    }

    out.push('\n');
    out.push_str(&format!(
        "> *Materialized at {}. Do not edit directly -- changes will be overwritten.*\n",
        Utc::now().to_rfc3339(),
    ));

    out
}

/// Render a single audit JSONL line (one JSON object, no newline at start).
pub fn render_audit_line(feature: &Feature, commit_oid: Option<&str>) -> String {
    let entry = json!({
        "timestamp": Utc::now().to_rfc3339(),
        "action": "materialized",
        "slug": feature.slug,
        "state": feature.state.to_string(),
        "commit": commit_oid,
    });
    // serde_json::to_string never fails for a well-formed Value.
    serde_json::to_string(&entry).expect("audit line serialization failed")
}

/// Render `wp/<id>.json` content for a work package.
pub fn render_wp_json(wp: &WorkPackage) -> Value {
    json!({
        "id": wp.id,
        "feature_id": wp.feature_id,
        "title": wp.title,
        "state": format!("{:?}", wp.state).to_lowercase(),
        "sequence": wp.sequence,
        "acceptance_criteria": wp.acceptance_criteria,
        "file_scope": wp.file_scope,
        "agent_id": wp.agent_id,
        "pr_url": wp.pr_url,
        "pr_state": wp.pr_state.map(|s| format!("{s:?}").to_lowercase()),
        "worktree_path": wp.worktree_path,
        "base_commit": wp.base_commit,
        "head_commit": wp.head_commit,
        "created_at": wp.created_at.to_rfc3339(),
        "updated_at": wp.updated_at.to_rfc3339(),
        "materialized_at": Utc::now().to_rfc3339(),
    })
}

// ---------------------------------------------------------------------------
// Git staging helpers
// ---------------------------------------------------------------------------

/// Write `content` to `full_path`, creating parent dirs, and stage the file.
fn write_and_stage(
    repo: &git2::Repository,
    repo_root: &Path,
    full_path: &Path,
    content: &[u8],
) -> Result<(), DomainError> {
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            DomainError::Storage(format!("create dirs for {}: {e}", full_path.display()))
        })?;
    }

    std::fs::write(full_path, content)
        .map_err(|e| DomainError::Storage(format!("write {}: {e}", full_path.display())))?;

    let relative = full_path
        .strip_prefix(repo_root)
        .map_err(|_| DomainError::Storage("materialized file outside repo root".into()))?;

    let mut index = repo.index().map_err(git_error)?;
    index.add_path(relative).map_err(git_error)?;
    index.write().map_err(git_error)?;

    Ok(())
}

/// Append `line` to an append-only JSONL file and stage it.
///
/// Creates the file if it does not exist. Each call appends exactly one line.
fn append_and_stage(
    repo: &git2::Repository,
    repo_root: &Path,
    full_path: &Path,
    line: &str,
) -> Result<(), DomainError> {
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            DomainError::Storage(format!("create dirs for {}: {e}", full_path.display()))
        })?;
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(full_path)
        .map_err(|e| DomainError::Storage(format!("open audit file: {e}")))?;

    writeln!(file, "{line}")
        .map_err(|e| DomainError::Storage(format!("append audit line: {e}")))?;

    let relative = full_path
        .strip_prefix(repo_root)
        .map_err(|_| DomainError::Storage("audit file outside repo root".into()))?;

    let mut index = repo.index().map_err(git_error)?;
    index.add_path(relative).map_err(git_error)?;
    index.write().map_err(git_error)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Materialize a feature and its work packages as git-tracked files.
///
/// Writes `meta.json`, `status.md`, and appends a line to `audit.jsonl` under
/// `docs/agileplus/<slug>/`, staging all modified files in the git index.
///
/// Returns the feature directory path (`docs/agileplus/<slug>/`).
pub fn materialize_feature(
    adapter: &GitVcsAdapter,
    feature: &Feature,
    work_packages: &[WorkPackage],
) -> Result<PathBuf, DomainError> {
    let (repo, repo_root) = open_repo(adapter)?;
    let feature_dir = repo_root.join("docs").join("agileplus").join(&feature.slug);

    // meta.json (overwrite every time -- SSOT wins)
    let meta_content = serde_json::to_string_pretty(&render_meta_json(feature))
        .map_err(|e| DomainError::Storage(format!("serialize meta.json: {e}")))?;
    write_and_stage(
        &repo,
        &repo_root,
        &feature_dir.join("meta.json"),
        meta_content.as_bytes(),
    )?;

    // status.md (overwrite every time)
    let status_content = render_status_md(feature, work_packages);
    write_and_stage(
        &repo,
        &repo_root,
        &feature_dir.join("status.md"),
        status_content.as_bytes(),
    )?;

    // audit.jsonl (append-only)
    let audit_line = render_audit_line(feature, feature.last_modified_commit.as_deref());
    append_and_stage(
        &repo,
        &repo_root,
        &feature_dir.join("audit.jsonl"),
        &audit_line,
    )?;

    Ok(feature_dir)
}

/// Materialize a single work package as a git-tracked JSON file.
///
/// Writes `docs/agileplus/<feature_slug>/work-packages/<wp_id>.json` and stages it.
pub fn materialize_work_package(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
    wp: &WorkPackage,
) -> Result<(), DomainError> {
    let (repo, repo_root) = open_repo(adapter)?;
    let wp_path = repo_root
        .join("docs")
        .join("agileplus")
        .join(feature_slug)
        .join("work-packages")
        .join(format!("{}.json", wp.id));

    let content = serde_json::to_string_pretty(&render_wp_json(wp))
        .map_err(|e| DomainError::Storage(format!("serialize wp {}: {e}", wp.id)))?;

    write_and_stage(&repo, &repo_root, &wp_path, content.as_bytes())?;

    Ok(())
}

/// Commit all currently staged materialization files.
///
/// Uses the repository's default signature when available, falling back to
/// "agileplus-bot <agileplus-bot@localhost>".
///
/// Returns the new commit OID as a hex string.
pub fn commit_materialization(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
    message: Option<&str>,
) -> Result<String, DomainError> {
    let (repo, _) = open_repo(adapter)?;

    let default_msg = format!("chore(specs): materialize {feature_slug} artifacts");
    let commit_msg = message.unwrap_or(&default_msg);

    // Resolve signature: try repo config, fall back to bot identity.
    let sig = repo.signature().unwrap_or_else(|_| {
        git2::Signature::now("agileplus-bot", "agileplus-bot@localhost")
            .expect("static signature is always valid")
    });

    // Write the current index to a tree.
    let mut index = repo.index().map_err(git_error)?;
    let tree_oid = index.write_tree().map_err(git_error)?;
    let tree = repo.find_tree(tree_oid).map_err(git_error)?;

    // Determine parent commit (HEAD), if any.
    let parent_commit = match repo.head() {
        Ok(head_ref) => {
            let head_oid = head_ref
                .target()
                .ok_or_else(|| DomainError::Storage("HEAD reference has no target OID".into()))?;
            Some(repo.find_commit(head_oid).map_err(git_error)?)
        }
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch => None,
        Err(e) => return Err(git_error(e)),
    };

    let parents: Vec<&git2::Commit<'_>> = parent_commit.iter().collect();

    let commit_oid = repo
        .commit(Some("HEAD"), &sig, &sig, commit_msg, &tree, &parents)
        .map_err(git_error)?;

    Ok(commit_oid.to_string())
}

/// Convenience: materialize feature + all work packages, then commit.
///
/// Equivalent to calling `materialize_feature`, `materialize_work_package` for
/// each WP, and finally `commit_materialization`. Returns the commit OID.
pub fn materialize_and_commit(
    adapter: &GitVcsAdapter,
    feature: &Feature,
    work_packages: &[WorkPackage],
) -> Result<String, DomainError> {
    materialize_feature(adapter, feature, work_packages)?;

    for wp in work_packages {
        materialize_work_package(adapter, &feature.slug, wp)?;
    }

    commit_materialization(adapter, &feature.slug, None)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::{
        feature::Feature,
        work_package::{WorkPackage, WpState},
    };

    fn make_feature() -> Feature {
        let mut f = Feature::new("my-feature", "My Feature", [0xabu8; 32], Some("main"));
        f.id = 1;
        f.labels = vec!["alpha".into(), "beta".into()];
        f.last_modified_commit = Some("deadbeef".into());
        f
    }

    fn make_wp(id: i64, title: &str) -> WorkPackage {
        let mut wp = WorkPackage::new(1, title, 1, "It must work.");
        wp.id = id;
        wp.agent_id = Some("agent-007".into());
        wp.worktree_path = Some("worktrees/my-feature-wp1".into());
        wp
    }

    // --- render_meta_json ---

    #[test]
    fn render_meta_json_contains_slug() {
        let f = make_feature();
        let v = render_meta_json(&f);
        assert_eq!(v["slug"], "my-feature");
    }

    #[test]
    fn render_meta_json_state_as_string() {
        let f = make_feature();
        let v = render_meta_json(&f);
        assert_eq!(v["state"], "created");
    }

    #[test]
    fn render_meta_json_spec_hash_hex() {
        let f = make_feature();
        let v = render_meta_json(&f);
        let hex = v["spec_hash"].as_str().unwrap();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn render_meta_json_last_modified_commit() {
        let f = make_feature();
        let v = render_meta_json(&f);
        assert_eq!(v["last_modified_commit"], "deadbeef");
    }

    // --- render_status_md ---

    #[test]
    fn render_status_md_contains_title() {
        let f = make_feature();
        let md = render_status_md(&f, &[]);
        assert!(md.contains("My Feature"));
    }

    #[test]
    fn render_status_md_no_wps_shows_placeholder() {
        let f = make_feature();
        let md = render_status_md(&f, &[]);
        assert!(md.contains("No work packages"));
    }

    #[test]
    fn render_status_md_wp_table() {
        let f = make_feature();
        let wp = make_wp(42, "Implement login");
        let md = render_status_md(&f, &[wp]);
        assert!(md.contains("Implement login"));
        assert!(md.contains("42"));
        assert!(md.contains("agent-007"));
        assert!(md.contains("planned"));
    }

    #[test]
    fn render_status_md_multiple_wps() {
        let f = make_feature();
        let wp1 = make_wp(1, "First WP");
        let wp2 = make_wp(2, "Second WP");
        let md = render_status_md(&f, &[wp1, wp2]);
        assert!(md.contains("First WP"));
        assert!(md.contains("Second WP"));
    }

    #[test]
    fn render_status_md_labels() {
        let f = make_feature();
        let md = render_status_md(&f, &[]);
        assert!(md.contains("alpha"));
        assert!(md.contains("beta"));
    }

    #[test]
    fn render_status_md_overwrite_warning() {
        let f = make_feature();
        let md = render_status_md(&f, &[]);
        assert!(md.contains("Do not edit directly"));
    }

    // --- render_audit_line ---

    #[test]
    fn render_audit_line_is_valid_json() {
        let f = make_feature();
        let line = render_audit_line(&f, Some("abc123"));
        let v: Value = serde_json::from_str(&line).expect("audit line must be valid JSON");
        assert_eq!(v["action"], "materialized");
        assert_eq!(v["slug"], "my-feature");
        assert_eq!(v["commit"], "abc123");
    }

    #[test]
    fn render_audit_line_no_newline_in_value() {
        let f = make_feature();
        let line = render_audit_line(&f, None);
        // The rendered string itself should not contain embedded newlines (JSONL invariant).
        assert!(!line.contains('\n'));
    }

    #[test]
    fn render_audit_line_null_commit_when_none() {
        let f = make_feature();
        let line = render_audit_line(&f, None);
        let v: Value = serde_json::from_str(&line).unwrap();
        assert!(v["commit"].is_null());
    }

    // --- render_wp_json ---

    #[test]
    fn render_wp_json_contains_id() {
        let wp = make_wp(99, "My WP");
        let v = render_wp_json(&wp);
        assert_eq!(v["id"], 99);
    }

    #[test]
    fn render_wp_json_state_lowercase() {
        let wp = make_wp(1, "WP");
        let v = render_wp_json(&wp);
        assert_eq!(v["state"], "planned");
    }

    #[test]
    fn render_wp_json_doing_state() {
        let mut wp = make_wp(1, "WP");
        wp.state = WpState::Doing;
        let v = render_wp_json(&wp);
        assert_eq!(v["state"], "doing");
    }

    #[test]
    fn render_wp_json_agent_id() {
        let wp = make_wp(1, "WP");
        let v = render_wp_json(&wp);
        assert_eq!(v["agent_id"], "agent-007");
    }

    #[test]
    fn render_wp_json_null_pr_state_when_none() {
        let wp = make_wp(1, "WP");
        let v = render_wp_json(&wp);
        assert!(v["pr_state"].is_null());
    }

    #[test]
    fn render_wp_json_file_scope() {
        let mut wp = make_wp(1, "WP");
        wp.file_scope = vec!["src/main.rs".into(), "src/lib.rs".into()];
        let v = render_wp_json(&wp);
        let scope = v["file_scope"].as_array().unwrap();
        assert_eq!(scope.len(), 2);
        assert_eq!(scope[0], "src/main.rs");
    }

    // --- git integration tests (require tempfile) ---

    /// Create a bare-minimum initial commit in a freshly initialised repo so
    /// HEAD is valid and the index is ready for staging operations.
    fn make_initial_commit(repo: &git2::Repository) {
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        let tree_oid = {
            let mut idx = repo.index().unwrap();
            idx.write_tree().unwrap()
        };
        // tree is in its own block so it's dropped before the function returns.
        {
            let tree = repo.find_tree(tree_oid).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
                .unwrap();
        }
    }

    #[test]
    fn materialize_feature_creates_human_artifacts_under_docs_agileplus() {
        let tmp = tempfile::TempDir::new().unwrap();
        {
            let repo = git2::Repository::init(tmp.path()).unwrap();
            make_initial_commit(&repo);
        }

        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf());
        let feature = make_feature();
        let wp = make_wp(1, "WP1");

        let dir = materialize_feature(&adapter, &feature, &[wp]).unwrap();
        assert_eq!(
            dir,
            std::fs::canonicalize(tmp.path().join("docs").join("agileplus").join("my-feature"),)
                .unwrap()
        );
        assert!(dir.join("meta.json").exists());
        assert!(dir.join("status.md").exists());
        assert!(dir.join("audit.jsonl").exists());
    }

    #[test]
    fn materialize_work_package_creates_file_under_feature_work_packages_directory() {
        let tmp = tempfile::TempDir::new().unwrap();
        {
            let repo = git2::Repository::init(tmp.path()).unwrap();
            make_initial_commit(&repo);
        }

        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf());
        let wp = make_wp(7, "My WP");
        materialize_work_package(&adapter, "my-feature", &wp).unwrap();

        let wp_path = tmp
            .path()
            .join("docs")
            .join("agileplus")
            .join("my-feature")
            .join("work-packages")
            .join("7.json");
        assert!(wp_path.exists());

        let content = std::fs::read_to_string(&wp_path).unwrap();
        let v: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(v["id"], 7);
    }

    #[test]
    fn materialize_and_commit_returns_oid() {
        let tmp = tempfile::TempDir::new().unwrap();
        {
            let repo = git2::Repository::init(tmp.path()).unwrap();
            make_initial_commit(&repo);
        }

        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf());
        let feature = make_feature();
        let wp = make_wp(1, "WP1");

        let oid = materialize_and_commit(&adapter, &feature, &[wp]).unwrap();
        // A git OID is a 40-character hex string.
        assert_eq!(oid.len(), 40);
        assert!(oid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn audit_jsonl_appends_across_calls() {
        let tmp = tempfile::TempDir::new().unwrap();
        {
            let repo = git2::Repository::init(tmp.path()).unwrap();
            make_initial_commit(&repo);
        }

        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf());
        let feature = make_feature();

        materialize_feature(&adapter, &feature, &[]).unwrap();
        materialize_feature(&adapter, &feature, &[]).unwrap();

        let audit_path = tmp
            .path()
            .join("docs")
            .join("agileplus")
            .join("my-feature")
            .join("audit.jsonl");
        let content = std::fs::read_to_string(&audit_path).unwrap();
        let line_count = content.lines().count();
        assert_eq!(line_count, 2, "each materialization appends one audit line");
    }

    // -----------------------------------------------------------------------
    // Additional tests — coverage for untested paths
    // -----------------------------------------------------------------------

    #[test]
    fn render_status_md_no_labels_omits_label_line() {
        let mut f = make_feature();
        f.labels.clear();
        let md = render_status_md(&f, &[]);
        assert!(!md.contains("**Labels**"));
    }

    #[test]
    fn render_status_md_empty_labels_omits_label_line() {
        let mut f = make_feature();
        f.labels.clear();
        let md = render_status_md(&f, &[]);
        assert!(!md.contains("**Labels**"));
    }

    #[test]
    fn render_status_md_target_branch() {
        let f = make_feature();
        let md = render_status_md(&f, &[]);
        assert!(md.contains("main"));
        assert!(md.contains("**Target branch**"));
    }

    #[test]
    fn render_status_md_updated_timestamp() {
        let f = make_feature();
        let md = render_status_md(&f, &[]);
        assert!(md.contains("**Updated**"));
    }

    #[test]
    fn render_status_md_wp_without_agent_shows_dash() {
        let f = make_feature();
        let mut wp = make_wp(1, "No Agent WP");
        wp.agent_id = None;
        let md = render_status_md(&f, &[wp]);
        // Agent column should show "-" when None
        let lines: Vec<&str> = md.lines().collect();
        let data_line = lines.iter().find(|l| l.contains("No Agent WP")).unwrap();
        assert!(data_line.contains("| - |"));
    }

    #[test]
    fn render_status_md_wp_without_worktree_shows_dash() {
        let f = make_feature();
        let mut wp = make_wp(1, "No Branch WP");
        wp.worktree_path = None;
        let md = render_status_md(&f, &[wp]);
        let lines: Vec<&str> = md.lines().collect();
        let data_line = lines.iter().find(|l| l.contains("No Branch WP")).unwrap();
        // Branch column should be at the end
        assert!(data_line.ends_with(" - |"));
    }

    #[test]
    fn render_meta_json_empty_labels() {
        let mut f = make_feature();
        f.labels.clear();
        let v = render_meta_json(&f);
        let labels = v["labels"].as_array().unwrap();
        assert!(labels.is_empty());
    }

    #[test]
    fn render_meta_json_with_plane_issue_id() {
        let mut f = make_feature();
        f.plane_issue_id = Some("PLN-123".into());
        let v = render_meta_json(&f);
        assert_eq!(v["plane_issue_id"], "PLN-123");
    }

    #[test]
    fn render_meta_json_without_plane_issue_id() {
        let f = make_feature();
        let v = render_meta_json(&f);
        assert!(v["plane_issue_id"].is_null());
    }

    #[test]
    fn render_meta_json_created_at_commit() {
        let mut f = make_feature();
        f.created_at_commit = Some("abc123".into());
        let v = render_meta_json(&f);
        assert_eq!(v["created_at_commit"], "abc123");
    }

    #[test]
    fn render_meta_json_target_branch() {
        let f = make_feature();
        let v = render_meta_json(&f);
        assert_eq!(v["target_branch"], "main");
    }

    #[test]
    fn render_wp_json_with_pr_state() {
        let mut wp = make_wp(1, "WP");
        wp.pr_state = Some(agileplus_domain::domain::work_package::PrState::Approved);
        let v = render_wp_json(&wp);
        assert_eq!(v["pr_state"], "approved");
    }

    #[test]
    fn render_wp_json_pr_state_variants() {
        use agileplus_domain::domain::work_package::PrState;
        for (state, expected) in [
            (PrState::Open, "open"),
            (PrState::Review, "review"),
            (PrState::ChangesRequested, "changesrequested"),
            (PrState::Approved, "approved"),
            (PrState::Merged, "merged"),
        ] {
            let mut wp = make_wp(1, "WP");
            wp.pr_state = Some(state);
            let v = render_wp_json(&wp);
            assert_eq!(v["pr_state"], expected);
        }
    }

    #[test]
    fn render_wp_json_without_worktree_path() {
        let mut wp = make_wp(1, "WP");
        wp.worktree_path = None;
        let v = render_wp_json(&wp);
        assert!(v["worktree_path"].is_null());
    }

    #[test]
    fn render_wp_json_acceptance_criteria() {
        let wp = make_wp(1, "WP");
        let v = render_wp_json(&wp);
        assert_eq!(v["acceptance_criteria"], "It must work.");
    }

    #[test]
    fn render_wp_json_base_and_head_commit() {
        let mut wp = make_wp(1, "WP");
        wp.base_commit = Some("aaa111".into());
        wp.head_commit = Some("bbb222".into());
        let v = render_wp_json(&wp);
        assert_eq!(v["base_commit"], "aaa111");
        assert_eq!(v["head_commit"], "bbb222");
    }

    #[test]
    fn render_wp_json_without_base_head_commit() {
        let wp = make_wp(1, "WP");
        let v = render_wp_json(&wp);
        assert!(v["base_commit"].is_null());
        assert!(v["head_commit"].is_null());
    }

    #[test]
    fn render_wp_json_materialized_at_present() {
        let wp = make_wp(1, "WP");
        let v = render_wp_json(&wp);
        assert!(v["materialized_at"].is_string());
    }

    #[test]
    fn render_wp_json_feature_id() {
        let wp = make_wp(1, "WP");
        let v = render_wp_json(&wp);
        assert_eq!(v["feature_id"], 1);
    }

    #[test]
    fn render_audit_line_with_commit() {
        let f = make_feature();
        let line = render_audit_line(&f, Some("deadbeef"));
        let v: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(v["commit"], "deadbeef");
    }

    #[test]
    fn render_audit_line_has_timestamp() {
        let f = make_feature();
        let line = render_audit_line(&f, None);
        let v: Value = serde_json::from_str(&line).unwrap();
        assert!(v["timestamp"].is_string());
    }

    #[test]
    fn render_audit_line_has_state() {
        let f = make_feature();
        let line = render_audit_line(&f, None);
        let v: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(v["state"], "created");
    }

    // --- git integration: error paths ---

    #[test]
    fn materialize_feature_on_non_repo_fails() {
        let tmp = tempfile::TempDir::new().unwrap();
        // No git init
        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf());
        let f = make_feature();
        let result = materialize_feature(&adapter, &f, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn materialize_work_package_on_non_repo_fails() {
        let tmp = tempfile::TempDir::new().unwrap();
        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf());
        let wp = make_wp(1, "WP");
        let result = materialize_work_package(&adapter, "my-feature", &wp);
        assert!(result.is_err());
    }

    #[test]
    fn commit_materialization_on_non_repo_fails() {
        let tmp = tempfile::TempDir::new().unwrap();
        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf());
        let result = commit_materialization(&adapter, "my-feature", None);
        assert!(result.is_err());
    }

    #[test]
    fn materialize_feature_with_multiple_wps() {
        let tmp = tempfile::TempDir::new().unwrap();
        {
            let repo = git2::Repository::init(tmp.path()).unwrap();
            make_initial_commit(&repo);
        }
        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf());
        let f = make_feature();
        let wp1 = make_wp(1, "WP One");
        let wp2 = make_wp(2, "WP Two");
        let wp3 = make_wp(3, "WP Three");
        materialize_feature(&adapter, &f, &[wp1, wp2, wp3]).unwrap();
        let md = std::fs::read_to_string(
            tmp.path()
                .join("docs")
                .join("agileplus")
                .join("my-feature")
                .join("status.md"),
        )
        .unwrap();
        assert!(md.contains("WP One"));
        assert!(md.contains("WP Two"));
        assert!(md.contains("WP Three"));
    }

    #[test]
    fn commit_materialization_with_custom_message() {
        let tmp = tempfile::TempDir::new().unwrap();
        {
            let repo = git2::Repository::init(tmp.path()).unwrap();
            make_initial_commit(&repo);
        }
        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf());
        let f = make_feature();
        materialize_feature(&adapter, &f, &[]).unwrap();
        let oid = commit_materialization(&adapter, &f.slug, Some("custom msg")).unwrap();
        assert_eq!(oid.len(), 40);

        // Verify the commit message
        let repo = git2::Repository::open(tmp.path()).unwrap();
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        assert_eq!(commit.message().unwrap(), "custom msg");
    }

    #[test]
    fn commit_materialization_default_message() {
        let tmp = tempfile::TempDir::new().unwrap();
        {
            let repo = git2::Repository::init(tmp.path()).unwrap();
            make_initial_commit(&repo);
        }
        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf());
        let f = make_feature();
        materialize_feature(&adapter, &f, &[]).unwrap();
        let oid = commit_materialization(&adapter, &f.slug, None).unwrap();
        assert_eq!(oid.len(), 40);

        let repo = git2::Repository::open(tmp.path()).unwrap();
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        assert!(commit.message().unwrap().contains("my-feature"));
    }

    #[test]
    fn materialize_and_commit_with_multiple_wps() {
        let tmp = tempfile::TempDir::new().unwrap();
        {
            let repo = git2::Repository::init(tmp.path()).unwrap();
            make_initial_commit(&repo);
        }
        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf());
        let f = make_feature();
        let wp1 = make_wp(10, "First");
        let wp2 = make_wp(20, "Second");
        let oid = materialize_and_commit(&adapter, &f, &[wp1, wp2]).unwrap();
        assert_eq!(oid.len(), 40);

        // Verify WP files exist
        let wp1_path = tmp
            .path()
            .join("docs/agileplus/my-feature/work-packages/10.json");
        let wp2_path = tmp
            .path()
            .join("docs/agileplus/my-feature/work-packages/20.json");
        assert!(wp1_path.exists());
        assert!(wp2_path.exists());
    }

    #[test]
    fn render_meta_json_materialized_at_is_present() {
        let f = make_feature();
        let v = render_meta_json(&f);
        assert!(v["materialized_at"].is_string());
    }

    #[test]
    fn render_wp_json_pr_url() {
        let mut wp = make_wp(1, "WP");
        wp.pr_url = Some("https://github.com/org/repo/pull/42".into());
        let v = render_wp_json(&wp);
        assert_eq!(v["pr_url"], "https://github.com/org/repo/pull/42");
    }

    #[test]
    fn render_wp_json_null_pr_url_when_none() {
        let wp = make_wp(1, "WP");
        let v = render_wp_json(&wp);
        assert!(v["pr_url"].is_null());
    }

    #[test]
    fn render_wp_json_title() {
        let wp = make_wp(42, "Fix the thing");
        let v = render_wp_json(&wp);
        assert_eq!(v["title"], "Fix the thing");
    }

    #[test]
    fn render_wp_json_sequence() {
        let wp = make_wp(1, "WP");
        let v = render_wp_json(&wp);
        assert_eq!(v["sequence"], 1);
    }
}

#[cfg(test)]
mod deep_tests {
    use super::*;
    use agileplus_domain::domain::{
        feature::Feature,
        work_package::{PrState, WorkPackage, WpState},
    };
    use std::process::Command as StdCommand;

    fn feature() -> Feature {
        let mut f = Feature::new("deep-feature", "Deep Feature", [0u8; 32], None);
        f.id = 7;
        f.labels = vec!["one".into(), "two".into()];
        f.plane_issue_id = Some("plane-42".into());
        f.created_at_commit = Some("cafe".into());
        f.last_modified_commit = Some("beef".into());
        f
    }

    fn wp() -> WorkPackage {
        let mut w = WorkPackage::new(7, "Deep WP", 3, "Criteria here.");
        w.id = 99;
        w.state = WpState::Doing;
        w.file_scope = vec!["src/a.rs".into(), "src/b.rs".into()];
        w.agent_id = Some("agent-x".into());
        w.pr_url = Some("https://example.com/pr/1".into());
        w.pr_state = Some(PrState::Open);
        w.worktree_path = Some("wt/deep".into());
        w.base_commit = Some("base".into());
        w.head_commit = Some("head".into());
        w
    }

    // ── render_meta_json ────────────────────────────────────────────────────

    #[test]
    fn meta_friendly_name() {
        assert_eq!(
            render_meta_json(&feature())["friendly_name"],
            "Deep Feature"
        );
    }

    #[test]
    fn meta_default_target_branch_is_main() {
        assert_eq!(render_meta_json(&feature())["target_branch"], "main");
    }

    #[test]
    fn meta_labels_are_array() {
        let v = render_meta_json(&feature());
        assert_eq!(v["labels"], serde_json::json!(["one", "two"]));
    }

    #[test]
    fn meta_plane_issue_id_present() {
        assert_eq!(render_meta_json(&feature())["plane_issue_id"], "plane-42");
    }

    #[test]
    fn meta_plane_issue_id_null_when_absent() {
        let mut f = feature();
        f.plane_issue_id = None;
        assert!(render_meta_json(&f)["plane_issue_id"].is_null());
    }

    #[test]
    fn meta_created_at_commit() {
        assert_eq!(render_meta_json(&feature())["created_at_commit"], "cafe");
    }

    #[test]
    fn meta_zero_spec_hash_is_all_zeros() {
        let hex = render_meta_json(&feature())["spec_hash"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(hex, "0".repeat(64));
    }

    #[test]
    fn meta_materialized_at_is_rfc3339() {
        let v = render_meta_json(&feature());
        let ts = v["materialized_at"].as_str().unwrap();
        assert!(chrono::DateTime::parse_from_rfc3339(ts).is_ok());
    }

    #[test]
    fn meta_has_expected_key_set() {
        let v = render_meta_json(&feature());
        let obj = v.as_object().unwrap();
        for key in [
            "slug",
            "friendly_name",
            "state",
            "target_branch",
            "spec_hash",
            "created_at",
            "updated_at",
            "created_at_commit",
            "last_modified_commit",
            "labels",
            "plane_issue_id",
            "materialized_at",
        ] {
            assert!(obj.contains_key(key), "missing key {key}");
        }
    }

    // ── render_status_md ────────────────────────────────────────────────────

    #[test]
    fn status_contains_slug_and_state() {
        let md = render_status_md(&feature(), &[]);
        assert!(md.contains("`deep-feature`"));
        assert!(md.contains("**State**: created"));
    }

    #[test]
    fn status_shows_target_branch() {
        let md = render_status_md(&feature(), &[]);
        assert!(md.contains("**Target branch**: `main`"));
    }

    #[test]
    fn status_omits_labels_line_when_empty() {
        let mut f = feature();
        f.labels.clear();
        let md = render_status_md(&f, &[]);
        assert!(!md.contains("**Labels**"));
    }

    #[test]
    fn status_wp_agent_and_branch_fallback_to_dash() {
        let mut w = wp();
        w.agent_id = None;
        w.worktree_path = None;
        let md = render_status_md(&feature(), &[w]);
        assert!(md.contains("| - | - |"));
    }

    #[test]
    fn status_wp_state_is_lowercased() {
        let md = render_status_md(&feature(), &[wp()]);
        assert!(md.contains("doing"));
        assert!(!md.contains("Doing"));
    }

    #[test]
    fn status_includes_table_header_when_wps_present() {
        let md = render_status_md(&feature(), &[wp()]);
        assert!(md.contains("| ID | Seq | Title | State | Agent | Branch |"));
        assert!(md.contains("|----|-----|-------|-------|-------|--------|"));
    }

    // ── render_audit_line ───────────────────────────────────────────────────

    #[test]
    fn audit_line_is_single_json_object() {
        let line = render_audit_line(&feature(), Some("abc"));
        assert!(!line.contains('\n'));
        let v: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(v["action"], "materialized");
        assert_eq!(v["slug"], "deep-feature");
        assert_eq!(v["state"], "created");
        assert_eq!(v["commit"], "abc");
    }

    #[test]
    fn audit_line_commit_null_when_none() {
        let line = render_audit_line(&feature(), None);
        let v: Value = serde_json::from_str(&line).unwrap();
        assert!(v["commit"].is_null());
    }

    #[test]
    fn audit_line_timestamp_is_rfc3339() {
        let line = render_audit_line(&feature(), None);
        let v: Value = serde_json::from_str(&line).unwrap();
        let ts = v["timestamp"].as_str().unwrap();
        assert!(chrono::DateTime::parse_from_rfc3339(ts).is_ok());
    }

    // ── render_wp_json ──────────────────────────────────────────────────────

    #[test]
    fn wp_json_core_fields() {
        let v = render_wp_json(&wp());
        assert_eq!(v["id"], 99);
        assert_eq!(v["feature_id"], 7);
        assert_eq!(v["title"], "Deep WP");
        assert_eq!(v["state"], "doing");
        assert_eq!(v["sequence"], 3);
        assert_eq!(v["acceptance_criteria"], "Criteria here.");
    }

    #[test]
    fn wp_json_file_scope() {
        let v = render_wp_json(&wp());
        assert_eq!(v["file_scope"], serde_json::json!(["src/a.rs", "src/b.rs"]));
    }

    #[test]
    fn wp_json_pr_state_lowercased() {
        assert_eq!(render_wp_json(&wp())["pr_state"], "open");
    }

    #[test]
    fn wp_json_pr_state_null_when_absent() {
        let mut w = wp();
        w.pr_state = None;
        assert!(render_wp_json(&w)["pr_state"].is_null());
    }

    #[test]
    fn wp_json_commit_fields() {
        let v = render_wp_json(&wp());
        assert_eq!(v["base_commit"], "base");
        assert_eq!(v["head_commit"], "head");
        assert_eq!(v["worktree_path"], "wt/deep");
    }

    #[test]
    fn wp_json_agent_null_when_absent() {
        let mut w = wp();
        w.agent_id = None;
        assert!(render_wp_json(&w)["agent_id"].is_null());
    }

    // ── End-to-end materialization ──────────────────────────────────────────

    fn make_repo() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "t@example.com"],
            vec!["config", "user.name", "tester"],
        ] {
            StdCommand::new("git")
                .args(&args)
                .current_dir(&path)
                .output()
                .unwrap();
        }
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
    fn materialize_feature_writes_expected_files() {
        let (_d, path) = make_repo();
        let adapter = GitVcsAdapter::new(path.clone());
        let dir = materialize_feature(&adapter, &feature(), &[wp()]).unwrap();
        assert!(dir.join("meta.json").is_file());
        assert!(dir.join("status.md").is_file());
        assert!(dir.join("audit.jsonl").is_file());
        let meta: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("meta.json")).unwrap()).unwrap();
        assert_eq!(meta["slug"], "deep-feature");
    }

    #[test]
    fn materialize_work_package_writes_file() {
        let (_d, path) = make_repo();
        let adapter = GitVcsAdapter::new(path.clone());
        materialize_work_package(&adapter, "deep-feature", &wp()).unwrap();
        let wp_path = path.join("docs/agileplus/deep-feature/work-packages/99.json");
        assert!(wp_path.is_file());
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&wp_path).unwrap()).unwrap();
        assert_eq!(v["id"], 99);
    }

    #[test]
    fn materialize_and_commit_returns_oid() {
        let (_d, path) = make_repo();
        let adapter = GitVcsAdapter::new(path.clone());
        let oid = materialize_and_commit(&adapter, &feature(), &[wp()]).unwrap();
        assert_eq!(oid.len(), 40);
        // Second commit appends another audit line.
        let audit = path.join("docs/agileplus/deep-feature/audit.jsonl");
        let before = std::fs::read_to_string(&audit).unwrap().lines().count();
        materialize_feature(&adapter, &feature(), &[wp()]).unwrap();
        let after = std::fs::read_to_string(&audit).unwrap().lines().count();
        assert_eq!(after, before + 1);
    }

    #[test]
    fn commit_materialization_custom_message() {
        let (_d, path) = make_repo();
        let adapter = GitVcsAdapter::new(path.clone());
        materialize_feature(&adapter, &feature(), &[]).unwrap();
        let oid = commit_materialization(&adapter, "deep-feature", Some("test: custom")).unwrap();
        let log = StdCommand::new("git")
            .args(["log", "-1", "--pretty=%s"])
            .current_dir(&path)
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&log.stdout).trim(), "test: custom");
        assert_eq!(oid.len(), 40);
    }
}
