// SPDX-License-Identifier: MIT OR Apache-2.0
//! Evidence handlers driven by real bundles on disk.
//!
//! `evidence_content`, `evidence_preview`, `feature_evidence_list`,
//! `feature_evidence_json`, and `feature_evidence_generate` all resolve paths
//! relative to the process working directory. Every test here therefore chdirs
//! into a private sandbox while holding a file-wide mutex, so no test can
//! observe another test's working directory and nothing is written into the
//! repository.
//!
//! The sibling suites only exercise the "artifact is missing" branch of these
//! handlers; the tests here cover the branches that actually read, escape, and
//! serve a bundle.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;

use agileplus_dashboard::app_state::{DashboardStore, SharedState, default_health};
use agileplus_dashboard::routes::{
    evidence_content, evidence_preview, feature_evidence_generate, feature_evidence_json,
    feature_evidence_list,
};
use axum::extract::{Path as RoutePath, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tokio::sync::RwLock;

// ── Sandbox ──────────────────────────────────────────────────────────────────

/// Private sandbox root for this test binary; each test works in a fresh
/// subdirectory so one test's fixtures can never satisfy another test's
/// assertions.
fn sandbox_root() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!(
            "agileplus-dashboard-evidence-disk-{}",
            std::process::id()
        ));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear previous sandbox root");
        }
        std::fs::create_dir_all(&dir).expect("create sandbox root");
        dir
    })
}

/// Serializes every test in this file and owns the process working directory.
///
/// The working directory is process-global, so any test that runs a handler
/// resolving `.agileplus/...` relatively must hold this guard for its whole
/// body.
struct CwdGuard {
    previous: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.previous);
    }
}

/// Enter a fresh, empty working directory for one test.
fn lock_cwd(name: &str) -> CwdGuard {
    static LOCK: Mutex<()> = Mutex::new(());
    let guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::current_dir().expect("read current dir");

    let work = sandbox_root().join(name);
    if work.exists() {
        std::fs::remove_dir_all(&work).expect("clear test directory");
    }
    std::fs::create_dir_all(&work).expect("create test directory");
    std::env::set_current_dir(&work).expect("enter test directory");

    CwdGuard {
        previous,
        _lock: guard,
    }
}

// ── Harness ──────────────────────────────────────────────────────────────────

fn state() -> SharedState {
    Arc::new(RwLock::new(DashboardStore {
        health: default_health(),
        ..Default::default()
    }))
}

async fn body_text(response: Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    String::from_utf8(bytes.to_vec()).expect("response body is utf-8")
}

/// The working directory currently owned by `CwdGuard`.
fn workdir() -> PathBuf {
    std::env::current_dir().expect("read current dir")
}

/// Write a directory under the sandbox-relative evidence root.
fn evidence_dir(feature_id: &str) -> PathBuf {
    let dir = workdir()
        .join(".agileplus")
        .join("evidence")
        .join(feature_id);
    std::fs::create_dir_all(&dir).expect("create evidence dir");
    dir
}

/// Write the bundle fixture the disk loader understands.
fn write_bundle(feature_id: &str, passed: bool) -> PathBuf {
    let path = evidence_dir(feature_id).join("bundle.json");
    let bundle = serde_json::json!({
        "timestamp": "2026-09-18T04:00:00Z",
        "test_results": {
            "passed": passed,
            "passed_count": 12,
            "failed_count": 0,
            "summary": "12 passed",
            "output_snippet": "running 12 tests ... ok"
        },
        "git_log": [{
            "short_hash": "abc1234",
            "subject": "feat(dashboard): serve disk bundles",
            "date": "2026-09-18",
            "author": "jcode",
            "url": "https://example.invalid/commit/abc1234"
        }],
        "prs": [{
            "number": 4242,
            "title": "Serve disk bundles",
            "url": "https://example.invalid/pull/4242",
            "state": "MERGED",
            "headRefName": "feat/disk-bundles",
            "createdAt": "2026-09-18T03:00:00Z"
        }],
        "ci_links": [{
            "id": 9001,
            "title": "dashboard ci",
            "status": "completed",
            "conclusion": "success",
            "url": "https://example.invalid/runs/9001",
            "created_at": "2026-09-18T03:30:00Z"
        }]
    });
    std::fs::write(&path, serde_json::to_string_pretty(&bundle).unwrap()).expect("write bundle");
    path
}

/// Poll for a file from the async context.
///
/// The generator handler finishes work on a spawned task, so this must yield
/// to the runtime instead of blocking the (current-thread) test runtime.
async fn wait_for_file(path: &Path, timeout: Duration) {
    let deadline = tokio::time::Instant::now() + timeout;
    while tokio::time::Instant::now() < deadline {
        if path.exists() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("timed out waiting for {}", path.display());
}

// ── evidence_content ─────────────────────────────────────────────────────────

#[tokio::test]
async fn evidence_content_serves_escaped_artifact_bytes() {
    let _cwd = lock_cwd("content-escaped");
    let dir = evidence_dir("77");
    std::fs::write(dir.join("report.txt"), "<b>bold</b> & \"quoted\"").expect("write artifact");

    let response = evidence_content(
        State(state()),
        RoutePath((77_i64, "report.txt".to_string())),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        body.contains("&lt;b&gt;bold&lt;/b&gt; &amp; &quot;quoted&quot;"),
        "artifact bytes must be HTML-escaped, got: {body}"
    );
    assert!(
        body.contains("<pre class='text-xs font-mono"),
        "artifact must be wrapped in the readonly pre block, got: {body}"
    );
}

#[tokio::test]
async fn evidence_content_rejects_absolute_and_nul_artifact_ids() {
    let _cwd = lock_cwd("content-forbidden");

    for artifact_id in ["/etc/passwd", "nul\0byte"] {
        let response =
            evidence_content(State(state()), RoutePath((77_i64, artifact_id.to_string()))).await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(
            body.contains("# Forbidden"),
            "artifact id {artifact_id:?} must be refused before any file read, got: {body}"
        );
    }
}

#[tokio::test]
async fn evidence_content_reports_missing_artifact_with_expected_path() {
    let _cwd = lock_cwd("content-missing");

    let response = evidence_content(
        State(state()),
        RoutePath((4242_i64, "absent.md".to_string())),
    )
    .await;

    let body = body_text(response).await;
    assert!(body.contains("# Evidence Bundle 4242"));
    assert!(body.contains("Artifact ID: absent.md"));
}

// ── evidence_preview ─────────────────────────────────────────────────────────

#[tokio::test]
async fn evidence_preview_wraps_existing_artifact_in_escaped_block() {
    let _cwd = lock_cwd("preview");
    let dir = evidence_dir("77");
    std::fs::write(dir.join("preview.md"), "<script>alert(1)</script>").expect("write artifact");

    let response = evidence_preview(
        State(state()),
        RoutePath((77_i64, "preview.md".to_string())),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        body.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
        "preview must escape markup, got: {body}"
    );
    assert!(body.contains("max-h-48 overflow-y-auto"));
}

// ── feature_evidence_list / feature_evidence_json ────────────────────────────

#[tokio::test]
async fn feature_evidence_list_renders_disk_bundle_metadata() {
    let _cwd = lock_cwd("list-disk-bundle");
    write_bundle("77", true);

    let response = feature_evidence_list(State(state()), RoutePath("77".to_string())).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("bundle-77-disk"), "got: {body}");
    assert!(body.contains("FR-77"), "got: {body}");
    assert!(
        body.contains("Tests passed"),
        "verified bundle must render the passing badge: {body}"
    );
    assert!(body.contains("verified"), "got: {body}");
    assert!(body.contains("12 pass"), "got: {body}");
    assert!(
        body.contains("abc1234"),
        "commit hash must be rendered: {body}"
    );
}

#[tokio::test]
async fn feature_evidence_json_describes_disk_artifact() {
    let _cwd = lock_cwd("json-disk-bundle");
    write_bundle("77", true);

    let response = feature_evidence_json(State(state()), RoutePath("77".to_string()))
        .await
        .into_response();

    assert_eq!(response.status(), StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body_text(response).await).expect("json");

    assert_eq!(json["feature_id"], "77");
    assert_eq!(json["artifacts"].as_array().expect("artifacts").len(), 1);
    assert_eq!(json["artifacts"][0]["id"], "bundle-77-disk");
    assert_eq!(json["artifacts"][0]["type_"], "generated_bundle");
    assert_eq!(
        json["artifacts"][0]["url"],
        "/api/evidence/77/bundle-77-disk/preview"
    );
    assert_eq!(json["generated_at"], "2026-09-18T04:00:00Z");
}

#[tokio::test]
async fn feature_evidence_json_is_empty_without_a_bundle_on_disk() {
    let _cwd = lock_cwd("json-no-bundle");

    let response = feature_evidence_json(State(state()), RoutePath("5150".to_string()))
        .await
        .into_response();

    let json: serde_json::Value = serde_json::from_str(&body_text(response).await).expect("json");
    assert_eq!(json["feature_id"], "5150");
    assert!(json["artifacts"].as_array().expect("artifacts").is_empty());
    assert!(json["generated_at"].is_null());
}

// ── feature_evidence_generate ────────────────────────────────────────────────

#[tokio::test]
async fn feature_evidence_generate_runs_the_bundle_script() {
    let _cwd = lock_cwd("generate-runs-script");

    let scripts = workdir().join("scripts");
    std::fs::create_dir_all(&scripts).expect("create scripts dir");
    std::fs::write(
        scripts.join("generate-evidence.sh"),
        "#!/usr/bin/env bash\n\
         set -eu\n\
         feature_id=\"$1\"\n\
         mkdir -p \".agileplus/evidence/${feature_id}\"\n\
         printf '%s' '{\"timestamp\":\"2026-09-18T05:00:00Z\"}' > \".agileplus/evidence/${feature_id}/bundle.json\"\n\
         printf 'generated' > \"generate-${feature_id}.marker\"\n",
    )
    .expect("write generator script");

    let response = feature_evidence_generate(State(state()), RoutePath("88".to_string()))
        .await
        .into_response();

    assert_eq!(response.status(), StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body_text(response).await).expect("json");
    assert_eq!(json["status"], "started");
    assert_eq!(json["feature_id"], "88");
    assert_eq!(json["bundle_path"], ".agileplus/evidence/88/bundle.json");

    // Hold the working directory until the detached generation finishes.
    let marker = workdir().join("generate-88.marker");
    wait_for_file(&marker, Duration::from_secs(15)).await;
    assert_eq!(
        std::fs::read_to_string(&marker).expect("read marker"),
        "generated"
    );

    // The generated bundle must be readable by the list handler.
    let listed = feature_evidence_list(State(state()), RoutePath("88".to_string())).await;
    let body = body_text(listed).await;
    assert!(
        body.contains("bundle-88-disk"),
        "generated bundle must be served, got: {body}"
    );
}

#[tokio::test]
async fn feature_evidence_generate_reports_missing_script() {
    let _cwd = lock_cwd("generate-missing-script");

    // The sandbox has no scripts/ directory for this id.
    let response = feature_evidence_generate(State(state()), RoutePath("99".to_string()))
        .await
        .into_response();

    assert_eq!(response.status(), StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body_text(response).await).expect("json");
    assert_eq!(json["status"], "error");
    assert_eq!(json["feature_id"], "99");
    assert!(
        json["message"]
            .as_str()
            .expect("message")
            .contains("generate-evidence.sh")
    );
}
