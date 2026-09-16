//! Integration tests for code_scanner: scan_repo, EvidenceItem, RepoScan.
//! Complements the inline unit tests in src/code_scanner.rs.

use agileplus_governance::*;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp_repo(tag: &str) -> std::path::PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let base = std::env::temp_dir().join(format!(
        "gov-scan-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

// ── EvidenceItem helpers (construct via struct fields since ctors are private) ──

fn make_presence(artifact_id: &str, path: &str, present: bool) -> EvidenceItem {
    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert("present".into(), present.to_string());
    EvidenceItem {
        artifact_id: artifact_id.into(),
        kind: "file_presence".into(),
        path: path.into(),
        metadata,
    }
}

fn make_count(artifact_id: &str, n: usize) -> EvidenceItem {
    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert("count".into(), n.to_string());
    EvidenceItem {
        artifact_id: artifact_id.into(),
        kind: "count".into(),
        path: String::new(),
        metadata,
    }
}

// ── EvidenceItem ─────────────────────────────────────────────────────

#[test]
fn evidence_item_presence_true_metadata() {
    let item = make_presence("file:README.md", "README.md", true);
    assert!(item.present());
    assert_eq!(item.artifact_id, "file:README.md");
    assert_eq!(item.kind, "file_presence");
    assert_eq!(item.path, "README.md");
    assert_eq!(item.metadata.get("present").unwrap(), "true");
}

#[test]
fn evidence_item_presence_false_metadata() {
    let item = make_presence("file:MISSING.md", "MISSING.md", false);
    assert!(!item.present());
    assert_eq!(item.metadata.get("present").unwrap(), "false");
}

#[test]
fn evidence_item_count_metadata() {
    let item = make_count("count:rs_files", 42);
    assert_eq!(item.count_value(), 42);
    assert_eq!(item.kind, "count");
    assert!(item.path.is_empty());
    assert_eq!(item.metadata.get("count").unwrap(), "42");
}

#[test]
fn evidence_item_count_zero() {
    let item = make_count("count:none", 0);
    assert_eq!(item.count_value(), 0);
}

#[test]
fn evidence_item_present_defaults_false_when_missing() {
    let item = EvidenceItem {
        artifact_id: "test".into(),
        kind: "file_presence".into(),
        path: "".into(),
        metadata: std::collections::BTreeMap::new(),
    };
    assert!(!item.present());
}

#[test]
fn evidence_item_count_value_defaults_zero_when_missing() {
    let item = EvidenceItem {
        artifact_id: "test".into(),
        kind: "count".into(),
        path: "".into(),
        metadata: std::collections::BTreeMap::new(),
    };
    assert_eq!(item.count_value(), 0);
}

// ── EvidenceItem serde ───────────────────────────────────────────────

#[test]
fn evidence_item_serde_roundtrip() {
    let item = make_presence("file:AGENTS.md", "AGENTS.md", true);
    let json = serde_json::to_string(&item).unwrap();
    let back: EvidenceItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.artifact_id, "file:AGENTS.md");
    assert!(back.present());
}

#[test]
fn evidence_item_count_serde_roundtrip() {
    let item = make_count("count:test_files", 10);
    let json = serde_json::to_string(&item).unwrap();
    let back: EvidenceItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.count_value(), 10);
}

// ── RepoScan ─────────────────────────────────────────────────────────

#[test]
fn repo_scan_default_is_empty() {
    let scan = RepoScan::default();
    assert!(scan.items.is_empty());
}

#[test]
fn repo_scan_get_returns_matching_item() {
    let scan = RepoScan {
        items: vec![
            make_presence("file:README.md", "README.md", true),
            make_count("count:rs_files", 5),
        ],
    };
    assert!(scan.get("file:README.md").is_some());
    assert!(scan.get("count:rs_files").is_some());
    assert!(scan.get("nonexistent").is_none());
}

#[test]
fn repo_scan_has_checks_file_presence() {
    let scan = RepoScan {
        items: vec![make_presence("file:AGENTS.md", "AGENTS.md", true)],
    };
    assert!(scan.has("AGENTS.md"));
}

#[test]
fn repo_scan_has_returns_false_for_absent() {
    let scan = RepoScan {
        items: vec![make_presence("file:AGENTS.md", "AGENTS.md", false)],
    };
    assert!(!scan.has("AGENTS.md"));
}

#[test]
fn repo_scan_has_returns_false_for_nonexistent() {
    let scan = RepoScan::default();
    assert!(!scan.has("anything"));
}

// ── scan_repo integration ────────────────────────────────────────────

#[test]
fn scan_repo_on_real_crate() {
    let crate_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scan = scan_repo(&crate_root).unwrap();
    // This crate should have README.md
    assert!(scan.has("README.md"));
    // Should have at least some Rust files
    let rs_count = scan
        .get("count:rs_files")
        .map(|i| i.count_value())
        .unwrap_or(0);
    assert!(rs_count > 0);
}

#[test]
fn scan_repo_nonexistent_directory() {
    let result = scan_repo("/nonexistent/governance-test-path");
    assert!(result.is_err());
}

#[test]
fn scan_repo_with_test_files() {
    let repo = tmp_repo("testfiles");
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(repo.join("src/lib.rs"), "#[test] fn t() {}").unwrap();
    fs::write(
        repo.join("src/other.rs"),
        "#[tokio::test] async fn at() {}",
    )
    .unwrap();
    fs::write(repo.join("src/no_test.rs"), "fn helper() {}").unwrap();

    let scan = scan_repo(&repo).unwrap();
    let test_count = scan.get("count:test_files").unwrap().count_value();
    assert_eq!(test_count, 2);

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn scan_repo_skips_target_directory() {
    let repo = tmp_repo("skiptarget");
    fs::create_dir_all(repo.join("target/debug")).unwrap();
    fs::write(repo.join("target/debug/test.rs"), "#[test] fn t() {}").unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(repo.join("src/lib.rs"), "").unwrap();

    let scan = scan_repo(&repo).unwrap();
    // target/ files should not count
    let test_count = scan.get("count:test_files").unwrap().count_value();
    assert_eq!(test_count, 0);

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn scan_repo_skips_node_modules() {
    let repo = tmp_repo("skipnode");
    fs::create_dir_all(repo.join("node_modules/pkg")).unwrap();
    fs::write(repo.join("node_modules/pkg/index.js"), "").unwrap();

    let scan = scan_repo(&repo).unwrap();
    // No items should reference node_modules
    assert!(scan.items.iter().all(|i| !i.path.contains("node_modules")));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn scan_repo_counts_cargo_manifests() {
    let repo = tmp_repo("cargo");
    fs::write(repo.join("Cargo.toml"), "[workspace]").unwrap();
    fs::create_dir_all(repo.join("crates/a")).unwrap();
    fs::write(repo.join("crates/a/Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(repo.join("crates/b")).unwrap();
    fs::write(repo.join("crates/b/Cargo.toml"), "[package]").unwrap();

    let scan = scan_repo(&repo).unwrap();
    let count = scan.get("count:cargo_manifests").unwrap().count_value();
    assert!(count >= 3, "expected >= 3, got {count}");

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn scan_repo_detects_ci_workflows() {
    let repo = tmp_repo("ci");
    fs::create_dir_all(repo.join(".github/workflows")).unwrap();
    fs::write(repo.join(".github/workflows/ci.yml"), "on: push").unwrap();

    let scan = scan_repo(&repo).unwrap();
    let ci = scan.get("dir:.github/workflows").unwrap().present();
    assert!(ci);

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn scan_repo_no_ci_workflows() {
    let repo = tmp_repo("noci");
    let scan = scan_repo(&repo).unwrap();
    let ci = scan.get("dir:.github/workflows").unwrap().present();
    assert!(!ci);

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn scan_repo_presence_file_detection() {
    let repo = tmp_repo("presence");
    fs::write(repo.join("AGENTS.md"), "# agents").unwrap();
    fs::write(repo.join("README.md"), "# readme").unwrap();
    fs::write(repo.join("CHANGELOG.md"), "# changelog").unwrap();

    let scan = scan_repo(&repo).unwrap();
    assert!(scan.has("AGENTS.md"));
    assert!(scan.has("README.md"));
    assert!(scan.has("CHANGELOG.md"));
    assert!(!scan.has("NONEXISTENT.md"));

    let _ = fs::remove_dir_all(&repo);
}
