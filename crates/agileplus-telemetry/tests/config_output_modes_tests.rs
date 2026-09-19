//! Output-mode parsing tests for `agileplus_telemetry::config`.
//!
//! The `logging.output` field is an externally tagged enum, so the YAML tag a
//! user writes decides which writer `init_logging` installs. The crate's inline
//! tests round-trip `LogOutput` directly and cover `!file`; these tests drive all
//! three tags (plus a rejected unknown tag) through the public config loader,
//! which is the path an operator's `~/.agileplus/otel-config.yaml` actually
//! takes.

use std::io::Write;
use std::path::PathBuf;

use agileplus_telemetry::config::{ConfigError, TelemetryConfig};
use agileplus_telemetry::logs::LogOutput;

/// Load a config from a temporary YAML file.
fn load(yaml: &str) -> Result<TelemetryConfig, ConfigError> {
    let mut file = tempfile::NamedTempFile::new().expect("create temp config file");
    write!(file, "{yaml}").expect("write temp config file");
    file.flush().expect("flush temp config file");
    TelemetryConfig::load_from(file.path())
}

#[test]
fn stdout_output_is_selected_by_the_plain_string_tag() {
    let config = load("logging:\n  output: \"stdout\"\n").expect("stdout tag must parse");

    assert_eq!(config.logging.output, LogOutput::Stdout);
}

#[test]
fn file_output_is_selected_by_the_file_tag() {
    let config =
        load("logging:\n  output: !file /var/log/agileplus.log\n").expect("file tag must parse");

    assert_eq!(
        config.logging.output,
        LogOutput::File(PathBuf::from("/var/log/agileplus.log"))
    );
}

#[test]
fn both_output_is_selected_by_the_both_tag() {
    let config =
        load("logging:\n  output: !both /var/log/agileplus.log\n").expect("both tag must parse");

    assert_eq!(
        config.logging.output,
        LogOutput::Both(PathBuf::from("/var/log/agileplus.log"))
    );
}

#[test]
fn an_unknown_output_tag_is_rejected_as_a_yaml_error() {
    let error = load("logging:\n  output: !syslog /dev/log\n")
        .expect_err("an unknown output tag must not silently fall back");

    match error {
        ConfigError::Yaml(message) => {
            assert!(
                message.to_string().contains("syslog"),
                "the error must name the offending tag: {message}"
            );
        }
        other => panic!("expected a YAML parse error, got {other:?}"),
    }
}
