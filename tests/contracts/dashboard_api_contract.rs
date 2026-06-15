//! T114 — `agileplus-api` ↔ `agileplus-dashboard` API response shape contract.
//!
//! Verifies that the `EventResponse` shape returned by the events API matches
//! what the dashboard templates expect: every field the dashboard renders
//! must be present in the serialized JSON.

use agileplus_api::routes::events::EventResponse;
use serde_json::Value;

/// Contract: every field the dashboard renders is present in the serialized
/// response and is the right JSON type. The dashboard template references:
/// `id`, `entity_type`, `entity_id`, `event_type`, `actor`, `timestamp`,
/// and the `payload` object.
#[test]
fn event_response_shape_matches_dashboard_contract() {
    let resp = EventResponse {
        id: 1,
        entity_type: "feature".to_string(),
        entity_id: 42,
        event_type: "created".to_string(),
        actor: "alice".to_string(),
        timestamp: "2026-06-14T12:00:00Z".to_string(),
        payload: serde_json::json!({"wp_id": null, "transition": "created"}),
    };

    let json: Value = serde_json::to_value(&resp).expect("serialize");

    assert_eq!(json["id"], 1);
    assert_eq!(json["entity_type"], "feature");
    assert_eq!(json["entity_id"], 42);
    assert_eq!(json["event_type"], "created");
    assert_eq!(json["actor"], "alice");
    assert_eq!(json["timestamp"], "2026-06-14T12:00:00Z");
    assert!(json["payload"].is_object(), "payload must be a JSON object for the dashboard");
    assert_eq!(json["payload"]["transition"], "created");
}

/// Contract: timestamps are RFC-3339 — the dashboard's `datetime` filter
/// will fail to parse anything else.
#[test]
fn event_response_timestamp_is_rfc3339() {
    let resp = EventResponse {
        id: 0,
        entity_type: "work_package".to_string(),
        entity_id: 1,
        event_type: "wp_done".to_string(),
        actor: "system".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        payload: serde_json::json!({}),
    };
    let json: Value = serde_json::to_value(&resp).unwrap();
    let ts = json["timestamp"].as_str().expect("timestamp must be a string");
    chrono::DateTime::parse_from_rfc3339(ts).expect("timestamp must be RFC-3339");
}

/// Contract: payload may be an empty object (no required keys); the dashboard
/// must tolerate this for legacy events written before the payload schema
/// was introduced.
#[test]
fn event_response_allows_empty_payload() {
    let resp = EventResponse {
        id: 0,
        entity_type: "feature".to_string(),
        entity_id: 1,
        event_type: "legacy".to_string(),
        actor: "system".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        payload: serde_json::json!({}),
    };
    let json: Value = serde_json::to_value(&resp).unwrap();
    assert!(json["payload"].as_object().unwrap().is_empty());
}
