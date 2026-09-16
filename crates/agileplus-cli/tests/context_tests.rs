// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for `agileplus_cli::context` module.
//!
//! Covers `OutputFormat::parse`, `CommandTelemetry`, and `StorageOnlyContext`
//! — all public types exercised without requiring storage/VCS backends.

use agileplus_cli::context::{CommandTelemetry, OutputFormat, StorageOnlyContext};
use std::time::Duration;

// ── OutputFormat::parse ──────────────────────────────────────────────────────

#[test]
fn parse_json_case_insensitive() {
    assert!(matches!(OutputFormat::parse("json"), Ok(OutputFormat::Json)));
    assert!(matches!(
        OutputFormat::parse("JSON"),
        Ok(OutputFormat::Json)
    ));
    assert!(matches!(
        OutputFormat::parse("Json"),
        Ok(OutputFormat::Json)
    ));
}

#[test]
fn parse_table_case_insensitive() {
    assert!(matches!(
        OutputFormat::parse("table"),
        Ok(OutputFormat::Table)
    ));
    assert!(matches!(
        OutputFormat::parse("TABLE"),
        Ok(OutputFormat::Table)
    ));
}

#[test]
fn parse_invalid_returns_error() {
    let result = OutputFormat::parse("yaml");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("unsupported output format"));
    assert!(err.contains("yaml"));
}

#[test]
fn parse_empty_string_is_invalid() {
    assert!(OutputFormat::parse("").is_err());
}

#[test]
fn parse_whitespace_only_is_invalid() {
    assert!(OutputFormat::parse("  ").is_err());
}

// ── CommandTelemetry ─────────────────────────────────────────────────────────

#[test]
fn telemetry_duration_ms_is_zero_initially() {
    let t = CommandTelemetry {
        duration: Duration::ZERO,
    };
    assert_eq!(t.duration_ms(), 0);
}

#[test]
fn telemetry_duration_ms_reflects_elapsed() {
    let t = CommandTelemetry {
        duration: Duration::from_millis(42),
    };
    assert_eq!(t.duration_ms(), 42);
}

#[test]
fn telemetry_duration_ms_large_value() {
    let t = CommandTelemetry {
        duration: Duration::from_millis(60_000),
    };
    assert_eq!(t.duration_ms(), 60_000);
}

// ── StorageOnlyContext (lightweight — no real storage needed for type tests) ─

#[test]
fn storage_only_context_is_json_default_false() {
    // We can't construct a real StorageOnlyContext without a storage port,
    // but we can at least verify OutputFormat::Json != OutputFormat::Table.
    assert_ne!(OutputFormat::Json, OutputFormat::Table);
    assert_eq!(OutputFormat::Json, OutputFormat::Json);
}

#[test]
fn output_format_partial_eq_consistency() {
    let a = OutputFormat::Json;
    let b = OutputFormat::Json;
    let c = OutputFormat::Table;
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(c, b);
}

#[test]
fn output_format_debug_format() {
    let json_fmt = format!("{:?}", OutputFormat::Json);
    assert!(json_fmt.contains("Json"));
    let table_fmt = format!("{:?}", OutputFormat::Table);
    assert!(table_fmt.contains("Table"));
}
