//! Git artifact read/write operations for AgilePlus.
//!
//! Artifacts live under kitty-specs/<feature-slug>/<relative-path>.
//! write_artifact stages the file but does NOT commit.
//!
//! Traceability: WP07-T041, WP07-T042

use std::path::{Path, PathBuf};

use agileplus_domain::{error::DomainError, ports::FeatureArtifacts};

use crate::GitVcsAdapter;

/// Compute full artifact path from repo root, rejecting path traversal.
fn artifact_full_path(
    repo_root: &Path,
    feature_slug: &str,
    relative_path: &str,
) -> Result<PathBuf, DomainError> {
    let base = repo_root.join("kitty-specs").join(feature_slug);
    let full = base.join(relative_path);

    // Normalize away any `..` components by iterating path components.
    // We can't use canonicalize() because the file may not exist yet (write case).
    let mut normalized = PathBuf::new();
    for component in full.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other),
        }
    }

    if !normalized.starts_with(&base) {
        return Err(DomainError::Vcs(format!(
            "artifact path traversal detected: {relative_path}"
        )));
    }
    Ok(normalized)
}

/// Read a text artifact from the working tree.
pub(crate) fn read_artifact(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
    relative_path: &str,
) -> Result<String, DomainError> {
    let path = artifact_full_path(adapter.repo_path(), feature_slug, relative_path)?;
    if !path.exists() {
        return Err(DomainError::NotFound(format!(
            "artifact not found: kitty-specs/{feature_slug}/{relative_path}"
        )));
    }
    std::fs::read_to_string(&path).map_err(|e| DomainError::Vcs(format!("read artifact: {e}")))
}

/// Write a text artifact and stage it in the git index.
pub(crate) fn write_artifact(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
    relative_path: &str,
    content: &str,
) -> Result<(), DomainError> {
    let path = artifact_full_path(adapter.repo_path(), feature_slug, relative_path)?;

    // Create parent directories.
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| DomainError::Vcs(format!("create artifact dirs: {e}")))?;
    }

    std::fs::write(&path, content).map_err(|e| DomainError::Vcs(format!("write artifact: {e}")))?;

    // Stage the file in git index.
    let repo = adapter.open_repo()?;
    let mut index = repo.index().map_err(crate::git_err)?;

    // Path relative to repo workdir for index.
    let relative = path
        .strip_prefix(adapter.repo_path())
        .map_err(|_| DomainError::Vcs("artifact path outside repo".into()))?;

    index.add_path(relative).map_err(crate::git_err)?;
    index.write().map_err(crate::git_err)?;

    Ok(())
}

/// Check whether an artifact exists in the working tree.
pub(crate) fn artifact_exists(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
    relative_path: &str,
) -> Result<bool, DomainError> {
    let path = artifact_full_path(adapter.repo_path(), feature_slug, relative_path)?;
    Ok(path.exists())
}

/// Scan and collect all artifacts for a feature from the working tree.
pub(crate) fn scan_feature_artifacts(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
) -> Result<FeatureArtifacts, DomainError> {
    let base = adapter.repo_path().join("kitty-specs").join(feature_slug);

    let meta_json = {
        let p = base.join("meta.json");
        if p.exists() {
            Some(std::fs::read_to_string(&p).map_err(|e| DomainError::Vcs(e.to_string()))?)
        } else {
            None
        }
    };

    let audit_chain = {
        let p = base.join("audit").join("chain.jsonl");
        if p.exists() {
            Some(std::fs::read_to_string(&p).map_err(|e| DomainError::Vcs(e.to_string()))?)
        } else {
            None
        }
    };

    let evidence_paths = collect_evidence(&base.join("evidence"))?;

    Ok(FeatureArtifacts {
        spec: None,
        research: None,
        plan: None,
        other: vec![],
        meta_json,
        audit_chain,
        evidence_paths,
    })
}

/// Recursively collect file paths under the evidence directory.
fn collect_evidence(evidence_dir: &Path) -> Result<Vec<String>, DomainError> {
    if !evidence_dir.exists() {
        return Ok(vec![]);
    }
    let mut paths = Vec::new();
    collect_evidence_recursive(evidence_dir, &mut paths)?;
    Ok(paths)
}

fn collect_evidence_recursive(dir: &Path, paths: &mut Vec<String>) -> Result<(), DomainError> {
    for entry in std::fs::read_dir(dir).map_err(|e| DomainError::Vcs(e.to_string()))? {
        let entry = entry.map_err(|e| DomainError::Vcs(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_evidence_recursive(&path, paths)?;
        } else {
            paths.push(path.to_string_lossy().to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> std::path::PathBuf {
        std::path::PathBuf::from("/repo")
    }

    #[test]
    fn full_path_joins_under_kitty_specs() {
        let p = artifact_full_path(&repo_root(), "001-feature", "spec.md").unwrap();
        assert!(p.ends_with("kitty-specs/001-feature/spec.md"));
    }

    #[test]
    fn full_path_allows_nested_relative_paths() {
        let p = artifact_full_path(&repo_root(), "001-feature", "evidence/logs/run.txt").unwrap();
        assert!(p.ends_with("kitty-specs/001-feature/evidence/logs/run.txt"));
    }

    #[test]
    fn full_path_accepts_empty_relative_path() {
        let p = artifact_full_path(&repo_root(), "001-feature", "").unwrap();
        assert!(p.ends_with("kitty-specs/001-feature"));
    }

    #[test]
    fn full_path_rejects_parent_traversal() {
        let err = artifact_full_path(&repo_root(), "001-feature", "../secret.txt").unwrap_err();
        assert!(format!("{err}").contains("traversal"));
    }

    #[test]
    fn full_path_rejects_deep_traversal() {
        let err =
            artifact_full_path(&repo_root(), "001-feature", "../../etc/passwd").unwrap_err();
        assert!(format!("{err}").contains("traversal"));
    }

    #[test]
    fn full_path_rejects_traversal_before_re_entry() {
        // `../x` pops the slug dir; the result is still under kitty-specs but
        // outside the feature directory, so it must be rejected.
        let err = artifact_full_path(&repo_root(), "001-feature", "../other/spec.md").unwrap_err();
        assert!(format!("{err}").contains("traversal"));
    }

    #[test]
    fn full_path_traversal_returning_under_slug_is_allowed() {
        // `sub/../spec.md` normalises back inside the slug directory.
        let p = artifact_full_path(&repo_root(), "001-feature", "sub/../spec.md").unwrap();
        assert!(p.ends_with("kitty-specs/001-feature/spec.md"));
    }

    #[test]
    fn artifact_exists_is_false_for_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let adapter = GitVcsAdapter::new(dir.path().to_path_buf());
        assert!(!artifact_exists(&adapter, "001-feature", "spec.md").unwrap());
    }

    #[test]
    fn artifact_exists_is_true_for_present_file() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("kitty-specs/001-feature");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("spec.md"), "content").unwrap();
        let adapter = GitVcsAdapter::new(dir.path().to_path_buf());
        assert!(artifact_exists(&adapter, "001-feature", "spec.md").unwrap());
    }

    #[test]
    fn read_artifact_returns_contents() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("kitty-specs/001-feature");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("spec.md"), "hello").unwrap();
        let adapter = GitVcsAdapter::new(dir.path().to_path_buf());
        assert_eq!(read_artifact(&adapter, "001-feature", "spec.md").unwrap(), "hello");
    }

    #[test]
    fn read_artifact_missing_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let adapter = GitVcsAdapter::new(dir.path().to_path_buf());
        let err = read_artifact(&adapter, "001-feature", "nope.md").unwrap_err();
        assert!(matches!(err, DomainError::NotFound(_)));
    }

    #[test]
    fn read_artifact_rejects_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let adapter = GitVcsAdapter::new(dir.path().to_path_buf());
        assert!(read_artifact(&adapter, "001-feature", "../../secret").is_err());
    }

    #[test]
    fn scan_feature_artifacts_empty_feature_has_no_files() {
        let dir = tempfile::tempdir().unwrap();
        let adapter = GitVcsAdapter::new(dir.path().to_path_buf());
        let arts = scan_feature_artifacts(&adapter, "missing-feature").unwrap();
        assert!(arts.meta_json.is_none());
        assert!(arts.audit_chain.is_none());
        assert!(arts.evidence_paths.is_empty());
    }

    #[test]
    fn scan_feature_artifacts_reads_meta_and_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("kitty-specs/001-feature");
        std::fs::create_dir_all(base.join("evidence/nested")).unwrap();
        std::fs::write(base.join("meta.json"), "{}").unwrap();
        std::fs::write(base.join("evidence/nested/log.txt"), "x").unwrap();
        std::fs::write(base.join("evidence/run.log"), "y").unwrap();
        let adapter = GitVcsAdapter::new(dir.path().to_path_buf());
        let arts = scan_feature_artifacts(&adapter, "001-feature").unwrap();
        assert_eq!(arts.meta_json.as_deref(), Some("{}"));
        assert_eq!(arts.evidence_paths.len(), 2);
    }

    #[test]
    fn scan_feature_artifacts_reads_audit_chain() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("kitty-specs/001-feature/audit");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("chain.jsonl"), "{}\n").unwrap();
        let adapter = GitVcsAdapter::new(dir.path().to_path_buf());
        let arts = scan_feature_artifacts(&adapter, "001-feature").unwrap();
        assert_eq!(arts.audit_chain.as_deref(), Some("{}\n"));
    }
}
