//! Initialisation tests for `agileplus_telemetry::logs` with the `Both` output.
//!
//! `init_logging` installs a process-global subscriber, so this file contains a
//! single test. This branch leaks its file `WorkerGuard` on purpose
//! (`std::mem::forget`), so the file contents are not guaranteed to be flushed
//! when the returned guard drops; the test therefore asserts on the branch's
//! observable contract (successful init plus the eagerly created appender file)
//! instead of racing the background writer.

use std::path::Path;

use agileplus_telemetry::logs::{LogConfig, LogOutput, init_logging};

fn appender_files(dir: &Path) -> Vec<String> {
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .expect("read appender dir")
        .map(|entry| {
            entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    files.sort();
    files
}

#[test]
fn both_output_initialises_and_creates_the_file_appender_eagerly() {
    let dir = tempfile::tempdir().expect("temp log dir");
    let config = LogConfig {
        level: "info".to_string(),
        output: LogOutput::Both(dir.path().join("agileplus.log")),
        include_spans: true,
        include_target: true,
    };

    let guard = init_logging(&config).expect("both-output init must succeed");

    // Writing must not panic on the dual-writer path.
    tracing::info!("dual output coverage marker");
    drop(guard);

    let files = appender_files(dir.path());
    assert_eq!(
        files.len(),
        1,
        "the rolling appender opens its file at construction: {files:?}"
    );
    assert!(
        files[0].starts_with("agileplus.log"),
        "unexpected appender file name: {}",
        files[0]
    );
}
