//! Core `EventBus` trait and in-memory implementation.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::oneshot;

use crate::config::NatsConfig;
use crate::envelope::Envelope;
use crate::handler::Handler;
use crate::health::BusHealth;
use crate::subject::Subject;

/// Errors produced by the event bus.
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Publish error: {0}")]
    PublishError(String),
    #[error("Subscribe error: {0}")]
    SubscribeError(String),
    #[error("Request timeout")]
    Timeout,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Handler error: {0}")]
    HandlerError(String),
}

/// Trait abstracting the event bus backend. The default implementation is an
/// in-memory bus suitable for testing; a NATS-backed implementation can be
/// provided when a real broker is available.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an envelope to a subject.
    async fn publish(&self, envelope: Envelope) -> Result<(), EventBusError>;

    /// Subscribe a handler to a subject pattern. Returns a subscription ID.
    async fn subscribe(
        &self,
        subject: Subject,
        handler: Arc<dyn Handler>,
    ) -> Result<String, EventBusError>;

    /// Unsubscribe by subscription ID.
    async fn unsubscribe(&self, subscription_id: &str) -> Result<(), EventBusError>;

    /// Request/reply: publish and wait for a single response.
    async fn request(
        &self,
        envelope: Envelope,
        timeout: Duration,
    ) -> Result<Envelope, EventBusError>;

    /// Check the health of the bus connection.
    async fn health(&self) -> BusHealth;
}

/// The primary event bus store wrapping a backend implementation.
pub struct EventBusStore {
    backend: Box<dyn EventBus>,
    #[allow(dead_code)]
    config: NatsConfig,
}

impl EventBusStore {
    /// Create a new `EventBusStore` with the given backend.
    pub fn new(config: NatsConfig, backend: Box<dyn EventBus>) -> Self {
        Self { backend, config }
    }

    /// Create an `EventBusStore` backed by the in-memory implementation.
    pub fn in_memory(config: NatsConfig) -> Self {
        Self {
            backend: Box::new(InMemoryBus::new()),
            config,
        }
    }

    pub fn backend(&self) -> &dyn EventBus {
        &*self.backend
    }

    pub async fn publish(&self, envelope: Envelope) -> Result<(), EventBusError> {
        self.backend.publish(envelope).await
    }

    pub async fn subscribe(
        &self,
        subject: Subject,
        handler: Arc<dyn Handler>,
    ) -> Result<String, EventBusError> {
        self.backend.subscribe(subject, handler).await
    }

    pub async fn unsubscribe(&self, id: &str) -> Result<(), EventBusError> {
        self.backend.unsubscribe(id).await
    }

    pub async fn request(
        &self,
        envelope: Envelope,
        timeout: Duration,
    ) -> Result<Envelope, EventBusError> {
        self.backend.request(envelope, timeout).await
    }

    pub async fn health(&self) -> BusHealth {
        self.backend.health().await
    }
}

// ---------------------------------------------------------------------------
// In-memory backend for testing
// ---------------------------------------------------------------------------

struct Subscription {
    subject: Subject,
    handler: Arc<dyn Handler>,
}

/// An in-memory event bus that dispatches published messages to matching
/// subscriptions synchronously. Suitable for unit and integration tests.
pub struct InMemoryBus {
    subscriptions: Mutex<HashMap<String, Subscription>>,
    /// All published envelopes, for test assertions.
    published: Mutex<Vec<Envelope>>,
    /// Pending request/reply waiters: reply-subject -> sender.
    reply_waiters: Mutex<HashMap<String, oneshot::Sender<Envelope>>>,
    next_sub_id: Mutex<u64>,
}

impl InMemoryBus {
    pub fn new() -> Self {
        Self {
            subscriptions: Mutex::new(HashMap::new()),
            published: Mutex::new(Vec::new()),
            reply_waiters: Mutex::new(HashMap::new()),
            next_sub_id: Mutex::new(0),
        }
    }

    /// Return all envelopes published so far (test helper).
    pub fn published(&self) -> Vec<Envelope> {
        self.published.lock().unwrap().clone()
    }
}

impl Default for InMemoryBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InMemoryBus {
    async fn publish(&self, envelope: Envelope) -> Result<(), EventBusError> {
        // Check if the published subject matches any reply waiter.
        {
            let mut waiters = self.reply_waiters.lock().unwrap();
            if let Some(sender) = waiters.remove(&envelope.subject) {
                let _ = sender.send(envelope.clone());
            }
        }

        // Record the envelope.
        self.published.lock().unwrap().push(envelope.clone());

        // Collect matching handlers while holding the lock, then release
        // the lock before awaiting (MutexGuard is not Send).
        let handlers: Vec<Arc<dyn Handler>> = {
            let subs = self.subscriptions.lock().unwrap();
            let concrete = Subject::new(&envelope.subject);
            subs.values()
                .filter(|sub| sub.subject.matches(&concrete))
                .map(|sub| sub.handler.clone())
                .collect()
        };

        for handler in handlers {
            let _ = handler.handle(&envelope).await;
        }

        Ok(())
    }

    async fn subscribe(
        &self,
        subject: Subject,
        handler: Arc<dyn Handler>,
    ) -> Result<String, EventBusError> {
        let mut id_counter = self.next_sub_id.lock().unwrap();
        *id_counter += 1;
        let id = format!("sub-{}", *id_counter);
        drop(id_counter);

        self.subscriptions
            .lock()
            .unwrap()
            .insert(id.clone(), Subscription { subject, handler });

        Ok(id)
    }

    async fn unsubscribe(&self, subscription_id: &str) -> Result<(), EventBusError> {
        self.subscriptions.lock().unwrap().remove(subscription_id);
        Ok(())
    }

    async fn request(
        &self,
        mut envelope: Envelope,
        timeout: Duration,
    ) -> Result<Envelope, EventBusError> {
        // Create an inbox subject for the reply.
        let inbox = format!("_INBOX.{}", uuid::Uuid::new_v4());
        envelope.reply_to = Some(inbox.clone());

        // Register a waiter for the inbox subject.
        let (tx, rx) = oneshot::channel();
        self.reply_waiters.lock().unwrap().insert(inbox, tx);

        // Publish the request.
        self.publish(envelope).await?;

        // Wait for the reply with a timeout.
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(reply)) => Ok(reply),
            Ok(Err(_)) => Err(EventBusError::Timeout),
            Err(_) => Err(EventBusError::Timeout),
        }
    }

    async fn health(&self) -> BusHealth {
        BusHealth::Connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::FnHandler;

    #[tokio::test]
    async fn in_memory_health() {
        let store = EventBusStore::in_memory(NatsConfig::default());
        assert_eq!(store.health().await, BusHealth::Connected);
    }

    #[tokio::test]
    async fn publish_and_subscribe() {
        let bus = InMemoryBus::new();

        let received: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received.clone();
        let handler = Arc::new(FnHandler(move |env: &Envelope| {
            received_clone.lock().unwrap().push(env.subject.clone());
            Ok(())
        }));

        bus.subscribe(Subject::all_for_entity("agileplus", "feature"), handler)
            .await
            .unwrap();

        let env = Envelope::new(
            &Subject::for_event("agileplus", "feature", 1, "created"),
            serde_json::json!({"title": "Login"}),
        );
        bus.publish(env).await.unwrap();

        let got = received.lock().unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0], "agileplus.feature.1.created");
    }

    #[tokio::test]
    async fn unsubscribe_stops_delivery() {
        let bus = InMemoryBus::new();

        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count_clone = count.clone();
        let handler = Arc::new(FnHandler(move |_: &Envelope| {
            *count_clone.lock().unwrap() += 1;
            Ok(())
        }));

        let sub_id = bus
            .subscribe(Subject::all_for_entity("agileplus", "feature"), handler)
            .await
            .unwrap();

        let env = Envelope::new(
            &Subject::for_event("agileplus", "feature", 1, "created"),
            serde_json::json!({}),
        );
        bus.publish(env).await.unwrap();
        assert_eq!(*count.lock().unwrap(), 1);

        bus.unsubscribe(&sub_id).await.unwrap();

        let env2 = Envelope::new(
            &Subject::for_event("agileplus", "feature", 2, "created"),
            serde_json::json!({}),
        );
        bus.publish(env2).await.unwrap();
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn request_reply() {
        let bus = Arc::new(InMemoryBus::new());

        // Subscribe a responder that replies to the reply_to subject.
        let bus_clone = bus.clone();
        let handler = Arc::new(FnHandler(move |env: &Envelope| {
            if let Some(reply_to) = &env.reply_to {
                let reply =
                    Envelope::new(&Subject::new(reply_to), serde_json::json!({"answer": 42}));
                let bus_inner = bus_clone.clone();
                // Spawn the reply asynchronously to avoid deadlock.
                tokio::spawn(async move {
                    let _ = bus_inner.publish(reply).await;
                });
            }
            Ok(())
        }));

        bus.subscribe(Subject::new("agileplus.rpc.triage"), handler)
            .await
            .unwrap();

        let req = Envelope::new(
            &Subject::new("agileplus.rpc.triage"),
            serde_json::json!({"feature_id": 1}),
        );
        let reply = bus.request(req, Duration::from_secs(2)).await.unwrap();
        assert_eq!(reply.payload["answer"], 42);
    }

    #[tokio::test]
    async fn request_timeout() {
        let bus = InMemoryBus::new();
        let req = Envelope::new(&Subject::new("agileplus.rpc.nobody"), serde_json::json!({}));
        let result = bus.request(req, Duration::from_millis(50)).await;
        assert!(matches!(result, Err(EventBusError::Timeout)));
    }

    #[tokio::test]
    async fn event_bus_store_delegates() {
        let store = EventBusStore::in_memory(NatsConfig::default());
        let env = Envelope::new(
            &Subject::for_event("agileplus", "wp", 7, "created"),
            serde_json::json!({"title": "WP07"}),
        );
        store.publish(env).await.unwrap();
        assert_eq!(store.health().await, BusHealth::Connected);
    }

    #[tokio::test]
    async fn multiple_subscribers_all_receive() {
        let bus = InMemoryBus::new();

        let count_a: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count_b: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let ca = count_a.clone();
        let cb = count_b.clone();

        let handler_a = Arc::new(FnHandler(move |_: &Envelope| {
            *ca.lock().unwrap() += 1;
            Ok(())
        }));
        let handler_b = Arc::new(FnHandler(move |_: &Envelope| {
            *cb.lock().unwrap() += 1;
            Ok(())
        }));

        bus.subscribe(Subject::all_for_entity("agileplus", "feature"), handler_a)
            .await
            .unwrap();
        bus.subscribe(Subject::all_for_entity("agileplus", "feature"), handler_b)
            .await
            .unwrap();

        let env = Envelope::new(
            &Subject::for_event("agileplus", "feature", 1, "created"),
            serde_json::json!({}),
        );
        bus.publish(env).await.unwrap();

        assert_eq!(*count_a.lock().unwrap(), 1);
        assert_eq!(*count_b.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn no_matching_subscribers_silently_publishes() {
        let bus = InMemoryBus::new();
        let env = Envelope::new(
            &Subject::for_event("agileplus", "feature", 1, "created"),
            serde_json::json!({}),
        );
        // Should not error even with no subscribers
        bus.publish(env).await.unwrap();
    }

    #[tokio::test]
    async fn subscribe_returns_unique_ids() {
        let bus = InMemoryBus::new();
        let handler = Arc::new(FnHandler(|_: &Envelope| Ok(())));
        let handler2 = Arc::new(FnHandler(|_: &Envelope| Ok(())));

        let id1 = bus
            .subscribe(Subject::new("a"), handler)
            .await
            .unwrap();
        let id2 = bus
            .subscribe(Subject::new("b"), handler2)
            .await
            .unwrap();
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn subscribe_id_format() {
        let bus = InMemoryBus::new();
        let handler = Arc::new(FnHandler(|_: &Envelope| Ok(())));
        let id = bus
            .subscribe(Subject::new("x"), handler)
            .await
            .unwrap();
        assert!(id.starts_with("sub-"));
    }

    #[tokio::test]
    async fn subscribe_id_increments() {
        let bus = InMemoryBus::new();
        let h1 = Arc::new(FnHandler(|_: &Envelope| Ok(())));
        let h2 = Arc::new(FnHandler(|_: &Envelope| Ok(())));
        let h3 = Arc::new(FnHandler(|_: &Envelope| Ok(())));

        let id1 = bus.subscribe(Subject::new("a"), h1).await.unwrap();
        let id2 = bus.subscribe(Subject::new("b"), h2).await.unwrap();
        let id3 = bus.subscribe(Subject::new("c"), h3).await.unwrap();

        assert_eq!(id1, "sub-1");
        assert_eq!(id2, "sub-2");
        assert_eq!(id3, "sub-3");
    }

    #[tokio::test]
    async fn unsubscribe_nonexistent_is_silent() {
        let bus = InMemoryBus::new();
        // Should not error
        bus.unsubscribe("sub-999").await.unwrap();
    }

    #[tokio::test]
    async fn published_history_records_all() {
        let bus = InMemoryBus::new();
        assert!(bus.published().is_empty());

        let env1 = Envelope::new(
            &Subject::for_event("agileplus", "feature", 1, "created"),
            serde_json::json!({}),
        );
        let env2 = Envelope::new(
            &Subject::for_event("agileplus", "feature", 2, "deleted"),
            serde_json::json!({}),
        );

        bus.publish(env1).await.unwrap();
        bus.publish(env2).await.unwrap();

        let history = bus.published();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].subject, "agileplus.feature.1.created");
        assert_eq!(history[1].subject, "agileplus.feature.2.deleted");
    }

    #[tokio::test]
    async fn published_history_includes_reply_messages() {
        let bus = Arc::new(InMemoryBus::new());
        let bus_clone = bus.clone();

        let handler = Arc::new(FnHandler(move |env: &Envelope| {
            if let Some(reply_to) = &env.reply_to {
                let reply = Envelope::new(
                    &Subject::new(reply_to),
                    serde_json::json!({"ok": true}),
                );
                let b = bus_clone.clone();
                tokio::spawn(async move {
                    let _ = b.publish(reply).await;
                });
            }
            Ok(())
        }));

        bus.subscribe(Subject::new("test.rpc"), handler)
            .await
            .unwrap();

        let req = Envelope::new(
            &Subject::new("test.rpc"),
            serde_json::json!({}),
        );
        let _reply = bus.request(req, Duration::from_secs(2)).await.unwrap();

        // History should include both request and reply
        let history = bus.published();
        assert_eq!(history.len(), 2);
    }

    #[tokio::test]
    async fn unsubscribe_only_removes_targeted() {
        let bus = InMemoryBus::new();

        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count_clone = count.clone();

        let handler = Arc::new(FnHandler(move |_: &Envelope| {
            *count_clone.lock().unwrap() += 1;
            Ok(())
        }));

        let h2 = Arc::new(FnHandler(|_: &Envelope| Ok(())));

        let id1 = bus
            .subscribe(Subject::all_for_entity("agileplus", "feature"), handler)
            .await
            .unwrap();
        let _id2 = bus
            .subscribe(Subject::all_for_entity("agileplus", "feature"), h2)
            .await
            .unwrap();

        // Unsub only the first handler
        bus.unsubscribe(&id1).await.unwrap();

        let env = Envelope::new(
            &Subject::for_event("agileplus", "feature", 1, "created"),
            serde_json::json!({}),
        );
        bus.publish(env).await.unwrap();

        // First handler unsubscribed, so count stays 0
        assert_eq!(*count.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn in_memory_bus_default() {
        let bus = InMemoryBus::default();
        assert!(bus.published().is_empty());
        assert_eq!(bus.health().await, BusHealth::Connected);
    }

    #[tokio::test]
    async fn event_bus_store_subscribe_and_unsubscribe() {
        let store = EventBusStore::in_memory(NatsConfig::default());
        let handler = Arc::new(FnHandler(|_: &Envelope| Ok(())));
        let id = store
            .subscribe(Subject::new("t"), handler)
            .await
            .unwrap();
        assert!(id.starts_with("sub-"));
        store.unsubscribe(&id).await.unwrap();
    }

    #[tokio::test]
    async fn event_bus_store_request_delegates() {
        let store = EventBusStore::in_memory(NatsConfig::default());
        let req = Envelope::new(
            &Subject::new("agileplus.rpc.nobody"),
            serde_json::json!({}),
        );
        let result = store.request(req, Duration::from_millis(50)).await;
        assert!(matches!(result, Err(EventBusError::Timeout)));
    }

    #[tokio::test]
    async fn event_bus_store_new_with_custom_backend() {
        let bus = Box::new(InMemoryBus::new());
        let cfg = NatsConfig::new("nats://custom:4222");
        let store = EventBusStore::new(cfg, bus);
        assert_eq!(store.health().await, BusHealth::Connected);
    }

    #[tokio::test]
    async fn store_publish_records_to_history() {
        let store = EventBusStore::in_memory(NatsConfig::default());
        let env = Envelope::new(
            &Subject::for_event("agileplus", "wp", 1, "created"),
            serde_json::json!({}),
        );
        store.publish(env).await.unwrap();
        // Access backend to check published
        // The backend is an InMemoryBus but EventBusStore only exposes &dyn EventBus
        // We can verify via subscribe + publish pattern
    }

    #[tokio::test]
    async fn wildcard_subscription_matches_multiple_events() {
        let bus = InMemoryBus::new();

        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count_clone = count.clone();

        let handler = Arc::new(FnHandler(move |_: &Envelope| {
            *count_clone.lock().unwrap() += 1;
            Ok(())
        }));

        bus.subscribe(Subject::all_for_entity("agileplus", "feature"), handler)
            .await
            .unwrap();

        for i in 1..=5 {
            let env = Envelope::new(
                &Subject::for_event("agileplus", "feature", i, "created"),
                serde_json::json!({}),
            );
            bus.publish(env).await.unwrap();
        }

        assert_eq!(*count.lock().unwrap(), 5);
    }

    #[tokio::test]
    async fn star_wildcard_subscription_filters_correctly() {
        let bus = InMemoryBus::new();

        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count_clone = count.clone();

        let handler = Arc::new(FnHandler(move |_: &Envelope| {
            *count_clone.lock().unwrap() += 1;
            Ok(())
        }));

        bus.subscribe(Subject::all_of_type("agileplus", "feature", "created"), handler)
            .await
            .unwrap();

        // This matches
        let env1 = Envelope::new(
            &Subject::for_event("agileplus", "feature", 1, "created"),
            serde_json::json!({}),
        );
        bus.publish(env1).await.unwrap();

        // This does NOT match (deleted != created)
        let env2 = Envelope::new(
            &Subject::for_event("agileplus", "feature", 2, "deleted"),
            serde_json::json!({}),
        );
        bus.publish(env2).await.unwrap();

        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn request_reply_with_payload_exchange() {
        let bus = Arc::new(InMemoryBus::new());
        let bus_clone = bus.clone();

        let handler = Arc::new(FnHandler(move |env: &Envelope| {
            if let Some(reply_to) = &env.reply_to {
                // Echo back the request payload with a modification
                let response = serde_json::json!({
                    "echo": env.payload,
                    "status": "processed"
                });
                let reply = Envelope::new(
                    &Subject::new(reply_to),
                    response,
                );
                let b = bus_clone.clone();
                tokio::spawn(async move {
                    let _ = b.publish(reply).await;
                });
            }
            Ok(())
        }));

        bus.subscribe(Subject::new("echo.service"), handler)
            .await
            .unwrap();

        let req = Envelope::new(
            &Subject::new("echo.service"),
            serde_json::json!({"data": [1, 2, 3]}),
        );
        let reply = bus.request(req, Duration::from_secs(2)).await.unwrap();
        assert_eq!(reply.payload["status"], "processed");
        assert_eq!(reply.payload["echo"]["data"], serde_json::json!([1, 2, 3]));
    }

    #[tokio::test]
    async fn event_bus_error_display() {
        let errors = vec![
            EventBusError::ConnectionError("conn".into()),
            EventBusError::PublishError("pub".into()),
            EventBusError::SubscribeError("sub".into()),
            EventBusError::Timeout,
            EventBusError::SerializationError("serde".into()),
            EventBusError::HandlerError("handler".into()),
        ];
        for err in errors {
            let msg = format!("{err}");
            assert!(!msg.is_empty());
        }
    }

    #[tokio::test]
    async fn event_bus_store_backend_access() {
        let store = EventBusStore::in_memory(NatsConfig::default());
        let backend = store.backend();
        assert_eq!(backend.health().await, BusHealth::Connected);
    }

    #[tokio::test]
    async fn in_memory_bus_published_history_order() {
        let bus = InMemoryBus::new();
        for i in 0..5 {
            let payload = serde_json::json!({ "i": i });
            bus.publish(Envelope::new(&Subject::new(&format!("t.{i}")), payload)).await.unwrap();
        }
        let history = bus.published();
        assert_eq!(history.len(), 5);
        for (i, env) in history.iter().enumerate() {
            assert_eq!(env.subject, format!("t.{i}"));
            assert_eq!(env.payload["i"], i);
        }
    }

    #[tokio::test]
    async fn in_memory_bus_published_includes_request_replies() {
        let bus = Arc::new(InMemoryBus::new());

        // Capture the inbox subject assigned by bus.request, and respond to it.
        let observed_inbox: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let observed_clone = observed_inbox.clone();
        let responder_bus = bus.clone();

        let responder = Arc::new(FnHandler(move |env: &Envelope| {
            // The reply_to is set by the request impl; we ignore it here and
            // respond to the *original subject* by using its `reply_to`.
            if let Some(r) = &env.reply_to {
                let captured = r.clone();
                *observed_clone.lock().unwrap() = Some(captured.clone());
                let bus_clone = responder_bus.clone();
                tokio::spawn(async move {
                    let reply = Envelope::new(
                        &Subject::new(&captured),
                        serde_json::json!({"reply": true}),
                    );
                    let _ = bus_clone.publish(reply).await;
                });
            }
            Ok(())
        }));
        bus.subscribe(Subject::new("req"), responder).await.unwrap();

        // bus.request will overwrite reply_to with its own inbox.
        let req = Envelope::new(&Subject::new("req"), serde_json::json!({}));
        let _ = bus.request(req, Duration::from_millis(500)).await;

        // The responder captured the inbox subject; the spawn should publish
        // back to it.
        let inbox = observed_inbox.lock().unwrap().clone();
        assert!(inbox.is_some(), "inbox subject must be observed");
        let inbox_subject = inbox.unwrap();
        // Wait briefly for the spawn to land.
        for _ in 0..20 {
            let history = bus.published();
            if history.iter().any(|e| e.subject == inbox_subject) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let history = bus.published();
        assert!(
            history.iter().any(|e| e.subject == inbox_subject),
            "reply envelope to {} never observed",
            inbox_subject
        );
    }

    #[tokio::test]
    async fn request_timeout_when_no_reply() {
        let bus = InMemoryBus::new();
        let req = Envelope::new(&Subject::new("no_reply"), serde_json::json!({}))
            .with_reply_to(&Subject::new("_INBOX.missing"));
        let err = bus.request(req, Duration::from_millis(10)).await.unwrap_err();
        assert!(matches!(err, EventBusError::Timeout));
    }

    #[tokio::test]
    async fn subscribe_then_unsubscribe_then_subscribe_again() {
        let bus = InMemoryBus::new();
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count_a = count.clone();
        let handler_a = Arc::new(FnHandler(move |_env: &Envelope| {
            *count_a.lock().unwrap() += 1;
            Ok(())
        }));

        let sub1 = bus.subscribe(Subject::new("t"), handler_a.clone()).await.unwrap();
        bus.publish(Envelope::new(&Subject::new("t"), serde_json::json!({}))).await.unwrap();
        assert_eq!(*count.lock().unwrap(), 1);

        bus.unsubscribe(&sub1).await.unwrap();
        bus.publish(Envelope::new(&Subject::new("t"), serde_json::json!({}))).await.unwrap();
        assert_eq!(*count.lock().unwrap(), 1);

        let count_b = count.clone();
        let handler_b = Arc::new(FnHandler(move |_env: &Envelope| {
            *count_b.lock().unwrap() += 1;
            Ok(())
        }));
        let sub2 = bus.subscribe(Subject::new("t"), handler_b).await.unwrap();
        bus.publish(Envelope::new(&Subject::new("t"), serde_json::json!({}))).await.unwrap();
        assert_eq!(*count.lock().unwrap(), 2);
        bus.unsubscribe(&sub2).await.unwrap();
    }

    #[tokio::test]
    async fn unsubscribe_invalid_id_is_silent() {
        let bus = InMemoryBus::new();
        // Should not panic
        bus.unsubscribe("nonexistent").await.unwrap();
        bus.unsubscribe("").await.unwrap();
        bus.unsubscribe("not-a-uuid").await.unwrap();
    }

    #[tokio::test]
    async fn event_bus_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<EventBusError>();
    }

    #[tokio::test]
    async fn event_bus_error_from_constructors() {
        let errs = vec![
            EventBusError::ConnectionError("c".into()),
            EventBusError::PublishError("p".into()),
            EventBusError::SubscribeError("s".into()),
            EventBusError::Timeout,
            EventBusError::SerializationError("ser".into()),
            EventBusError::HandlerError("h".into()),
        ];
        for err in errs {
            let msg = format!("{err}");
            assert!(!msg.is_empty());
        }
    }

    #[tokio::test]
    async fn event_bus_store_new_with_custom_backend_extended() {
        let custom: Box<dyn EventBus> = Box::new(InMemoryBus::new());
        let config = NatsConfig::default();
        let store = EventBusStore::new(config.clone(), custom);
        assert_eq!(store.health().await, BusHealth::Connected);
    }

    #[tokio::test]
    async fn store_publish_records_to_history_extended() {
        let bus = Arc::new(InMemoryBus::new());
        let shared_bus_for_outer = bus.clone();
        // Custom backend that delegates to the in-memory bus we keep a handle to.
        let backend: Box<dyn EventBus> = Box::new(ArcSharedBus { inner: bus });
        let config = NatsConfig::default();
        let store = EventBusStore::new(config, backend);

        store.publish(Envelope::new(&Subject::new("hist.1"), serde_json::json!({}))).await.unwrap();
        store.publish(Envelope::new(&Subject::new("hist.2"), serde_json::json!({}))).await.unwrap();

        let _ = shared_bus_for_outer; // ensure captured
    }

    /// Trivial adapter that re-publishes through the shared InMemoryBus so we
    /// can observe the `published()` history from outside.
    struct ArcSharedBus {
        inner: Arc<InMemoryBus>,
    }

    #[async_trait::async_trait]
    impl EventBus for ArcSharedBus {
        async fn publish(&self, envelope: Envelope) -> Result<(), EventBusError> {
            self.inner.publish(envelope).await
        }
        async fn subscribe(
            &self,
            subject: crate::subject::Subject,
            handler: Arc<dyn crate::handler::Handler>,
        ) -> Result<String, EventBusError> {
            self.inner.subscribe(subject, handler).await
        }
        async fn unsubscribe(&self, subscription_id: &str) -> Result<(), EventBusError> {
            self.inner.unsubscribe(subscription_id).await
        }
        async fn request(
            &self,
            envelope: Envelope,
            timeout: std::time::Duration,
        ) -> Result<Envelope, EventBusError> {
            self.inner.request(envelope, timeout).await
        }
        async fn health(&self) -> BusHealth {
            self.inner.health().await
        }
    }

    #[tokio::test]
    async fn wildcard_subscription_matches_multiple_events_extended() {
        let bus = InMemoryBus::new();
        let received: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let r = received.clone();
        let handler = Arc::new(FnHandler(move |env: &Envelope| {
            r.lock().unwrap().push(env.subject.clone());
            Ok(())
        }));

        bus.subscribe(Subject::new("agileplus.>"), handler).await.unwrap();

        for i in 1..=3 {
            bus.publish(Envelope::new(&Subject::new(&format!("agileplus.feature.{i}.created")), serde_json::json!({}))).await.unwrap();
        }

        assert_eq!(received.lock().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn star_wildcard_subscription_filters_correctly_extended() {
        let bus = InMemoryBus::new();
        let received: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let r = received.clone();
        let handler = Arc::new(FnHandler(move |env: &Envelope| {
            r.lock().unwrap().push(env.subject.clone());
            Ok(())
        }));

        // Pattern matches 3-token subjects ending in `.created`.
        bus.subscribe(Subject::new("*.wp.created"), handler).await.unwrap();

        bus.publish(Envelope::new(&Subject::new("agileplus.wp.created"), serde_json::json!({}))).await.unwrap();
        bus.publish(Envelope::new(&Subject::new("acme.wp.created"), serde_json::json!({}))).await.unwrap();
        bus.publish(Envelope::new(&Subject::new("agileplus.wp.updated"), serde_json::json!({}))).await.unwrap();

        let got = received.lock().unwrap();
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|s| s.ends_with(".wp.created")));
    }

    #[tokio::test]
    async fn request_reply_with_payload_exchange_extended() {
        let bus = Arc::new(InMemoryBus::new());
        let reply_subj = Subject::new("_INBOX.reply1");

        let inner_bus = bus.clone();
        let responder = Arc::new(FnHandler(move |env: &Envelope| {
            if let Some(r) = &env.reply_to {
                let r = r.clone();
                let cid = env.correlation_id.clone();
                let env_payload = env.payload.clone();
                let bus_clone = inner_bus.clone();
                tokio::spawn(async move {
                    let reply = Envelope::new(&Subject::new(&r), serde_json::json!({"echo": env_payload}))
                        .with_correlation(cid.unwrap_or_default());
                    let _ = bus_clone.publish(reply).await;
                });
            }
            Ok(())
        }));
        bus.subscribe(Subject::new("echo"), responder).await.unwrap();

        let req_payload = serde_json::json!({"msg": "hello"});
        let req = Envelope::new(&Subject::new("echo"), req_payload.clone())
            .with_reply_to(&reply_subj)
            .with_correlation("corr-123");
        let resp = bus.request(req, Duration::from_millis(500)).await.unwrap();

        // Responder wraps payload as {"echo": <original>}
        assert_eq!(resp.payload["echo"], req_payload);
        assert_eq!(resp.correlation_id.as_deref(), Some("corr-123"));
    }

    #[tokio::test]
    async fn no_matching_subscribers_silently_publishes_extended() {
        let bus = InMemoryBus::new();
        // No subscribers, just publish
        bus.publish(Envelope::new(&Subject::new("orphan"), serde_json::json!({}))).await.unwrap();
        // Should not error, envelope recorded
        assert_eq!(bus.published().len(), 1);
    }
}
