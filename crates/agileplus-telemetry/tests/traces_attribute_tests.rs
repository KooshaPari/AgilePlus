//! Span-attribute tests for `agileplus_telemetry::traces`.
//!
//! These tests install a thread-scoped JSON `tracing` subscriber that writes into
//! a buffer, exercise the span helpers, and then assert on the span fields and
//! span paths that were actually emitted — not just that the helpers did not
//! panic. The crate's inline tests only covered the latter.

use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use agileplus_telemetry::traces::{
    ATTR_AGENT_TYPE, ATTR_COMMAND, ATTR_FEATURE_SLUG, ATTR_REVIEW_CYCLE, ATTR_WP_ID, SpanGuard,
    create_agent_span, create_command_span, create_review_span, record_span_event,
};
use tracing_subscriber::prelude::*;

// ---------------------------------------------------------------------------
// Capture harness
// ---------------------------------------------------------------------------

/// A `MakeWriter` that appends formatted log lines into a shared buffer.
#[derive(Clone, Default)]
struct CaptureWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer
            .lock()
            .expect("capture buffer poisoned")
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CaptureWriter {
    type Writer = CaptureWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Run `body` under a thread-local JSON subscriber and return the emitted lines.
///
/// The subscriber is scoped with `tracing::subscriber::with_default`, so this is
/// deterministic and never fights other tests for the global subscriber slot.
fn capture_lines<F: FnOnce()>(body: F) -> String {
    let writer = CaptureWriter::default();
    let buffer = Arc::clone(&writer.buffer);
    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::fmt::layer()
            .json()
            .with_writer(writer)
            .with_target(true)
            .with_span_list(true)
            .with_current_span(true),
    );

    tracing::subscriber::with_default(subscriber, body);

    let bytes = buffer.lock().expect("capture buffer poisoned").clone();
    String::from_utf8(bytes).expect("captured output is UTF-8")
}

/// Parse the first captured JSON line. JSON is a YAML subset, so `serde_yaml`
/// reads it without pulling in another dev-dependency.
fn first_line(output: &str) -> serde_yaml::Value {
    let line = output
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_else(|| panic!("no log lines were captured; output was {output:?}"));
    serde_yaml::from_str(line)
        .unwrap_or_else(|err| panic!("captured line is not valid JSON/YAML ({err}): {line}"))
}

/// Recursively find a field by name anywhere in the parsed line.
///
/// The JSON formatter nests the current span under `span` and the span path under
/// `spans`, so a plain path lookup would be brittle. Searching recursively also
/// proves the field is present *somewhere* in the emitted span context.
fn find_field<'a>(value: &'a serde_yaml::Value, field: &str) -> Option<&'a serde_yaml::Value> {
    match value {
        serde_yaml::Value::Mapping(map) => {
            if let Some(found) = map.get(serde_yaml::Value::String(field.to_string())) {
                return Some(found);
            }
            map.values().find_map(|nested| find_field(nested, field))
        }
        serde_yaml::Value::Sequence(sequence) => {
            sequence.iter().find_map(|nested| find_field(nested, field))
        }
        _ => None,
    }
}

fn text_field<'a>(value: &'a serde_yaml::Value, field: &str) -> Option<&'a str> {
    find_field(value, field).and_then(serde_yaml::Value::as_str)
}

/// A custom field attached to an emitted event (`fields.<name>` in the JSON line).
fn event_field<'a>(line: &'a serde_yaml::Value, name: &str) -> Option<&'a serde_yaml::Value> {
    line.get("fields").and_then(|fields| fields.get(name))
}

/// The emitted span path, root first.
fn span_path(line: &serde_yaml::Value) -> &[serde_yaml::Value] {
    line["spans"]
        .as_sequence()
        .map(Vec::as_slice)
        .unwrap_or_else(|| panic!("no span path was emitted: {line:?}"))
}

fn span_names(line: &serde_yaml::Value) -> Vec<&str> {
    span_path(line)
        .iter()
        .filter_map(|span| span["name"].as_str())
        .collect()
}

// ---------------------------------------------------------------------------
// Command spans
// ---------------------------------------------------------------------------

#[test]
fn command_span_records_command_and_feature_slug() {
    let output = capture_lines(|| {
        let span = create_command_span("implement", Some("001-sde"));
        let _entered = span.enter();
        tracing::info!("marker");
    });
    let line = first_line(&output);

    assert_eq!(text_field(&line, ATTR_COMMAND), Some("implement"));
    assert_eq!(text_field(&line, ATTR_FEATURE_SLUG), Some("001-sde"));
    assert_eq!(span_names(&line), vec!["agileplus.command"]);
}

#[test]
fn command_span_without_feature_slug_omits_that_field() {
    let output = capture_lines(|| {
        let span = create_command_span("list", None);
        let _entered = span.enter();
        tracing::info!("marker");
    });
    let line = first_line(&output);

    assert_eq!(text_field(&line, ATTR_COMMAND), Some("list"));
    assert!(
        find_field(&line, ATTR_FEATURE_SLUG).is_none(),
        "no feature slug field must be emitted when no slug is given: {line:?}"
    );
}

#[test]
fn command_span_field_names_match_the_published_attribute_constants() {
    let output = capture_lines(|| {
        let span = create_command_span("implement", Some("001-sde"));
        let _entered = span.enter();
        tracing::info!("marker");
    });

    // The constants are the contract with dashboards; assert they appear verbatim.
    assert!(output.contains(ATTR_COMMAND), "output: {output}");
    assert!(output.contains(ATTR_FEATURE_SLUG), "output: {output}");
    assert_eq!(ATTR_COMMAND, "agileplus.command");
    assert_eq!(ATTR_FEATURE_SLUG, "agileplus.feature.slug");
    assert_eq!(ATTR_WP_ID, "agileplus.wp.id");
    assert_eq!(ATTR_AGENT_TYPE, "agileplus.agent.type");
    assert_eq!(ATTR_REVIEW_CYCLE, "agileplus.review.cycle");
}

// ---------------------------------------------------------------------------
// Child spans
// ---------------------------------------------------------------------------

#[test]
fn agent_span_records_wp_id_and_agent_type_and_nests_under_the_command_span() {
    let output = capture_lines(|| {
        let command = create_command_span("implement", Some("001-sde"));
        let agent = create_agent_span(&command, "WP10", "claude-code");
        let _entered = agent.enter();
        tracing::info!("marker");
    });
    let line = first_line(&output);

    assert_eq!(text_field(&line, ATTR_WP_ID), Some("WP10"));
    assert_eq!(text_field(&line, ATTR_AGENT_TYPE), Some("claude-code"));
    assert_eq!(
        span_names(&line),
        vec!["agileplus.command", "agileplus.agent"],
        "the agent span must be a child of the command span"
    );
    assert_eq!(text_field(&line, ATTR_COMMAND), Some("implement"));
}

#[test]
fn review_span_records_the_cycle_number_as_a_number() {
    let output = capture_lines(|| {
        let command = create_command_span("review", None);
        let agent = create_agent_span(&command, "WP1", "codex");
        let review = create_review_span(&agent, 3);
        let _entered = review.enter();
        tracing::info!("marker");
    });
    let line = first_line(&output);

    let cycle = find_field(&line, ATTR_REVIEW_CYCLE).expect("cycle field must be emitted");
    assert_eq!(
        cycle.as_i64(),
        Some(3),
        "the cycle is recorded as an integer, not a string: {cycle:?}"
    );
    assert_eq!(
        span_names(&line),
        vec!["agileplus.command", "agileplus.agent", "agileplus.review"]
    );
}

#[test]
fn nested_spans_report_the_full_ancestor_chain_root_first() {
    let output = capture_lines(|| {
        let command = create_command_span("implement", Some("001-sde"));
        let agent = create_agent_span(&command, "WP16", "codex");
        let review = create_review_span(&agent, 1);
        let _entered = review.enter();
        tracing::info!("marker");
    });
    let line = first_line(&output);
    let path = span_path(&line);

    assert_eq!(path.len(), 3);
    assert_eq!(path[0]["name"].as_str(), Some("agileplus.command"));
    assert_eq!(path[0][ATTR_FEATURE_SLUG].as_str(), Some("001-sde"));
    assert_eq!(path[1]["name"].as_str(), Some("agileplus.agent"));
    assert_eq!(path[1][ATTR_WP_ID].as_str(), Some("WP16"));
    assert_eq!(path[2]["name"].as_str(), Some("agileplus.review"));
    assert_eq!(path[2][ATTR_REVIEW_CYCLE].as_i64(), Some(1));
}

// ---------------------------------------------------------------------------
// Milestone events
// ---------------------------------------------------------------------------

#[test]
fn record_span_event_emits_the_event_name_and_attribute_pairs() {
    let output = capture_lines(|| {
        let span = create_command_span("implement", Some("001-sde"));
        record_span_event(
            &span,
            "pr_created",
            &[
                ("wp_id".to_string(), "WP10".to_string()),
                ("run_id".to_string(), "42".to_string()),
            ],
        );
    });
    let line = first_line(&output);

    assert_eq!(
        event_field(&line, "event").and_then(serde_yaml::Value::as_str),
        Some("pr_created")
    );
    let fields = event_field(&line, "fields")
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_else(|| panic!("attribute list must be emitted as text: {line:?}"));
    assert!(fields.contains("wp_id=WP10"), "fields: {fields}");
    assert!(fields.contains("run_id=42"), "fields: {fields}");
    // The event is recorded on the span, so the span fields travel with it.
    assert_eq!(text_field(&line, ATTR_COMMAND), Some("implement"));
}

#[test]
fn record_span_event_without_attributes_emits_an_empty_list() {
    let output = capture_lines(|| {
        let span = create_command_span("test", None);
        record_span_event(&span, "noop", &[]);
    });
    let line = first_line(&output);

    assert_eq!(
        event_field(&line, "event").and_then(serde_yaml::Value::as_str),
        Some("noop")
    );
    assert_eq!(
        event_field(&line, "fields").and_then(serde_yaml::Value::as_str),
        Some("[]"),
        "line: {line:?}"
    );
}

// ---------------------------------------------------------------------------
// SpanGuard
// ---------------------------------------------------------------------------

#[test]
fn dropping_a_span_guard_leaves_the_wrapped_span_intact() {
    let output = capture_lines(|| {
        let span = create_command_span("implement", Some("001-sde"));
        drop(SpanGuard::new(span.clone()));
        let _entered = span.enter();
        tracing::info!("marker");
    });
    let line = first_line(&output);

    assert_eq!(text_field(&line, ATTR_COMMAND), Some("implement"));
    assert_eq!(text_field(&line, ATTR_FEATURE_SLUG), Some("001-sde"));
}

#[test]
fn span_guard_command_wraps_a_usable_command_span() {
    let output = capture_lines(|| {
        let guard = SpanGuard::command("apply", Some("002-x"));
        let _entered = guard.span.enter();
        tracing::info!("marker");
    });
    let line = first_line(&output);

    assert_eq!(text_field(&line, ATTR_COMMAND), Some("apply"));
    assert_eq!(text_field(&line, ATTR_FEATURE_SLUG), Some("002-x"));
    assert_eq!(span_names(&line), vec!["agileplus.command"]);
}

// ---------------------------------------------------------------------------
// Subscriber isolation
// ---------------------------------------------------------------------------

#[test]
fn spans_created_outside_the_capture_scope_emit_no_span_context() {
    // A span created while no subscriber is installed has no span id, so entering
    // it later must not add span fields to the event. This pins the harness
    // behaviour the other tests rely on.
    let detached = create_command_span("detached", Some("003-y"));
    let output = capture_lines(|| {
        let _entered = detached.enter();
        tracing::info!("marker");
    });
    let line = first_line(&output);

    assert!(find_field(&line, ATTR_COMMAND).is_none(), "line: {line:?}");
    assert!(line.get("spans").is_none(), "line: {line:?}");
}
