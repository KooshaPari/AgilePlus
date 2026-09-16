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
}
