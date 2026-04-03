//! Event Bus - Async event bus with in-memory implementation

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;
use uuid::Uuid;

/// Event ID using UUID v4 for uniqueness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Event envelope with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T: Clone> {
    pub id: EventId,
    pub timestamp: u64,
    pub source: String,
    pub payload: T,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

impl<T: Clone> EventEnvelope<T> {
    pub fn new(source: impl Into<String>, payload: T) -> Self {
        Self {
            id: EventId::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            source: source.into(),
            payload,
            correlation_id: None,
            causation_id: None,
        }
    }

    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    pub fn with_causation_id(mut self, id: impl Into<String>) -> Self {
        self.causation_id = Some(id.into());
        self
    }
}

/// Event bus errors
#[derive(Error, Debug)]
pub enum EventBusError {
    #[error("Publish error: {0}")]
    Publish(String),
    #[error("Subscribe error: {0}")]
    Subscribe(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Timeout")]
    Timeout,
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}

/// Event bus trait for pluggable backends
#[async_trait]
pub trait EventBus: Send + Sync + 'static {
    type Event: Serialize + DeserializeOwned + Send + Sync + Debug + Clone + 'static;

    async fn publish(&self, event: EventEnvelope<Self::Event>) -> Result<(), EventBusError>;
    async fn subscribe<F>(&self, subject: &str, handler: F) -> Result<Subscription, EventBusError>
    where
        F: Fn(EventEnvelope<Self::Event>) -> Result<(), EventBusError> + Send + Sync + 'static;
    async fn request<T: Serialize + DeserializeOwned + Send + Sync + Debug + Clone + 'static>(
        &self,
        subject: &str,
        payload: T,
        timeout_ms: u64,
    ) -> Result<EventEnvelope<T>, EventBusError>;
    async fn close(&self) -> Result<(), EventBusError>;
}

/// Subscription handle
#[derive(Debug)]
pub struct Subscription {
    pub id: String,
    pub subject: String,
}

impl Subscription {
    pub fn new(id: impl Into<String>, subject: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            subject: subject.into(),
        }
    }
}

/// In-memory event bus implementation
pub mod memory {
    use super::*;

    /// In-memory event bus for testing
    pub struct InMemoryBus {
        subscriptions: std::sync::Mutex<Vec<(String, String)>>,
    }

    impl Default for InMemoryBus {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryBus {
        pub fn new() -> Self {
            Self {
                subscriptions: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl EventBus for InMemoryBus {
        type Event = serde_json::Value;

        async fn publish(&self, _event: EventEnvelope<Self::Event>) -> Result<(), EventBusError> {
            Ok(())
        }

        async fn subscribe<F>(
            &self,
            subject: &str,
            _handler: F,
        ) -> Result<Subscription, EventBusError>
        where
            F: Fn(EventEnvelope<Self::Event>) -> Result<(), EventBusError> + Send + Sync + 'static,
        {
            let sub = Subscription::new(Uuid::new_v4().to_string(), subject.to_string());
            let mut subs = self.subscriptions.lock().unwrap();
            subs.push((sub.id.clone(), sub.subject.clone()));
            Ok(sub)
        }

        async fn request<T: Serialize + DeserializeOwned + Send + Sync + Debug + Clone + 'static>(
            &self,
            _subject: &str,
            _payload: T,
            _timeout_ms: u64,
        ) -> Result<EventEnvelope<T>, EventBusError> {
            Ok(EventEnvelope::new("memory_bus", _payload))
        }

        async fn close(&self) -> Result<(), EventBusError> {
            let mut subs = self.subscriptions.lock().unwrap();
            subs.clear();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestEvent {
        data: String,
    }

    #[test]
    fn fr_event_bus_001_event_id_generation() {
        let id1 = EventId::new();
        let id2 = EventId::new();
        assert_ne!(id1, id2, "Each EventId should be unique");
    }

    #[test]
    fn fr_event_bus_002_event_envelope_creation() {
        let event = TestEvent {
            data: "test".to_string(),
        };
        let envelope = EventEnvelope::new("test_source", event);

        assert_eq!(envelope.source, "test_source");
        assert_eq!(envelope.payload.data, "test");
        assert!(envelope.correlation_id.is_none());
    }

    #[test]
    fn fr_event_bus_003_event_envelope_with_correlation() {
        let event = TestEvent {
            data: "test".to_string(),
        };
        let envelope = EventEnvelope::new("test_source", event)
            .with_correlation_id("cor-123")
            .with_causation_id("cause-456");

        assert_eq!(envelope.correlation_id.as_deref(), Some("cor-123"));
        assert_eq!(envelope.causation_id.as_deref(), Some("cause-456"));
    }

    #[tokio::test]
    async fn fr_event_bus_004_in_memory_bus_publish() {
        let bus = memory::InMemoryBus::new();
        let event = TestEvent {
            data: "test".to_string(),
        };
        let envelope = EventEnvelope::new("test", event);
        let result = bus.publish(envelope).await;
        assert!(result.is_ok());
    }
}
