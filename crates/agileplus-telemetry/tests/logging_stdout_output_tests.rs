//! Initialisation tests for `agileplus_telemetry::logs` with stdout output.
//!
//! `init_logging` installs a process-global subscriber, so this file contains a
//! single test: a second `try_init` in the same process is guaranteed to fail and
//! would make any sibling test's outcome depend on execution order. The test
//! harness captures the stdout writer, so this test asserts on the returned
//! guard and the init contract rather than on captured console text.

use agileplus_telemetry::logs::{LogConfig, LogError, LogOutput, flush, init_logging};

#[test]
fn stdout_output_initialises_returns_a_guard_and_rejects_a_second_init() {
    let config = LogConfig {
        level: "info".to_string(),
        output: LogOutput::Stdout,
        include_spans: true,
        include_target: true,
    };

    let guard = init_logging(&config).expect("stdout init must succeed");

    // Level filtering is wired to the configured level; an event above the
    // threshold must be accepted without panicking.
    tracing::warn!("stdout output coverage marker");
    flush();
    drop(guard);

    // Even an explicit `init_logging` call cannot replace the installed
    // subscriber: the crate documents "call exactly once".
    match init_logging(&LogConfig {
        level: "debug".to_string(),
        ..LogConfig::default()
    }) {
        Err(LogError::Init(message)) => assert!(
            message.contains("already been set"),
            "the failure reason must be the pre-existing global subscriber, got: {message}"
        ),
        other => panic!("second init must be rejected, got {other:?}"),
    }
}
