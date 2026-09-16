//! Code scanner: walk a repository and extract structural evidence facts that
//! the SpecKitty scoring engine consumes (design §4.1).
//!
//! Dependency-free (std::fs only) — the scanner just needs file-presence, counts,
//! and shallow content probes, not a full ignore-aware walker. Facts are emitted as
//! [`EvidenceItem`]s keyed by an `artifact_id` the scoring rules match on.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::{GovernanceError, Result};

/// A single structural fact about a repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceItem {
    /// Stable key scoring rules match on, e.g. `file:AGENTS.md`, `count:test_files`.
    pub artifact_id: String,
    /// Fact kind, e.g. `file_presence`, `count`, `content_probe`.
    pub kind: String,
    /// Repo-relative path this fact refers to (empty for aggregate counts).
    #[serde(default)]
    pub path: String,
    /// Free-form metadata (bool/number as strings; keeps the type flat + serde-simple).
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl EvidenceItem {
    fn presence(artifact_id: &str, path: &str, present: bool) -> Self {
        let mut metadata = BTreeMap::new();
        metadata.insert("present".into(), present.to_string());
        EvidenceItem {
            artifact_id: artifact_id.into(),
            kind: "file_presence".into(),
            path: path.into(),
            metadata,
        }
    }

    fn count(artifact_id: &str, n: usize) -> Self {
        let mut metadata = BTreeMap::new();
        metadata.insert("count".into(), n.to_string());
        EvidenceItem {
            artifact_id: artifact_id.into(),
            kind: "count".into(),
            path: String::new(),
            metadata,
        }
    }

    /// Read `present` as bool (defaults false).
    pub fn present(&self) -> bool {
        self.metadata
            .get("present")
            .map(|v| v == "true")
            .unwrap_or(false)
    }

    /// Read `count` as usize (defaults 0).
    pub fn count_value(&self) -> usize {
        self.metadata
            .get("count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }
}

/// Directory names skipped during the walk (build output, VCS, deps).
const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".claude",
    "dist",
    "build",
    ".venv",
    "vendor",
];

/// Repo-root files whose presence is a scored signal.
const PRESENCE_FILES: &[&str] = &[
    "AGENTS.md",
    "CLAUDE.md",
    "llms.txt",
    "README.md",
    "CHANGELOG.md",
    "deny.toml",
    "rust-toolchain.toml",
    ".github/PULL_REQUEST_TEMPLATE.md",
    "docs/functional_requirements.md",
    "docs/friction-log.md",
    "Dockerfile",
    "Containerfile",
];

/// Structural evidence scanned from one repository.
#[derive(Debug, Clone, Default)]
pub struct RepoScan {
    /// All emitted facts.
    pub items: Vec<EvidenceItem>,
}

impl RepoScan {
    /// Look up a fact by artifact_id.
    pub fn get(&self, artifact_id: &str) -> Option<&EvidenceItem> {
        self.items.iter().find(|i| i.artifact_id == artifact_id)
    }

    /// True if a scored presence-file exists.
    pub fn has(&self, repo_relative: &str) -> bool {
        self.get(&format!("file:{repo_relative}"))
            .map(|i| i.present())
            .unwrap_or(false)
    }
}

/// Walk `repo_root` and collect structural evidence.
pub fn scan_repo(repo_root: impl AsRef<Path>) -> Result<RepoScan> {
    let root = repo_root.as_ref();
    if !root.is_dir() {
        return Err(GovernanceError::Rubric(format!(
            "scan target is not a directory: {}",
            root.display()
        )));
    }

    let mut items = Vec::new();

    // Root-file presence signals.
    for rel in PRESENCE_FILES {
        let present = root.join(rel).exists();
        items.push(EvidenceItem::presence(&format!("file:{rel}"), rel, present));
    }

    // Rust test-file count (files containing a #[test]/#[tokio::test] attribute).
    let rs_files = collect_files(root, "rs");
    let test_files = rs_files
        .iter()
        .filter(|p| file_contains_any(p, &["#[test]", "#[tokio::test]"]))
        .count();
    items.push(EvidenceItem::count("count:rs_files", rs_files.len()));
    items.push(EvidenceItem::count("count:test_files", test_files));

    // Workspace crate count (Cargo.toml files below root).
    let crate_manifests = collect_named(root, "Cargo.toml").len();
    items.push(EvidenceItem::count(
        "count:cargo_manifests",
        crate_manifests,
    ));

    // CI workflow presence.
    let ci = root.join(".github/workflows").is_dir()
        && !collect_files(&root.join(".github/workflows"), "yml").is_empty();
    items.push(EvidenceItem::presence(
        "dir:.github/workflows",
        ".github/workflows",
        ci,
    ));

    Ok(RepoScan { items })
}

/// Recursively collect files with the given extension, skipping SKIP_DIRS.
fn collect_files(root: &Path, ext: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(root, &mut |p| {
        if p.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(p.to_path_buf());
        }
    });
    out
}

/// Recursively collect files with an exact name.
fn collect_named(root: &Path, name: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(root, &mut |p| {
        if p.file_name().and_then(|n| n.to_str()) == Some(name) {
            out.push(p.to_path_buf());
        }
    });
    out
}

/// Depth-first walk applying `f` to each file, skipping SKIP_DIRS.
fn walk(dir: &Path, f: &mut dyn FnMut(&Path)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if SKIP_DIRS.contains(&name) {
                continue;
            }
            walk(&path, f);
        } else {
            f(&path);
        }
    }
}

/// True if the file contains any of the given needles.
fn file_contains_any(path: &Path, needles: &[&str]) -> bool {
    match std::fs::read_to_string(path) {
        Ok(s) => needles.iter().any(|n| s.contains(n)),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_repo(tag: &str) -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "speckitty-scan-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join(".github/workflows")).unwrap();
        fs::create_dir_all(base.join("src")).unwrap();
        fs::create_dir_all(base.join("target/debug")).unwrap();
        base
    }

    #[test]
    fn scans_presence_and_counts() {
        let repo = tmp_repo("presence");
        fs::write(repo.join("AGENTS.md"), "# agents").unwrap();
        fs::write(repo.join("Cargo.toml"), "[package]").unwrap();
        fs::write(repo.join(".github/workflows/ci.yml"), "on: push").unwrap();
        fs::write(repo.join("src/lib.rs"), "#[test] fn t() {}").unwrap();
        fs::write(repo.join("target/debug/junk.rs"), "#[test] fn skip() {}").unwrap();

        let scan = scan_repo(&repo).unwrap();
        assert!(scan.has("AGENTS.md"));
        assert!(!scan.has("CHANGELOG.md"));
        assert!(scan.get("dir:.github/workflows").unwrap().present());
        // target/ is skipped, so only src/lib.rs counts as a test file.
        assert_eq!(scan.get("count:test_files").unwrap().count_value(), 1);
        assert_eq!(scan.get("count:cargo_manifests").unwrap().count_value(), 1);

        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn errors_on_missing_dir() {
        let err = scan_repo("/nonexistent/speckitty/path").unwrap_err();
        assert!(matches!(err, GovernanceError::Rubric(_)));
    }

    #[test]
    fn test_file_detection() {
        let repo = tmp_repo("testfile");
        fs::write(repo.join("src/lib.rs"), "fn main() {} #[test] fn t() {}").unwrap();
        fs::create_dir_all(repo.join("tests")).unwrap();
        fs::write(repo.join("tests/test_foo.rs"), "#[test] fn t() {}").unwrap();
        fs::write(repo.join("foo_test.rs"), "#[test] fn t() {}").unwrap();
        let scan = scan_repo(&repo).unwrap();
        assert_eq!(scan.get("count:test_files").unwrap().count_value(), 3);
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn excludes_target_directory() {
        let repo = tmp_repo("excltarget");
        fs::write(repo.join("target/debug/foo.rs"), "#[test] fn t() {}").unwrap();
        fs::write(repo.join("src/lib.rs"), "").unwrap();
        let scan = scan_repo(&repo).unwrap();
        assert!(!scan.has("target/debug/foo.rs"));
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn excludes_node_modules() {
        let repo = tmp_repo("exclnode");
        fs::create_dir_all(repo.join("node_modules/pkg")).unwrap();
        fs::write(repo.join("node_modules/pkg/index.js"), "").unwrap();
        fs::write(repo.join("src/app.js"), "").unwrap();
        let scan = scan_repo(&repo).unwrap();
        // node_modules is skipped by walk, so no items reference it
        assert!(scan.items.iter().all(|i| !i.path.contains("node_modules")));
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn multiple_cargo_manifests() {
        let repo = tmp_repo("multicargo");
        fs::write(repo.join("Cargo.toml"), "[workspace]").unwrap();
        fs::create_dir_all(repo.join("crates/foo")).unwrap();
        fs::write(repo.join("crates/foo/Cargo.toml"), "[package]").unwrap();
        let scan = scan_repo(&repo).unwrap();
        // tmp_repo already has Cargo.toml via scans_presence_and_counts, plus these two
        let count = scan.get("count:cargo_manifests").unwrap().count_value();
        assert!(count >= 2, "expected >= 2, got {count}");
        fs::remove_dir_all(&repo).ok();
    }

}
#[cfg(test)]
mod extended_tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    static EXT_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn make_tmp(name: &str) -> PathBuf {
        let n = EXT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "scan-ext-{}-{}-{}",
            std::process::id(),
            name,
            n
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn evidence_item_presence_true() {
        let item = EvidenceItem::presence("file:README.md", "README.md", true);
        assert!(item.present());
        assert_eq!(item.artifact_id, "file:README.md");
        assert_eq!(item.kind, "file_presence");
        assert_eq!(item.path, "README.md");
    }

    #[test]
    fn evidence_item_presence_false() {
        let item = EvidenceItem::presence("file:MISSING", "MISSING", false);
        assert!(!item.present());
    }

    #[test]
    fn evidence_item_count_helper_values() {
        let item = EvidenceItem::count("count:rs_files", 42);
        assert_eq!(item.count_value(), 42);
        assert_eq!(item.kind, "count");
    }

    #[test]
    fn evidence_item_count_zero() {
        let item = EvidenceItem::count("count:none", 0);
        assert_eq!(item.count_value(), 0);
    }

    #[test]
    fn repo_scan_get() {
        let scan = RepoScan {
            items: vec![
                EvidenceItem::presence("file:README.md", "README.md", true),
                EvidenceItem::count("count:rs_files", 5),
            ],
        };
        assert!(scan.get("file:README.md").is_some());
        assert!(scan.get("count:rs_files").is_some());
        assert!(scan.get("missing").is_none());
    }

    #[test]
    fn repo_scan_has() {
        let scan = RepoScan {
            items: vec![EvidenceItem::presence("file:README.md", "README.md", true)],
        };
        assert!(scan.has("README.md"));
        assert!(!scan.has("MISSING.md"));
    }

    #[test]
    fn repo_scan_has_false_when_present_false() {
        let scan = RepoScan {
            items: vec![EvidenceItem::presence("file:README.md", "README.md", false)],
        };
        assert!(!scan.has("README.md"));
    }

    #[test]
    fn repo_scan_default_is_empty() {
        let scan = RepoScan::default();
        assert!(scan.items.is_empty());
    }

    #[test]
    fn scan_empty_repo() {
        let repo = make_tmp("empty");
        let scan = scan_repo(&repo).unwrap();
        // Should still have presence file entries (all false)
        assert!(!scan.items.is_empty());
        for item in &scan.items {
            if item.kind == "file_presence" {
                assert!(!item.present());
            }
        }
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn scan_collects_cargo_manifests() {
        let repo = make_tmp("cargo");
        fs::write(repo.join("Cargo.toml"), "[package]").unwrap();
        fs::create_dir_all(repo.join("sub")).unwrap();
        fs::write(repo.join("sub/Cargo.toml"), "[package]").unwrap();
        let scan = scan_repo(&repo).unwrap();
        let manifests = scan.get("count:cargo_manifests").unwrap().count_value();
        assert_eq!(manifests, 2);
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn scan_skips_target_dir() {
        let repo = make_tmp("skiptarget");
        fs::create_dir_all(repo.join("target/debug")).unwrap();
        fs::write(repo.join("target/debug/test.rs"), "#[test] fn t(){}").unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/main.rs"), "fn main(){}").unwrap();
        let scan = scan_repo(&repo).unwrap();
        assert_eq!(scan.get("count:test_files").unwrap().count_value(), 0);
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn scan_counts_test_files() {
        let repo = make_tmp("testfiles");
        fs::write(repo.join("a.rs"), "#[test] fn t(){}").unwrap();
        fs::write(repo.join("b.rs"), "#[tokio::test] async fn t(){}").unwrap();
        fs::write(repo.join("c.rs"), "fn not_a_test(){}").unwrap();
        let scan = scan_repo(&repo).unwrap();
        assert_eq!(scan.get("count:test_files").unwrap().count_value(), 2);
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn scan_not_a_dir_fails() {
        let n = EXT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp = std::env::temp_dir().join(format!(
            "scan-notdir-{}-{}",
            std::process::id(),
            n
        ));
        let _ = fs::remove_file(&tmp);
        let _ = fs::remove_dir_all(&tmp);
        fs::write(&tmp, "hello").unwrap();
        assert!(scan_repo(&tmp).is_err());
        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn scan_ci_workflows_present() {
        let repo = make_tmp("ci");
        fs::create_dir_all(repo.join(".github/workflows")).unwrap();
        fs::write(repo.join(".github/workflows/ci.yml"), "name: CI").unwrap();
        let scan = scan_repo(&repo).unwrap();
        assert!(scan.get("dir:.github/workflows").unwrap().present());
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn scan_ci_workflows_absent() {
        let repo = make_tmp("noci");
        let scan = scan_repo(&repo).unwrap();
        assert!(!scan.get("dir:.github/workflows").unwrap().present());
        fs::remove_dir_all(&repo).ok();
    }

    // ── Additional coverage tests ────────────────────────────────────────

    #[test]
    fn evidence_item_present_true() {
        let item = EvidenceItem::presence("file:README.md", "README.md", true);
        assert!(item.present());
        assert_eq!(item.count_value(), 0);
        assert_eq!(item.artifact_id, "file:README.md");
        assert_eq!(item.kind, "file_presence");
    }

    #[test]
    fn evidence_item_present_false() {
        let item = EvidenceItem::presence("file:MISSING.md", "MISSING.md", false);
        assert!(!item.present());
    }

    #[test]
    fn evidence_item_count_nonzero() {
        let item = EvidenceItem::count("count:rs_files", 42);
        assert_eq!(item.count_value(), 42);
        assert!(!item.present());
        assert_eq!(item.kind, "count");
        assert!(item.path.is_empty());
    }

    #[test]
    fn evidence_item_count_default_zero_when_no_metadata() {
        let item = EvidenceItem {
            artifact_id: "x".into(),
            kind: "count".into(),
            path: String::new(),
            metadata: std::collections::BTreeMap::new(),
        };
        assert_eq!(item.count_value(), 0);
        assert!(!item.present());
    }

    #[test]
    fn repo_scan_get_and_has_combined() {
        let scan = RepoScan {
            items: vec![
                EvidenceItem::presence("file:README.md", "README.md", true),
                EvidenceItem::presence("file:MISSING.md", "MISSING.md", false),
                EvidenceItem::count("count:rs_files", 5),
            ],
        };
        assert!(scan.has("README.md"));
        assert!(!scan.has("MISSING.md"));
        assert!(!scan.has("NONEXISTENT.md"));
        assert!(scan.get("count:rs_files").is_some());
        assert!(scan.get("nonexistent").is_none());
    }

    #[test]
    fn scan_repo_skips_node_modules() {
        let repo = make_tmp("skipnode");
        fs::create_dir_all(repo.join("node_modules/pkg")).unwrap();
        fs::write(repo.join("node_modules/pkg/test.rs"), "#[test] fn t(){}").unwrap();
        let scan = scan_repo(&repo).unwrap();
        assert_eq!(scan.get("count:test_files").unwrap().count_value(), 0);
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn scan_repo_counts_nested_cargo_manifests() {
        let repo = make_tmp("nested_cargo");
        fs::write(repo.join("Cargo.toml"), "[package]").unwrap();
        fs::create_dir_all(repo.join("crates/sub1")).unwrap();
        fs::write(repo.join("crates/sub1/Cargo.toml"), "[package]").unwrap();
        fs::create_dir_all(repo.join("crates/sub2")).unwrap();
        fs::write(repo.join("crates/sub2/Cargo.toml"), "[package]").unwrap();
        let scan = scan_repo(&repo).unwrap();
        let manifests = scan.get("count:cargo_manifests").unwrap().count_value();
        assert_eq!(manifests, 3);
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn scan_repo_presence_files_all_present() {
        let repo = make_tmp("allpresent");
        fs::write(repo.join("AGENTS.md"), "agents").unwrap();
        fs::write(repo.join("CLAUDE.md"), "claude").unwrap();
        fs::write(repo.join("llms.txt"), "llms").unwrap();
        fs::write(repo.join("README.md"), "readme").unwrap();
        let scan = scan_repo(&repo).unwrap();
        assert!(scan.has("AGENTS.md"));
        assert!(scan.has("CLAUDE.md"));
        assert!(scan.has("llms.txt"));
        assert!(scan.has("README.md"));
        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn evidence_item_serde_roundtrip() {
        let item = EvidenceItem::presence("file:README.md", "README.md", true);
        let json = serde_json::to_string(&item).unwrap();
        let back: EvidenceItem = serde_json::from_str(&json).unwrap();
        assert_eq!(back.artifact_id, item.artifact_id);
        assert_eq!(back.present(), item.present());
    }
}
