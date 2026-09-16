//! NATS-style event bus for AgilePlus.
//!
//! Provides a trait-based event bus abstraction with typed domain events,
//! hierarchical subject routing, and request/reply semantics. The default
//! implementation is an in-memory bus suitable for testing; a NATS-backed
//! implementation can be plugged in when the real broker is available.
//!
//! Traceability: WP06 — Event Bus

pub mod bus;
pub mod config;
pub mod envelope;
pub mod handler;
pub mod health;
pub mod subject;

pub use bus::{EventBus, EventBusError, EventBusStore, InMemoryBus};
pub use config::NatsConfig;
pub use envelope::Envelope;
pub use handler::Handler;
pub use health::BusHealth;
pub use subject::Subject;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Event bus error: {0}")]
    Bus(#[from] EventBusError),
    #[error("Config error: {0}")]
    Config(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_from_event_bus_error() {
        let bus_err = EventBusError::ConnectionError("refused".into());
        let err: Error = bus_err.into();
        let msg = format!("{err}");
        assert!(msg.contains("Event bus error"));
        assert!(msg.contains("refused"));
    }

    #[test]
    fn error_config_variant() {
        let err = Error::Config("bad url".into());
        let msg = format!("{err}");
        assert!(msg.contains("Config error"));
        assert!(msg.contains("bad url"));
    }

    #[test]
    fn error_bus_from_timeout() {
        let err: Error = EventBusError::Timeout.into();
        let msg = format!("{err}");
        assert!(msg.contains("Request timeout"));
    }

    #[test]
    fn error_bus_from_publish() {
        let err: Error = EventBusError::PublishError("queue full".into()).into();
        let msg = format!("{err}");
        assert!(msg.contains("Publish error"));
        assert!(msg.contains("queue full"));
    }

    #[test]
    fn error_bus_from_subscribe() {
        let err: Error =
            EventBusError::SubscribeError("no topic".into()).into();
        let msg = format!("{err}");
        assert!(msg.contains("Subscribe error"));
    }

    #[test]
    fn error_bus_from_serialization() {
        let err: Error =
            EventBusError::SerializationError("bad json".into()).into();
        let msg = format!("{err}");
        assert!(msg.contains("Serialization error"));
    }

    #[test]
    fn error_bus_from_handler() {
        let err: Error =
            EventBusError::HandlerError("crash".into()).into();
        let msg = format!("{err}");
        assert!(msg.contains("Handler error"));
    }

    #[test]
    fn error_debug_impl() {
        let err = Error::Config("x".into());
        let debug = format!("{err:?}");
        assert!(debug.contains("Config"));
    }

    #[test]
    fn re_exports_are_accessible() {
        // Verify all re-exported types are usable from crate root.
        let _cfg = NatsConfig::default();
        let _subject = Subject::new("test");
        let _health = BusHealth::Connected;
        let _bus = InMemoryBus::new();
        let _env = Envelope::new(
            &Subject::new("test"),
            serde_json::json!({}),
        );
    }
}
