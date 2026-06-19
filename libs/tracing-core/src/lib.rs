//! Distributed tracing for AgilePlus
//!
//! Provides span-based distributed tracing with no-op implementation for testing.

pub mod error;

pub use error::{Result, TraceError};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Trace event within a span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub attributes: HashMap<String, String>,
}

impl SpanEvent {
    /// Create a new span event.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            timestamp: Utc::now(),
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute to the event.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// Span trait for distributed tracing.
pub trait Span: Send + Sync {
    /// Record an event within the span.
    fn add_event(&self, event: SpanEvent);

    /// Record an event by name.
    fn add_event_with_name(&self, name: &str) {
        self.add_event(SpanEvent::new(name));
    }

    /// Set a span attribute.
    fn set_attribute(&self, key: &str, value: &str);

    /// Record an error.
    fn record_error(&self, error: &str);

    /// End the span.
    fn end(&self);

    /// Check if span is ended.
    fn is_ended(&self) -> bool;

    /// Get span name.
    fn name(&self) -> &str;

    /// Get span ID.
    fn id(&self) -> &str;
}

/// No-operation span implementation for testing and disabled tracing.
#[derive(Debug)]
pub struct NoOpSpan {
    name: String,
    id: String,
    ended: parking_lot::RwLock<bool>,
}

impl NoOpSpan {
    /// Create a new no-op span.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            id: Self::generate_id(),
            ended: parking_lot::RwLock::new(false),
        }
    }

    fn generate_id() -> String {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        format!("{:x}-{:x}", duration.as_secs(), duration.subsec_nanos())
    }
}

impl Span for NoOpSpan {
    fn add_event(&self, _event: SpanEvent) {
        // No-op: silently ignore events
    }

    fn set_attribute(&self, _key: &str, _value: &str) {
        // No-op: silently ignore attributes
    }

    fn record_error(&self, _error: &str) {
        // No-op: silently ignore errors
    }

    fn end(&self) {
        *self.ended.write() = true;
    }

    fn is_ended(&self) -> bool {
        *self.ended.read()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> &str {
        &self.id
    }
}

/// Trace context for creating and managing spans.
#[derive(Debug, Default)]
pub struct TraceContext {
    spans: parking_lot::RwLock<Vec<String>>,
}

impl TraceContext {
    /// Create a new trace context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new span.
    pub fn start_span(&self, name: &str) -> Box<dyn Span> {
        Box::new(NoOpSpan::new(name))
    }

    /// Get the number of active spans.
    pub fn span_count(&self) -> usize {
        self.spans.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_event_creation() {
        let event = SpanEvent::new("test_event")
            .with_attribute("key", "value")
            .with_attribute("num", "42");
        assert_eq!(event.name, "test_event");
        assert_eq!(event.attributes.len(), 2);
    }

    #[test]
    fn test_noop_span_lifecycle() {
        let span = NoOpSpan::new("test_span");
        assert_eq!(span.name(), "test_span");
        assert!(!span.is_ended());
        span.end();
        assert!(span.is_ended());
    }

    #[test]
    fn test_noop_span_operations() {
        let span = NoOpSpan::new("operations");
        span.add_event_with_name("event1");
        span.add_event(SpanEvent::new("event2").with_attribute("a", "b"));
        span.set_attribute("key", "value");
        span.record_error("some error");
        assert!(!span.is_ended());
    }

    #[test]
    fn test_trace_context() {
        let ctx = TraceContext::new();
        let span = ctx.start_span("my_operation");
        assert_eq!(span.name(), "my_operation");
    }

    #[test]
    fn test_span_id_uniqueness() {
        let span1 = NoOpSpan::new("span1");
        let span2 = NoOpSpan::new("span2");
        assert_ne!(span1.id(), span2.id());
    }
}
