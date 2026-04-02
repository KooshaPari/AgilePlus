//! EventStore trait — async append-only event storage.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use agileplus_domain::domain::event::Event;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event not found: {0}")]
    NotFound(String),
    #[error("Duplicate sequence: {0}")]
    DuplicateSequence(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Invalid hash: {0}")]
    InvalidHash(String),
    #[error("Sequence gap: expected {expected}, got {actual}")]
    SequenceGap { expected: i64, actual: i64 },
}

#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, event: &Event) -> Result<i64, EventError>;
    async fn get_events(&self, entity_type: &str, entity_id: i64) -> Result<Vec<Event>, EventError>;
    async fn get_events_since(
        &self,
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
    ) -> Result<Vec<Event>, EventError>;
    async fn get_events_by_range(
        &self,
        entity_type: &str,
        entity_id: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Event>, EventError>;
    async fn get_latest_sequence(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<i64, EventError>;
}

pub struct InMemoryEventStore {
    events: Arc<Mutex<HashMap<(String, i64), Vec<Event>>>>,
    sequences: Arc<Mutex<HashMap<(String, i64), i64>>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(HashMap::new())),
            sequences: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append(&self, event: &Event) -> Result<i64, EventError> {
        let key = (event.entity_type.clone(), event.entity_id);
        let mut events = self.events.lock().unwrap();
        let mut seq_map = self.sequences.lock().unwrap();
        let next_seq = seq_map.entry(key.clone()).or_insert(0);
        let assigned = *next_seq;
        *next_seq += 1;
        let entry = events.entry(key).or_insert_with(Vec::new);
        let mut e = event.clone();
        e.sequence = assigned;
        entry.push(e);
        Ok(assigned)
    }

    async fn get_events(&self, entity_type: &str, entity_id: i64) -> Result<Vec<Event>, EventError> {
        let key = (entity_type.to_string(), entity_id);
        let events = self.events.lock().unwrap();
        Ok(events.get(&key).cloned().unwrap_or_default())
    }

    async fn get_events_since(
        &self,
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
    ) -> Result<Vec<Event>, EventError> {
        let key = (entity_type.to_string(), entity_id);
        let events = self.events.lock().unwrap();
        let all = events.get(&key).cloned().unwrap_or_default();
        Ok(all.into_iter().filter(|e| e.sequence > sequence).collect())
    }

    async fn get_events_by_range(
        &self,
        entity_type: &str,
        entity_id: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Event>, EventError> {
        let key = (entity_type.to_string(), entity_id);
        let events = self.events.lock().unwrap();
        let all = events.get(&key).cloned().unwrap_or_default();
        Ok(all
            .into_iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .collect())
    }

    async fn get_latest_sequence(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<i64, EventError> {
        let key = (entity_type.to_string(), entity_id);
        let seq_map = self.sequences.lock().unwrap();
        Ok(seq_map.get(&key).copied().unwrap_or(0))
    }
}
