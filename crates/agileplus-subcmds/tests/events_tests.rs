//! Integration tests for the events subcommand.
//!
//! Covers: parse_since, filter_events, rendering, EventRecord serialization.

#![cfg(feature = "events")]

use agileplus_subcmds::{
    EventOutputFormat, EventQueryResult, EventRecord, EventsArgs, filter_events, parse_since,
    render_json, render_jsonl, render_table,
};
use chrono::{TimeZone, Utc};

// ---------------------------------------------------------------------------
// Test data factory
// ---------------------------------------------------------------------------

fn sample_events() -> Vec<EventRecord> {
    vec![
        EventRecord {
            id: 1,
            timestamp: Utc.with_ymd_and_hms(2026, 6, 15, 10, 0, 0).unwrap(),
            event_type: "feature_created".into(),
            entity_type: "feature".into(),
            entity_id: 1,
            actor: "spec-kitty".into(),
            summary: "Login flow created".into(),
            payload: serde_json::json!({"title": "Login flow"}),
        },
        EventRecord {
            id: 2,
            timestamp: Utc.with_ymd_and_hms(2026, 6, 15, 11, 30, 0).unwrap(),
            event_type: "state_changed".into(),
            entity_type: "feature".into(),
            entity_id: 1,
            actor: "user".into(),
            summary: "Login flow: created -> implementing".into(),
            payload: serde_json::json!({"from": "created", "to": "implementing"}),
        },
        EventRecord {
            id: 3,
            timestamp: Utc.with_ymd_and_hms(2026, 6, 15, 12, 0, 0).unwrap(),
            event_type: "state_changed".into(),
            entity_type: "work-package".into(),
            entity_id: 10,
            actor: "sync-oracle".into(),
            summary: "db-schema: specified -> implementing".into(),
            payload: serde_json::json!({}),
        },
        EventRecord {
            id: 4,
            timestamp: Utc.with_ymd_and_hms(2026, 6, 16, 9, 0, 0).unwrap(),
            event_type: "updated".into(),
            entity_type: "feature".into(),
            entity_id: 2,
            actor: "agileplus".into(),
            summary: "Auth feature updated".into(),
            payload: serde_json::json!({}),
        },
        EventRecord {
            id: 5,
            timestamp: Utc.with_ymd_and_hms(2026, 6, 16, 10, 0, 0).unwrap(),
            event_type: "sync_conflict".into(),
            entity_type: "feature".into(),
            entity_id: 2,
            actor: "platform".into(),
            summary: "Conflict detected on auth feature".into(),
            payload: serde_json::json!({"resolution": "LocalWins"}),
        },
    ]
}

// ---------------------------------------------------------------------------
// parse_since
// ---------------------------------------------------------------------------

#[test]
fn parse_since_minutes() {
    let dt = parse_since("30m").unwrap();
    let elapsed = Utc::now() - dt;
    assert!(elapsed.num_minutes() >= 29 && elapsed.num_minutes() <= 31);
}

#[test]
fn parse_since_one_hour() {
    let dt = parse_since("1h").unwrap();
    let elapsed = Utc::now() - dt;
    assert!(elapsed.num_hours() >= 0 && elapsed.num_hours() <= 2);
}

#[test]
fn parse_since_days() {
    let dt = parse_since("7d").unwrap();
    let elapsed = Utc::now() - dt;
    assert!(elapsed.num_days() >= 6 && elapsed.num_days() <= 8);
}

#[test]
fn parse_since_iso_date() {
    let dt = parse_since("2025-01-15").unwrap();
    // Should be midnight UTC on that date.
    assert_eq!(dt.format("%Y-%m-%d").to_string(), "2025-01-15");
}

#[test]
fn parse_since_invalid_returns_none() {
    assert!(parse_since("bogus").is_none());
    assert!(parse_since("").is_none());
    assert!(parse_since("abc123").is_none());
}

#[test]
fn parse_since_with_whitespace() {
    let dt = parse_since("  2h  ").unwrap();
    let elapsed = Utc::now() - dt;
    assert!(elapsed.num_hours() >= 0 && elapsed.num_hours() <= 2);
}

// ---------------------------------------------------------------------------
// filter_events
// ---------------------------------------------------------------------------

fn base_args() -> EventsArgs {
    EventsArgs {
        feature: None,
        since: None,
        event_type: None,
        actor: None,
        entity_type: None,
        format: EventOutputFormat::Table,
        limit: 100,
    }
}

#[test]
fn filter_no_args_returns_all() {
    let events = sample_events();
    let result = filter_events(&events, &base_args());
    assert_eq!(result.len(), 5);
}

#[test]
fn filter_by_actor() {
    let events = sample_events();
    let mut args = base_args();
    args.actor = Some("spec-kitty".into());
    let result = filter_events(&events, &args);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].actor, "spec-kitty");
}

#[test]
fn filter_by_event_type() {
    let events = sample_events();
    let mut args = base_args();
    args.event_type = Some("state_changed".into());
    let result = filter_events(&events, &args);
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|e| e.event_type == "state_changed"));
}

#[test]
fn filter_by_entity_type() {
    let events = sample_events();
    let mut args = base_args();
    args.entity_type = Some("work-package".into());
    let result = filter_events(&events, &args);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].entity_type, "work-package");
}

#[test]
fn filter_by_feature_slug_in_summary() {
    let events = sample_events();
    let mut args = base_args();
    args.feature = Some("Login".into());
    let result = filter_events(&events, &args);
    // "Login flow created" and "Login flow: created -> implementing" are features
    // and their summaries contain "Login" (case-insensitive).
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|e| e.entity_type == "feature"));
}

#[test]
fn filter_by_feature_id() {
    let events = sample_events();
    let mut args = base_args();
    args.feature = Some("2".into());
    let result = filter_events(&events, &args);
    // entity_id == 2, entity_type == "feature"
    assert_eq!(result.len(), 2);
}

#[test]
fn filter_limit_respected() {
    let events = sample_events();
    let mut args = base_args();
    args.limit = 3;
    let result = filter_events(&events, &args);
    assert_eq!(result.len(), 3);
}

#[test]
fn filter_limit_zero_returns_empty() {
    let events = sample_events();
    let mut args = base_args();
    args.limit = 0;
    let result = filter_events(&events, &args);
    assert!(result.is_empty());
}

#[test]
fn filter_combined_actor_and_type() {
    let events = sample_events();
    let mut args = base_args();
    args.actor = Some("user".into());
    args.event_type = Some("state_changed".into());
    let result = filter_events(&events, &args);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, 2);
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

#[test]
fn render_table_empty() {
    let out = render_table(&[]);
    assert!(out.contains("No events found"));
}

#[test]
fn render_table_contains_headers() {
    let events = sample_events();
    let out = render_table(&events);
    assert!(out.contains("Time"));
    assert!(out.contains("Entity"));
    assert!(out.contains("Type"));
    assert!(out.contains("Actor"));
    assert!(out.contains("Summary"));
}

#[test]
fn render_table_contains_event_data() {
    let events = sample_events();
    let out = render_table(&events);
    assert!(out.contains("feature_created"));
    assert!(out.contains("spec-kitty"));
    assert!(out.contains("Login flow created"));
}

#[test]
fn render_json_is_valid_array() {
    let events = sample_events();
    let json = render_json(&events).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 5);
}

#[test]
fn render_json_contains_expected_fields() {
    let events = &sample_events()[..1];
    let json = render_json(events).unwrap();
    assert!(json.contains("feature_created"));
    assert!(json.contains("Login flow created"));
}

#[test]
fn render_jsonl_line_count_matches() {
    let events = sample_events();
    let jsonl = render_jsonl(&events).unwrap();
    let lines: Vec<&str> = jsonl.trim_end().split('\n').collect();
    assert_eq!(lines.len(), 5);
}

#[test]
fn render_jsonl_each_line_is_valid_json_object() {
    let events = sample_events();
    let jsonl = render_jsonl(&events).unwrap();
    for line in jsonl.lines() {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(v.is_object());
    }
}

#[test]
fn render_jsonl_empty_input() {
    let jsonl = render_jsonl(&[]).unwrap();
    assert!(jsonl.is_empty());
}

#[test]
fn render_json_empty_array() {
    let json = render_json(&[]).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 0);
}

// ---------------------------------------------------------------------------
// EventRecord serialization
// ---------------------------------------------------------------------------

#[test]
fn event_record_serde_round_trip() {
    let events = sample_events();
    for event in &events {
        let json = serde_json::to_string(event).unwrap();
        let restored: EventRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, event.id);
        assert_eq!(restored.event_type, event.event_type);
        assert_eq!(restored.actor, event.actor);
        assert_eq!(restored.summary, event.summary);
    }
}

#[test]
fn event_query_result_serde() {
    let result = EventQueryResult {
        events: sample_events(),
        total: 5,
    };
    let json = serde_json::to_string(&result).unwrap();
    let restored: EventQueryResult = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.total, 5);
    assert_eq!(restored.events.len(), 5);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn filter_feature_only_matches_feature_entities() {
    let events = sample_events();
    let mut args = base_args();
    args.feature = Some("db-schema".into());
    let result = filter_events(&events, &args);
    // db-schema summary is in entity_type "work-package", not "feature"
    // so feature filter should exclude it.
    assert!(result.is_empty());
}

#[test]
fn parse_since_zero_minutes() {
    let dt = parse_since("0m").unwrap();
    let elapsed = Utc::now() - dt;
    assert!(elapsed.num_seconds() < 5);
}

#[test]
fn parse_since_large_days() {
    let dt = parse_since("365d").unwrap();
    let elapsed = Utc::now() - dt;
    assert!(elapsed.num_days() >= 364 && elapsed.num_days() <= 366);
}
