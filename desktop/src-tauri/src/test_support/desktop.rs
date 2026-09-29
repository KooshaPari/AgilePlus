// SPDX-License-Identifier: MIT OR Apache-2.0
//! Behavior tests for the desktop crate's filesystem-facing commands.
//!
//! These live inside the crate rather than in `tests/` because the commands
//! under test sit in private modules. They are exercised through Tauri's own
//! IPC layer with a `MockRuntime`, which is the same path the frontend uses,
//! so assertions are about observable command results rather than about a
//! test-only reimplementation. `tauri::State` itself cannot be constructed
//! outside a runtime because its tuple field is private, so going through IPC
//! is the only way to reach these functions at all.

#![cfg(test)]

use crate::AppState;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
use tauri::test::{MockRuntime, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::{Manager, WebviewWindow, WebviewWindowBuilder};

/// Unique scratch directory per test, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "agileplus-desktop-test-{name}-{}-{n}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("creating scratch dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, rel: &str, contents: &str) -> PathBuf {
        let full = self.0.join(rel);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).expect("creating parent dir");
        }
        std::fs::write(&full, contents).expect("writing scratch file");
        full
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Build a mock app with the production command set registered.
///
/// Mirrors the handler list in `lib.rs::run`. The `run` function itself cannot
/// be called in a test because it installs a panic hook, builds a real tray
/// icon, and enters the platform event loop, none of which is available under
/// `MockRuntime`.
fn test_app(repo_path: &Path) -> tauri::App<tauri::test::MockRuntime> {
    let app = tauri::test::mock_builder()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            crate::commands::list_features,
            crate::commands::get_feature,
            crate::commands::get_feature_with_details,
            crate::commands::get_dashboard_stats,
            crate::commands::set_repo_path,
            crate::commands::get_repo_path,
            crate::commands::open_project,
            crate::adrs::list_adrs,
            crate::adrs::read_adr,
            crate::traces::list_traces,
            crate::traces::read_trace,
            crate::work_packages::list_work_packages,
            crate::work_packages::create_work_package,
            crate::work_packages::update_work_package_state,
            crate::evidence::list_evidence,
            crate::evidence::create_evidence,
            crate::cli_bridge::run_cli_read,
            crate::cli_bridge::run_cli_lifecycle,
            crate::crashes::list_crash_logs,
            crate::crashes::clear_crash_logs,
        ])
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("building mock app");

    {
        let state = app.state::<AppState>();
        state.set_repo_path(repo_path.to_string_lossy().to_string());
    }

    app
}

fn webview(app: &tauri::App<MockRuntime>) -> WebviewWindow<MockRuntime> {
    WebviewWindowBuilder::new(app, "main", Default::default())
        .build()
        .expect("building mock webview")
}

/// Invoke a command over IPC and return its JSON result.
///
/// The helper returns `Result<serde_json::Value, serde_json::Value>` so tests
/// can assert on either the success payload or the exact error message the
/// frontend would receive.
fn invoke(
    webview: &WebviewWindow<MockRuntime>,
    cmd: &str,
    args: serde_json::Map<String, serde_json::Value>,
) -> Result<serde_json::Value, serde_json::Value> {
    let response = tauri::test::get_ipc_response(
        webview,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(args.into()),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    );

    // A successful command body is the raw serialized return value, with no
    // Ok/Err envelope. Tauri only wraps failures, and that wrapper is JSON
    // like `{"Err":"message"}` because the commands return Result<_, String>.
    match response {
        Ok(InvokeResponseBody::Json(raw)) => Ok(serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("response was not JSON: {e}; body={raw}"))),
        Ok(other) => panic!("expected a JSON response body, got {other:?}"),
        Err(err) => Err(unwrap_command_error(err)),
    }
}

/// Commands return `Result<_, String>`, which Tauri serializes as `{"Err": ..}`.
/// Pull the message out so tests can assert on the text the frontend shows.
fn unwrap_command_error(err: serde_json::Value) -> serde_json::Value {
    match &err {
        serde_json::Value::Object(map) => match map.get("Err") {
            Some(serde_json::Value::String(msg)) => serde_json::Value::String(msg.clone()),
            _ => err,
        },
        _ => err,
    }
}

fn args(pairs: Vec<(&str, serde_json::Value)>) -> serde_json::Map<String, serde_json::Value> {
    pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

#[test]
fn set_and_get_repo_path_round_trip_through_state() {
    let scratch = Scratch::new("repo-path");
    let app = test_app(scratch.path());
    let wv = webview(&app);

    let echoed = invoke(&wv, "get_repo_path", args(vec![])).expect("get_repo_path should succeed");
    assert_eq!(
        echoed.as_str(),
        Some(scratch.path().to_string_lossy().as_ref())
    );

    let new_path = "/tmp/some-other-project";
    invoke(&wv, "set_repo_path", args(vec![("path", json!(new_path))]))
        .expect("set_repo_path should succeed");

    let updated = invoke(&wv, "get_repo_path", args(vec![])).expect("get_repo_path after set");
    assert_eq!(updated.as_str(), Some(new_path));
}

#[test]
fn list_adrs_returns_empty_when_docs_adr_is_absent() {
    let scratch = Scratch::new("adr-missing");
    let app = test_app(scratch.path());
    let wv = webview(&app);

    let adrs = invoke(&wv, "list_adrs", args(vec![])).expect("list_adrs should succeed");
    assert_eq!(adrs, json!([]));
}

#[test]
fn list_adrs_parses_numbered_filenames_and_status_lines() {
    let scratch = Scratch::new("adr-parse");
    scratch.write(
        "docs/adr/0002-use-rust-for-the-core.md",
        "# ADR 2\n\nStatus: Accepted\n\nSome prose.\n",
    );
    scratch.write(
        "docs/adr/0001-initial-decision.md",
        "# ADR 1\n\nStatus: Superseded\n",
    );
    // No numeric prefix: id and title both fall back to the whole stem.
    scratch.write("docs/adr/notes.md", "Status: Draft\n");
    // Wrong extension: must be ignored entirely.
    scratch.write("docs/adr/0003-ignored.txt", "Status: Accepted\n");

    let app = test_app(scratch.path());
    let wv = webview(&app);

    let adrs = invoke(&wv, "list_adrs", args(vec![])).expect("list_adrs should succeed");
    let list = adrs.as_array().expect("list_adrs should return an array");
    assert_eq!(list.len(), 3, "only the three .md files should be listed");

    // `list_adrs` sorts by the id string descending, so the unprefixed
    // "notes" outranks the zero-padded numeric ids, which sort "notes" > "0002".
    assert_eq!(list[0]["id"], json!("notes"));
    assert_eq!(list[0]["title"], json!("notes"));
    assert_eq!(list[0]["status"], json!("Draft"));

    assert_eq!(list[1]["id"], json!("0002"));
    assert_eq!(list[1]["title"], json!("use rust for the core"));
    assert_eq!(list[1]["status"], json!("Accepted"));

    assert_eq!(list[2]["id"], json!("0001"));
    assert_eq!(list[2]["title"], json!("initial decision"));
    assert_eq!(list[2]["status"], json!("Superseded"));
}

#[test]
fn read_adr_returns_contents_for_matching_id() {
    let scratch = Scratch::new("adr-read");
    scratch.write(
        "docs/adr/0042-the-answer.md",
        "# ADR 42\n\nStatus: Accepted\n",
    );

    let app = test_app(scratch.path());
    let wv = webview(&app);

    let body = invoke(&wv, "read_adr", args(vec![("id", json!("0042"))]))
        .expect("read_adr should succeed");
    assert!(body
        .as_str()
        .expect("read_adr returns a string")
        .contains("ADR 42"));
}

#[test]
fn read_adr_errors_when_id_has_no_match() {
    let scratch = Scratch::new("adr-read-missing");
    scratch.write("docs/adr/0001-present.md", "Status: Accepted\n");

    let app = test_app(scratch.path());
    let wv = webview(&app);

    let err = invoke(&wv, "read_adr", args(vec![("id", json!("9999"))]))
        .expect_err("read_adr should fail for an unknown id");
    assert_eq!(err.as_str(), Some("ADR with id '9999' not found"));
}

#[test]
fn read_adr_errors_when_directory_is_absent() {
    let scratch = Scratch::new("adr-read-nodir");
    let app = test_app(scratch.path());
    let wv = webview(&app);

    let err = invoke(&wv, "read_adr", args(vec![("id", json!("0001"))]))
        .expect_err("read_adr should fail when docs/adr is missing");
    assert!(
        err.as_str()
            .expect("error is a string")
            .contains("ADR directory not found"),
        "unexpected error: {err}"
    );
}

#[test]
fn list_traces_returns_empty_when_traces_dir_is_absent() {
    let scratch = Scratch::new("traces-missing");
    let app = test_app(scratch.path());
    let wv = webview(&app);

    let traces = invoke(&wv, "list_traces", args(vec![])).expect("list_traces should succeed");
    assert_eq!(traces, json!([]));
}

#[test]
fn list_traces_filters_extensions_and_sorts_by_name_descending() {
    let scratch = Scratch::new("traces-filter");
    scratch.write("traces/alpha.jsonl", "{\"a\":1}\n");
    scratch.write("traces/beta.md", "# beta\n");
    scratch.write("traces/gamma.jsonl", "{\"g\":1}\n");
    // Wrong extension, must be skipped.
    scratch.write("traces/delta.txt", "not a trace");
    // No extension, also skipped.
    scratch.write("traces/README", "nope");

    let app = test_app(scratch.path());
    let wv = webview(&app);

    let traces = invoke(&wv, "list_traces", args(vec![])).expect("list_traces should succeed");
    let list = traces
        .as_array()
        .expect("list_traces should return an array");
    assert_eq!(list.len(), 3, "only .jsonl and .md files are included");

    let names: Vec<&str> = list.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(
        names,
        vec!["gamma", "beta", "alpha"],
        "sorted by name descending"
    );

    assert_eq!(list[0]["file_type"], json!("jsonl"));
    assert_eq!(list[1]["file_type"], json!("md"));
    assert!(list[0]["path"].as_str().unwrap().ends_with("gamma.jsonl"));
}

#[test]
fn read_trace_prefers_jsonl_over_markdown_for_the_same_name() {
    let scratch = Scratch::new("traces-precedence");
    scratch.write("traces/session.jsonl", "jsonl body\n");
    scratch.write("traces/session.md", "markdown body\n");

    let app = test_app(scratch.path());
    let wv = webview(&app);

    let body = invoke(&wv, "read_trace", args(vec![("name", json!("session"))]))
        .expect("read_trace should succeed");
    assert_eq!(body.as_str(), Some("jsonl body\n"));
}

#[test]
fn read_trace_falls_back_to_markdown_when_jsonl_is_absent() {
    let scratch = Scratch::new("traces-md");
    scratch.write("traces/only-md.md", "markdown body\n");

    let app = test_app(scratch.path());
    let wv = webview(&app);

    let body = invoke(&wv, "read_trace", args(vec![("name", json!("only-md"))]))
        .expect("read_trace should succeed");
    assert_eq!(body.as_str(), Some("markdown body\n"));
}

#[test]
fn read_trace_errors_for_unknown_name() {
    let scratch = Scratch::new("traces-unknown");
    scratch.write("traces/present.jsonl", "{}\n");

    let app = test_app(scratch.path());
    let wv = webview(&app);

    let err = invoke(&wv, "read_trace", args(vec![("name", json!("absent"))]))
        .expect_err("read_trace should fail for an unknown name");
    assert_eq!(err.as_str(), Some("Trace 'absent' not found"));
}

#[test]
fn read_trace_errors_when_traces_dir_is_absent() {
    let scratch = Scratch::new("traces-nodir-read");
    let app = test_app(scratch.path());
    let wv = webview(&app);

    let err = invoke(&wv, "read_trace", args(vec![("name", json!("anything"))]))
        .expect_err("read_trace should fail when traces/ is missing");
    assert!(
        err.as_str()
            .expect("error is a string")
            .contains("Traces directory not found"),
        "unexpected error: {err}"
    );
}

#[test]
fn open_project_rejects_a_directory_without_an_agileplus_db() {
    let scratch = Scratch::new("open-project-missing");
    let app = test_app(scratch.path());
    let wv = webview(&app);

    let err = invoke(
        &wv,
        "open_project",
        args(vec![(
            "path",
            json!(scratch.path().to_string_lossy().as_ref()),
        )]),
    )
    .expect_err("open_project should reject a non-project directory");
    let msg = err.as_str().expect("error is a string");
    assert!(
        msg.contains("No .agileplus/agileplus.db found"),
        "unexpected error: {msg}"
    );
}

#[test]
fn db_backed_commands_report_uninitialized_database() {
    let scratch = Scratch::new("db-uninitialized");
    let app = test_app(scratch.path());
    let wv = webview(&app);

    // No connection has been stored, so every DB-backed command must say so
    // rather than panic or return a misleading empty result.
    let err = invoke(&wv, "list_features", args(vec![]))
        .expect_err("list_features should fail without a database");
    // `list_features` carries a longer, action-oriented hint than the other
    // DB-backed commands, so assert the exact text rather than a prefix.
    assert_eq!(
        err.as_str(),
        Some("Database not initialized. Open a project first.")
    );

    let err = invoke(&wv, "get_dashboard_stats", args(vec![]))
        .expect_err("get_dashboard_stats should fail without a database");
    assert_eq!(err.as_str(), Some("Database not initialized"));

    let err = invoke(
        &wv,
        "list_work_packages",
        args(vec![("featureId", json!(1))]),
    )
    .expect_err("list_work_packages should fail without a database");
    assert_eq!(err.as_str(), Some("Database not initialized"));

    let err = invoke(&wv, "list_evidence", args(vec![("featureId", json!(1))]))
        .expect_err("list_evidence should fail without a database");
    assert_eq!(err.as_str(), Some("Database not initialized"));
}
