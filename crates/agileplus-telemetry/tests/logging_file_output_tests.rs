//! Initialisation tests for `agileplus_telemetry::logs` with file output.
//!
//! `init_logging` installs a process-global subscriber, so this file contains a
//! single test: a second `try_init` in the same process is guaranteed to fail and
//! would make any sibling test's outcome depend on execution order.

use std::path::{Path, PathBuf};

use agileplus_telemetry::logs::{LogConfig, LogError, LogOutput, init_logging};

/// Collect the files the rolling appender created in `dir`.
fn appender_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("read appender dir")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    files
}

#[test]
fn file_output_writes_structured_json_and_rejects_a_second_init() {
    let dir = tempfile::tempdir().expect("temp log dir");
    let config = LogConfig {
        level: "info".to_string(),
        output: LogOutput::File(dir.path().join("agileplus.log")),
        include_spans: true,
        include_target: true,
    };

    let guard = init_logging(&config).expect("first init must succeed");
    tracing::info!(answer = 42, "file output coverage marker");
    // The non-blocking worker is flushed when the guard drops.
    drop(guard);

    let files = appender_files(dir.path());
    assert_eq!(
        files.len(),
        1,
        "the rolling appender creates exactly one dated file: {files:?}"
    );
    let file_name = files[0].file_name().expect("file name").to_string_lossy();
    assert!(
        file_name.starts_with("agileplus.log"),
        "unexpected appender file name: {file_name}"
    );

    let content = std::fs::read_to_string(&files[0]).expect("read log file");
    let lines: Vec<&str> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(lines.len(), 1, "exactly one event was logged: {content:?}");

    let record: serde_yaml::Value = serde_yaml::from_str(lines[0]).expect("log line must be JSON");
    assert_eq!(record["level"].as_str(), Some("INFO"));
    assert_eq!(
        record["fields"]["message"].as_str(),
        Some("file output coverage marker")
    );
    assert_eq!(record["fields"]["answer"].as_i64(), Some(42));
    assert!(
        record["target"]
            .as_str()
            .is_some_and(|target| target.contains("logging_file_output_tests")),
        "include_target must emit the call site target: {record:?}"
    );
    assert!(
        record["timestamp"].as_str().is_some(),
        "structured logs must carry a timestamp: {record:?}"
    );

    // `try_init` only ever succeeds once per process.
    let second = init_logging(&LogConfig::default());
    match second {
        Err(LogError::Init(message)) => {
            assert!(!message.is_empty(), "init failure must carry a reason");
        }
        other => panic!("second init must be rejected, got {other:?}"),
    }
}
