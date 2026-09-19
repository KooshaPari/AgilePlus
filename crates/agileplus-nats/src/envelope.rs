//! Typed message envelope wrapping domain payloads.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::subject::Subject;

/// A message envelope carrying a serialised domain payload along with
/// routing and tracing metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Unique message identifier.
    pub id: String,
    /// The subject this message was published to.
    pub subject: String,
    /// Serialised JSON payload.
    pub payload: serde_json::Value,
    /// ISO-8601 timestamp of publication.
    pub timestamp: DateTime<Utc>,
    /// Optional reply-to subject for request/reply patterns.
    pub reply_to: Option<String>,
    /// Optional correlation ID linking related messages.
    pub correlation_id: Option<String>,
}

impl Envelope {
    /// Create a new envelope for a publish operation.
    pub fn new(subject: &Subject, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            subject: subject.to_string(),
            payload,
            timestamp: Utc::now(),
            reply_to: None,
            correlation_id: None,
        }
    }

    /// Attach a reply-to subject (for request/reply).
    pub fn with_reply_to(mut self, reply_to: &Subject) -> Self {
        self.reply_to = Some(reply_to.to_string());
        self
    }

    /// Attach a correlation ID.
    pub fn with_correlation(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Serialise the envelope to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialise an envelope from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let env = Envelope::new(
            &Subject::new("agileplus.feature.1.created"),
            serde_json::json!({"title": "Login page"}),
        );
        let bytes = env.to_bytes().unwrap();
        let env2 = Envelope::from_bytes(&bytes).unwrap();
        assert_eq!(env2.subject, "agileplus.feature.1.created");
        assert_eq!(env2.payload["title"], "Login page");
    }

    #[test]
    fn with_reply_to() {
        let env = Envelope::new(&Subject::new("agileplus.rpc.triage"), serde_json::json!({}))
            .with_reply_to(&Subject::new("_INBOX.abc123"));
        assert_eq!(env.reply_to.as_deref(), Some("_INBOX.abc123"));
    }

    #[test]
    fn with_correlation_sets_id() {
        let env = Envelope::new(&Subject::new("t"), serde_json::json!({}))
            .with_correlation("corr-999");
        assert_eq!(env.correlation_id.as_deref(), Some("corr-999"));
    }

    #[test]
    fn correlation_accepts_string_ref() {
        let cid = String::from("cid-123");
        let env =
            Envelope::new(&Subject::new("t"), serde_json::json!({}))
                .with_correlation(&cid);
        assert_eq!(env.correlation_id.as_deref(), Some("cid-123"));
    }

    #[test]
    fn new_generates_unique_ids() {
        let a = Envelope::new(&Subject::new("t"), serde_json::json!({}));
        let b = Envelope::new(&Subject::new("t"), serde_json::json!({}));
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn new_sets_timestamp() {
        let before = Utc::now();
        let env =
            Envelope::new(&Subject::new("t"), serde_json::json!({}));
        let after = Utc::now();
        assert!(env.timestamp >= before);
        assert!(env.timestamp <= after);
    }

    #[test]
    fn new_reply_to_is_none_by_default() {
        let env =
            Envelope::new(&Subject::new("t"), serde_json::json!({}));
        assert_eq!(env.reply_to, None);
    }

    #[test]
    fn new_correlation_id_is_none_by_default() {
        let env =
            Envelope::new(&Subject::new("t"), serde_json::json!({}));
        assert_eq!(env.correlation_id, None);
    }

    #[test]
    fn builder_chain_reply_and_correlation() {
        let env = Envelope::new(&Subject::new("x"), serde_json::json!({}))
            .with_reply_to(&Subject::new("inbox"))
            .with_correlation("c1");
        assert_eq!(env.reply_to.as_deref(), Some("inbox"));
        assert_eq!(env.correlation_id.as_deref(), Some("c1"));
    }

    #[test]
    fn roundtrip_preserves_reply_to_and_correlation() {
        let env = Envelope::new(&Subject::new("s"), serde_json::json!({"v": 1}))
            .with_reply_to(&Subject::new("_INBOX.xy"))
            .with_correlation("rel-7");
        let bytes = env.to_bytes().unwrap();
        let env2 = Envelope::from_bytes(&bytes).unwrap();
        assert_eq!(env2.reply_to.as_deref(), Some("_INBOX.xy"));
        assert_eq!(env2.correlation_id.as_deref(), Some("rel-7"));
        assert_eq!(env2.payload["v"], 1);
    }

    #[test]
    fn from_bytes_invalid_data_returns_error() {
        let result = Envelope::from_bytes(b"not json at all {{}}");
        assert!(result.is_err());
    }

    #[test]
    fn from_bytes_empty_slice_returns_error() {
        let result = Envelope::from_bytes(b"");
        assert!(result.is_err());
    }

    #[test]
    fn from_bytes_partial_json_returns_error() {
        let result = Envelope::from_bytes(br#"{"id":"x""#);
        assert!(result.is_err());
    }

    #[test]
    fn roundtrip_complex_payload() {
        let payload = serde_json::json!({
            "features": ["a", "b", "c"],
            "metadata": {"count": 42, "nested": {"deep": true}}
        });
        let env = Envelope::new(&Subject::new("t"), payload.clone());
        let bytes = env.to_bytes().unwrap();
        let env2 = Envelope::from_bytes(&bytes).unwrap();
        assert_eq!(env2.payload, payload);
    }

    #[test]
    fn envelope_is_clone() {
        let env = Envelope::new(
            &Subject::new("clone.test"),
            serde_json::json!({"k": 1}),
        );
        let env2 = env.clone();
        assert_eq!(env.id, env2.id);
        assert_eq!(env.subject, env2.subject);
        assert_eq!(env.payload, env2.payload);
    }

    #[test]
    fn envelope_display_debug() {
        let env = Envelope::new(&Subject::new("dbg"), serde_json::json!({}));
        let debug = format!("{env:?}");
        assert!(debug.contains("Envelope"));
        assert!(debug.contains("dbg"));
    }

    #[test]
    fn to_bytes_returns_non_empty_vec() {
        let env = Envelope::new(&Subject::new("t"), serde_json::json!({}));
        let bytes = env.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn subject_field_matches_input() {
        let sub = Subject::new("agileplus.wp.5.created");
        let env = Envelope::new(&sub, serde_json::json!({}));
        assert_eq!(env.subject, "agileplus.wp.5.created");
    }

    #[test]
    fn id_is_uuid_v4_format() {
        let env = Envelope::new(&Subject::new("t"), serde_json::json!({}));
        let parts: Vec<&str> = env.id.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert!(parts[2].starts_with('4'));
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }

    #[test]
    fn timestamp_is_recent() {
        let before = chrono::Utc::now();
        let env = Envelope::new(&Subject::new("t"), serde_json::json!({}));
        let after = chrono::Utc::now();
        assert!(env.timestamp >= before && env.timestamp <= after);
    }

    #[test]
    fn roundtrip_preserves_reply_to() {
        let env = Envelope::new(&Subject::new("a.b"), serde_json::json!({"k": 1}))
            .with_reply_to(&Subject::new("_INBOX.reply"));
        let bytes = env.to_bytes().unwrap();
        let back = Envelope::from_bytes(&bytes).unwrap();
        assert_eq!(back.reply_to.as_deref(), Some("_INBOX.reply"));
    }

    #[test]
    fn roundtrip_preserves_correlation_id() {
        let env = Envelope::new(&Subject::new("a"), serde_json::json!({}))
            .with_correlation("corr-42");
        let bytes = env.to_bytes().unwrap();
        let back = Envelope::from_bytes(&bytes).unwrap();
        assert_eq!(back.correlation_id.as_deref(), Some("corr-42"));
    }

    #[test]
    fn roundtrip_preserves_payload_exactly() {
        let payload = serde_json::json!({"nested": {"arr": [1, 2, 3]}, "null": null, "bool": false});
        let env = Envelope::new(&Subject::new("t"), payload.clone());
        let bytes = env.to_bytes().unwrap();
        let back = Envelope::from_bytes(&bytes).unwrap();
        assert_eq!(back.payload, payload);
    }

    #[test]
    fn from_bytes_invalid_json_fails() {
        assert!(Envelope::from_bytes(b"not json").is_err());
    }

    #[test]
    fn from_bytes_missing_fields_fails() {
        let bytes = serde_json::to_vec(&serde_json::json!({"subject": "x"})).unwrap();
        assert!(Envelope::from_bytes(&bytes).is_err());
    }

    #[test]
    fn reply_to_none_by_default() {
        let env = Envelope::new(&Subject::new("t"), serde_json::json!({}));
        assert!(env.reply_to.is_none());
    }

    #[test]
    fn correlation_id_none_by_default() {
        let env = Envelope::new(&Subject::new("t"), serde_json::json!({}));
        assert!(env.correlation_id.is_none());
    }

    #[test]
    fn clone_is_deep() {
        let env = Envelope::new(&Subject::new("t"), serde_json::json!({"v": 1}))
            .with_correlation("c");
        let copy = env.clone();
        assert_eq!(env.id, copy.id);
        assert_eq!(env.subject, copy.subject);
        assert_eq!(env.payload, copy.payload);
        assert_eq!(env.reply_to, copy.reply_to);
        assert_eq!(copy.correlation_id.as_deref(), Some("c"));
    }

    #[test]
    fn debug_output_contains_fields() {
        let env = Envelope::new(&Subject::new("debug.test"), serde_json::json!({"x": 1}))
            .with_correlation("dbg-1")
            .with_reply_to(&Subject::new("_INBOX.r"));
        let s = format!("{env:?}");
        assert!(s.contains("debug.test"));
        assert!(s.contains("dbg-1"));
        assert!(s.contains("_INBOX.r"));
    }

    #[test]
    fn missing_each_required_field_is_rejected() {
        let full = serde_json::json!({
            "id": "00000000-0000-4000-8000-000000000000",
            "subject": "a.b",
            "payload": {},
            "timestamp": "2026-01-02T03:04:05Z",
        });
        assert!(
            serde_json::from_value::<Envelope>(full.clone()).is_ok(),
            "full document must parse"
        );
        for field in ["id", "subject", "payload", "timestamp"] {
            let mut partial = full.clone();
            partial.as_object_mut().unwrap().remove(field);
            let res = serde_json::from_value::<Envelope>(partial);
            assert!(
                res.is_err(),
                "removing `{field}` must break deserialization"
            );
        }
    }

    #[test]
    fn wrong_type_for_each_required_field_is_rejected() {
        for field in ["id", "subject", "timestamp"] {
            let doc = serde_json::json!({
                "id": "00000000-0000-4000-8000-000000000000",
                "subject": "a.b",
                "payload": {},
                "timestamp": "2026-01-02T03:04:05Z",
                field: 42,
            });
            assert!(
                serde_json::from_value::<Envelope>(doc).is_err(),
                "numeric `{field}` must be rejected"
            );
        }
        // payload, however, is a free-form Value and accepts non-objects.
        let doc = serde_json::json!({
            "id": "00000000-0000-4000-8000-000000000000",
            "subject": "a.b",
            "payload": "just-a-string",
            "timestamp": "2026-01-02T03:04:05Z",
        });
        let env = serde_json::from_value::<Envelope>(doc).unwrap();
        assert_eq!(env.payload, serde_json::json!("just-a-string"));
    }

    #[test]
    fn optional_fields_accept_any_json_value_type_or_null() {
        let base = serde_json::json!({
            "id": "00000000-0000-4000-8000-000000000000",
            "subject": "a.b",
            "payload": {},
            "timestamp": "2026-01-02T03:04:05Z",
        });
        // Explicit null deserializes to None (same as absent).
        let mut with_nulls = base.clone();
        with_nulls["reply_to"] = serde_json::Value::Null;
        with_nulls["correlation_id"] = serde_json::Value::Null;
        let env = serde_json::from_value::<Envelope>(with_nulls).unwrap();
        assert_eq!(env.reply_to, None);
        assert_eq!(env.correlation_id, None);

        // Wrong type for an optional field is still rejected.
        let mut bad_reply = base.clone();
        bad_reply["reply_to"] = serde_json::json!(7);
        assert!(serde_json::from_value::<Envelope>(bad_reply).is_err());
        let mut bad_corr = base.clone();
        bad_corr["correlation_id"] = serde_json::json!({ "nested": true });
        assert!(serde_json::from_value::<Envelope>(bad_corr).is_err());
    }

    #[test]
    fn unknown_extra_fields_are_tolerated_and_dropped() {
        let doc = serde_json::json!({
            "id": "00000000-0000-4000-8000-000000000000",
            "subject": "a.b",
            "payload": {"k": 1},
            "timestamp": "2026-01-02T03:04:05Z",
            "schema_version": 2,
            "future_field": ["anything"],
        });
        let env = serde_json::from_value::<Envelope>(doc).unwrap();
        assert_eq!(env.subject, "a.b");
        assert_eq!(env.payload["k"], 1);
        // Extras must not surface anywhere: a round-trip loses them.
        let round = Envelope::from_bytes(&env.to_bytes().unwrap()).unwrap();
        let serialized: serde_json::Value =
            serde_json::from_slice(&round.to_bytes().unwrap()).unwrap();
        assert!(serialized.get("schema_version").is_none());
        assert!(serialized.get("future_field").is_none());
    }

    #[test]
    fn malformed_timestamp_is_rejected() {
        let base = serde_json::json!({
            "id": "00000000-0000-4000-8000-000000000000",
            "subject": "a.b",
            "payload": {},
        });
        for bad in ["not-a-date", "2026-13-45T99:99:99Z", "", "2026-01-02"] {
            let mut doc = base.clone();
            doc["timestamp"] = serde_json::json!(bad);
            assert!(
                serde_json::from_value::<Envelope>(doc).is_err(),
                "timestamp {bad:?} must be rejected"
            );
        }
    }

    #[test]
    fn non_utc_timestamp_offset_is_accepted_as_utc() {
        // chrono's DateTime<Utc> deserializer accepts RFC3339 with an explicit
        // non-zero offset and normalizes it to UTC.
        let doc = serde_json::json!({
            "id": "00000000-0000-4000-8000-000000000000",
            "subject": "a.b",
            "payload": {},
            "timestamp": "2026-01-02T05:04:05+02:00",
        });
        let env = serde_json::from_value::<Envelope>(doc).unwrap();
        assert_eq!(env.timestamp.to_rfc3339(), "2026-01-02T03:04:05+00:00");
    }

    #[test]
    fn timestamp_roundtrip_preserves_instant_exactly() {
        let env = Envelope::new(&Subject::new("t"), serde_json::json!({}));
        let back = Envelope::from_bytes(&env.to_bytes().unwrap()).unwrap();
        assert_eq!(back.timestamp, env.timestamp);
        assert_eq!(back.timestamp.timestamp_subsec_nanos() >= 0, true);
        // Nanosecond precision survives the wire.
        let back2 = Envelope::from_bytes(&back.to_bytes().unwrap()).unwrap();
        assert_eq!(back2.timestamp, back.timestamp);
    }

    #[test]
    fn payload_accepts_arrays_strings_numbers_and_null() {
        for payload in [
            serde_json::json!([1, 2, 3]),
            serde_json::json!("scalar"),
            serde_json::json!(3.14),
            serde_json::json!(true),
            serde_json::Value::Null,
        ] {
            let env = Envelope::new(&Subject::new("t"), payload.clone());
            let back = Envelope::from_bytes(&env.to_bytes().unwrap()).unwrap();
            assert_eq!(back.payload, payload);
        }
    }

    #[test]
    fn to_bytes_is_valid_json_object_with_all_required_keys() {
        let env = Envelope::new(&Subject::new("a.b.c"), serde_json::json!({}))
            .with_reply_to(&Subject::new("inbox"))
            .with_correlation("cid");
        let value: serde_json::Value =
            serde_json::from_slice(&env.to_bytes().unwrap()).unwrap();
        let obj = value.as_object().expect("envelope must serialize to a JSON object");
        for key in ["id", "subject", "payload", "timestamp", "reply_to", "correlation_id"] {
            assert!(obj.contains_key(key), "serialized envelope must contain `{key}`");
        }
        assert_eq!(obj["subject"], "a.b.c");
        assert_eq!(obj["correlation_id"], "cid");
    }

    #[test]
    fn from_bytes_trailing_garbage_is_rejected() {
        let env = Envelope::new(&Subject::new("t"), serde_json::json!({}));
        let mut bytes = env.to_bytes().unwrap();
        bytes.extend_from_slice(b"trailing");
        assert!(Envelope::from_bytes(&bytes).is_err());
    }

    #[test]
    fn from_bytes_utf8_invalid_bytes_are_rejected() {
        // Build a document whose subject value is a known unique marker, then
        // corrupt one of its bytes to invalid UTF-8.
        let marker = "ZZSUBJECTZZ";
        let env = Envelope::new(&Subject::new(marker), serde_json::json!({"k": "v"}));
        let mut bytes = env.to_bytes().unwrap();
        let needle = format!("\"{marker}\"");
        let idx = bytes
            .windows(needle.len())
            .position(|w| w == needle.as_bytes())
            .expect("subject value present in serialized envelope");
        bytes[idx + 1] = 0xFF; // first byte inside the subject string
        assert!(Envelope::from_bytes(&bytes).is_err());
    }

    #[test]
    fn huge_payload_roundtrips() {
        // ~100k entries: stresses serialization without wall-clock dependence.
        let items: Vec<i32> = (0..100_000).collect();
        let env = Envelope::new(&Subject::new("big"), serde_json::json!({ "items": items }));
        let back = Envelope::from_bytes(&env.to_bytes().unwrap()).unwrap();
        assert_eq!(back.payload["items"].as_array().map(Vec::len), Some(100_000));
        assert_eq!(back.payload["items"][0], 0);
        assert_eq!(back.payload["items"][99_999], 99_999);
    }
}
