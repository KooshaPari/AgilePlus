//! Shared test infrastructure for `agileplus-cache` integration tests.
//!
//! Provides a deterministic in-memory `CacheStore` implementation with TTL
//! expiry, LRU capacity eviction, and injectable failure modes. This lets the
//! store contract be exercised without a live Dragonfly/Redis instance.
#![allow(dead_code)]

use agileplus_cache::store::{CacheError, CacheStore};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

struct Entry {
    json: String,
    expires_at: Option<Instant>,
    last_tick: u64,
}

/// Injectable failure messages, one per operation.
#[derive(Debug, Default, Clone)]
pub struct FailureMode {
    pub get: Option<String>,
    pub set: Option<String>,
    pub delete: Option<String>,
    pub exists: Option<String>,
}

/// In-memory `CacheStore` with TTL, LRU eviction, and failure injection.
pub struct InMemoryCacheStore {
    entries: Mutex<HashMap<String, Entry>>,
    capacity: usize,
    tick: AtomicU64,
    default_ttl: Option<Duration>,
    failures: Mutex<FailureMode>,
}

impl InMemoryCacheStore {
    /// Create a store with unbounded capacity and no default TTL.
    pub fn unbounded() -> Self {
        Self::with_capacity(usize::MAX)
    }

    /// Create a store with the given LRU capacity and no default TTL.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            capacity,
            tick: AtomicU64::new(0),
            default_ttl: None,
            failures: Mutex::new(FailureMode::default()),
        }
    }

    /// Apply a default TTL used when `set` is called with `ttl = None`.
    pub fn with_default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = Some(ttl);
        self
    }

    /// Inject a failure for one operation kind.
    pub fn fail(&self, mode: FailureMode) {
        *self.failures.lock().expect("failures lock") = mode;
    }

    /// Clear all injected failures.
    pub fn clear_failures(&self) {
        *self.failures.lock().expect("failures lock") = FailureMode::default();
    }

    /// Number of live (non-expired) entries.
    pub fn live_len(&self) -> usize {
        let mut entries = self.entries.lock().expect("entries lock");
        let now = Instant::now();
        entries.retain(|_, entry| {
            entry
                .expires_at
                .map(|deadline| deadline > now)
                .unwrap_or(true)
        });
        entries.len()
    }

    /// True if the raw entry exists, regardless of expiry.
    pub fn raw_contains(&self, key: &str) -> bool {
        self.entries.lock().expect("entries lock").contains_key(key)
    }

    fn next_tick(&self) -> u64 {
        self.tick.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn failure(kind: FailureKind) -> CacheError {
        CacheError::RedisError(format!("injected {kind} failure"))
    }
}

#[derive(Debug, Clone, Copy)]
enum FailureKind {
    Get,
    Set,
    Delete,
    Exists,
}

impl std::fmt::Display for FailureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            FailureKind::Get => "get",
            FailureKind::Set => "set",
            FailureKind::Delete => "delete",
            FailureKind::Exists => "exists",
        };
        write!(f, "{label}")
    }
}

#[async_trait]
impl CacheStore for InMemoryCacheStore {
    async fn get<T: for<'de> Deserialize<'de> + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError> {
        if let Some(message) = self.failures.lock().expect("failures lock").get.clone() {
            return Err(CacheError::RedisError(message));
        }

        let now = Instant::now();
        let tick = self.next_tick();
        let mut entries = self.entries.lock().expect("entries lock");

        let entry = match entries.get_mut(key) {
            Some(entry) => entry,
            None => return Ok(None),
        };

        if entry
            .expires_at
            .map(|deadline| deadline <= now)
            .unwrap_or(false)
        {
            entries.remove(key);
            return Ok(None);
        }

        entry.last_tick = tick;
        serde_json::from_str(&entry.json)
            .map(Some)
            .map_err(|e| CacheError::SerializationError(e.to_string()))
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        if let Some(message) = self.failures.lock().expect("failures lock").set.clone() {
            return Err(CacheError::RedisError(message));
        }

        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let effective_ttl = ttl.or(self.default_ttl);
        let expires_at = effective_ttl.map(|ttl| Instant::now() + ttl);
        let tick = self.next_tick();

        let mut entries = self.entries.lock().expect("entries lock");
        entries.insert(
            key.to_string(),
            Entry {
                json: serialized,
                expires_at,
                last_tick: tick,
            },
        );

        // LRU eviction: drop the least recently touched entries first.
        while entries.len() > self.capacity {
            if let Some(victim) = entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_tick)
                .map(|(key, _)| key.clone())
            {
                entries.remove(&victim);
            } else {
                break;
            }
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        if let Some(message) = self.failures.lock().expect("failures lock").delete.clone() {
            return Err(CacheError::RedisError(message));
        }
        self.entries.lock().expect("entries lock").remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        if let Some(message) = self.failures.lock().expect("failures lock").exists.clone() {
            return Err(CacheError::RedisError(message));
        }

        let now = Instant::now();
        let mut entries = self.entries.lock().expect("entries lock");
        match entries.get(key) {
            Some(entry) if entry.expires_at.map(|d| d <= now).unwrap_or(false) => {
                entries.remove(key);
                Ok(false)
            }
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
}

/// A value whose `Serialize` impl always fails, for error-path tests.
pub struct FailsToSerialize;

impl Serialize for FailsToSerialize {
    fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(
            "intentional serialization failure",
        ))
    }
}

/// A small serializable struct used to exercise typed round-trips.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub id: i64,
    pub name: String,
    pub tags: Vec<String>,
}

impl Record {
    pub fn sample(id: i64) -> Self {
        Self {
            id,
            name: format!("record-{id}"),
            tags: vec!["alpha".to_string(), "beta".to_string()],
        }
    }
}
