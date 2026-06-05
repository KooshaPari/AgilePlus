//! agileplus-nats — NATS infrastructure adapter for AgilePlus.
//!
//! Provides:
//! - `NatsEventBus`: hexagonal adapter implementing `agileplus_events::AsyncEventBus`
//!   and `agileplus_events::EventBus` ports over a live async-nats connection.
//! - `InMemoryBus`: in-process bus for unit/integration tests (no broker required).
//! - `Subject`: validated dot-separated NATS subject addressing with wildcard helpers.
//!
//! Subject scheme: `agileplus.events.<AggregateType>.<event_type>`
//! e.g. `agileplus.events.Project.project.created`
//!
//! Traceability: FR-008 / WP02 (infrastructure layer)

pub mod bus;
pub mod config;
pub mod envelope;
pub mod handler;
pub mod health;
pub mod nats_adapter;
pub mod subject;

pub use bus::{EventBus, EventBusError, EventBusStore, InMemoryBus};
pub use config::NatsConfig;
pub use envelope::Envelope;
pub use handler::{FnHandler, Handler};
pub use health::BusHealth;
pub use nats_adapter::{derive_subject, NatsEventBus, NatsEventBusError};
pub use subject::Subject;
