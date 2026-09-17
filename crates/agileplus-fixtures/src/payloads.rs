//! API payload builders for testing HTTP handlers.
//!
//! Provides canonical JSON payloads for feature creation, state transitions,
//! and webhook simulations.

use serde_json::{Value, json};

/// Build a canonical JSON payload for creating a feature via the API.
pub fn feature_create_payload(title: &str, description: &str) -> Value {
    json!({
        "title": title,
        "description": description,
    })
}

/// Build a canonical JSON payload for a state transition request.
pub fn transition_payload(target_state: &str) -> Value {
    json!({
        "target_state": target_state,
    })
}

/// Build a canonical Plane.so webhook payload simulating an issue update.
pub fn plane_webhook_payload(feature_id: i64, title: &str, description: &str) -> Value {
    json!({
        "event": "issue.updated",
        "data": {
            "id": feature_id.to_string(),
            "title": title,
            "description": description,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_create_payload_is_valid_json() {
        let p = feature_create_payload("My Feature", "Some description");
        assert_eq!(p["title"], "My Feature");
        assert_eq!(p["description"], "Some description");
    }

    #[test]
    fn transition_payload_contains_target_state() {
        let p = transition_payload("specified");
        assert_eq!(p["target_state"], "specified");
    }

    #[test]
    fn plane_webhook_payload_structure() {
        let p = plane_webhook_payload(42, "Title", "Desc");
        assert_eq!(p["event"], "issue.updated");
        assert_eq!(p["data"]["id"], "42");
        assert_eq!(p["data"]["title"], "Title");
    }

    #[test]
    fn feature_create_payload_has_exactly_two_keys() {
        let p = feature_create_payload("T", "D");
        let obj = p.as_object().expect("object");
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("title"));
        assert!(obj.contains_key("description"));
    }

    #[test]
    fn feature_create_payload_handles_empty_strings() {
        let p = feature_create_payload("", "");
        assert_eq!(p["title"], "");
        assert_eq!(p["description"], "");
    }

    #[test]
    fn feature_create_payload_preserves_unicode() {
        let p = feature_create_payload("café ☕", "日本語の説明");
        assert_eq!(p["title"], "café ☕");
        assert_eq!(p["description"], "日本語の説明");
    }

    #[test]
    fn feature_create_payload_serializes_to_valid_json() {
        let p = feature_create_payload("T", "D");
        let text = serde_json::to_string(&p).expect("serialize");
        let roundtrip: Value = serde_json::from_str(&text).expect("deserialize");
        assert_eq!(roundtrip, p);
    }

    #[test]
    fn transition_payload_has_exactly_one_key() {
        let p = transition_payload("planned");
        assert_eq!(p.as_object().expect("object").len(), 1);
    }

    #[test]
    fn transition_payload_serializes_to_valid_json() {
        let p = transition_payload("implementing");
        let text = serde_json::to_string(&p).expect("serialize");
        assert!(text.contains("target_state"));
        let roundtrip: Value = serde_json::from_str(&text).expect("deserialize");
        assert_eq!(roundtrip, p);
    }

    #[test]
    fn transition_payload_accepts_every_lifecycle_state() {
        for state in [
            "created",
            "specified",
            "researched",
            "planned",
            "implementing",
            "validated",
            "shipped",
            "retrospected",
        ] {
            assert_eq!(transition_payload(state)["target_state"], state);
        }
    }

    #[test]
    fn plane_webhook_payload_has_expected_shape() {
        let p = plane_webhook_payload(1, "T", "D");
        assert_eq!(p.as_object().expect("object").len(), 2);
        let data = p["data"].as_object().expect("data object");
        assert_eq!(data.len(), 3);
        assert!(data.contains_key("id"));
        assert!(data.contains_key("title"));
        assert!(data.contains_key("description"));
    }

    #[test]
    fn plane_webhook_payload_stringifies_negative_ids() {
        let p = plane_webhook_payload(-7, "T", "D");
        assert_eq!(p["data"]["id"], "-7");
    }

    #[test]
    fn plane_webhook_payload_preserves_unicode() {
        let p = plane_webhook_payload(1, "café", "emoji 🚀");
        assert_eq!(p["data"]["title"], "café");
        assert_eq!(p["data"]["description"], "emoji 🚀");
    }

    #[test]
    fn plane_webhook_payload_serializes_to_valid_json() {
        let p = plane_webhook_payload(42, "T", "D");
        let text = serde_json::to_string(&p).expect("serialize");
        let roundtrip: Value = serde_json::from_str(&text).expect("deserialize");
        assert_eq!(roundtrip, p);
    }

    #[test]
    fn payload_builders_are_deterministic() {
        assert_eq!(
            feature_create_payload("a", "b"),
            feature_create_payload("a", "b")
        );
        assert_eq!(transition_payload("shipped"), transition_payload("shipped"));
        assert_eq!(
            plane_webhook_payload(9, "t", "d"),
            plane_webhook_payload(9, "t", "d")
        );
    }
}
