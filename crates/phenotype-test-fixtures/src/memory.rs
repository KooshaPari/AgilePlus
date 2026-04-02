//! Generic in-memory store implementations for testing.
//!
//! This module provides thread-safe, generic in-memory stores that can be used
//! across multiple crates for testing purposes, eliminating duplicate implementations.
//!
//! Traces to: FR-TESTFIXTURES-MEMORY-001

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

/// A generic thread-safe in-memory key-value store.
///
/// This store provides basic CRUD operations and is suitable for testing
/// repositories, caches, and event stores without external dependencies.
///
/// Traces to: FR-TESTFIXTURES-MEMORY-002
#[derive(Debug, Clone)]
pub struct InMemoryStore<K, V> {
    data: Arc<RwLock<HashMap<K, V>>>,
}

impl<K, V> Default for InMemoryStore<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> InMemoryStore<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new empty in-memory store.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-003
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new store with initial capacity.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-004
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::with_capacity(capacity))),
        }
    }

    /// Insert a key-value pair into the store.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-005
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let mut data = self.data.write().unwrap();
        data.insert(key, value)
    }

    /// Get a value by key.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-006
    pub fn get(&self, key: &K) -> Option<V> {
        let data = self.data.read().unwrap();
        data.get(key).cloned()
    }

    /// Remove a key-value pair from the store.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-007
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().unwrap();
        data.remove(key)
    }

    /// Check if the store contains a key.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-008
    pub fn contains_key(&self, key: &K) -> bool {
        let data = self.data.read().unwrap();
        data.contains_key(key)
    }

    /// Get the number of entries in the store.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-009
    pub fn len(&self) -> usize {
        let data = self.data.read().unwrap();
        data.len()
    }

    /// Check if the store is empty.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-010
    pub fn is_empty(&self) -> bool {
        let data = self.data.read().unwrap();
        data.is_empty()
    }

    /// Clear all entries from the store.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-011
    pub fn clear(&self) {
        let mut data = self.data.write().unwrap();
        data.clear();
    }

    /// Get all keys in the store.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-012
    pub fn keys(&self) -> Vec<K> {
        let data = self.data.read().unwrap();
        data.keys().cloned().collect()
    }

    /// Get all values in the store.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-013
    pub fn values(&self) -> Vec<V> {
        let data = self.data.read().unwrap();
        data.values().cloned().collect()
    }

    /// Get all entries as a vector of tuples.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-014
    pub fn entries(&self) -> Vec<(K, V)> {
        let data = self.data.read().unwrap();
        data.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Update a value if the key exists, returning the old value.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-015
    pub fn update<F>(&self, key: &K, f: F) -> Option<V>
    where
        F: FnOnce(&mut V),
    {
        let mut data = self.data.write().unwrap();
        if let Some(value) = data.get_mut(key) {
            f(value);
            Some(value.clone())
        } else {
            None
        }
    }
}

/// An append-only in-memory event store for testing.
///
/// This is a specialized store for event sourcing patterns that maintains
/// events in insertion order per entity.
///
/// Traces to: FR-TESTFIXTURES-MEMORY-016
#[derive(Debug, Clone)]
pub struct InMemoryEventStore<E> {
    events: Arc<RwLock<HashMap<String, Vec<E>>>>,
}

impl<E: Clone> Default for InMemoryEventStore<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Clone> InMemoryEventStore<E> {
    /// Create a new empty event store.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-017
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Append an event for a specific entity.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-018
    pub fn append(&self, entity_id: &str, event: E) {
        let mut events = self.events.write().unwrap();
        events
            .entry(entity_id.to_string())
            .or_default()
            .push(event);
    }

    /// Get all events for a specific entity.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-019
    pub fn get_events(&self, entity_id: &str) -> Vec<E> {
        let events = self.events.read().unwrap();
        events
            .get(entity_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get the number of events for a specific entity.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-020
    pub fn event_count(&self, entity_id: &str) -> usize {
        let events = self.events.read().unwrap();
        events.get(entity_id).map(|v| v.len()).unwrap_or(0)
    }

    /// Get total event count across all entities.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-021
    pub fn total_event_count(&self) -> usize {
        let events = self.events.read().unwrap();
        events.values().map(|v| v.len()).sum()
    }

    /// Clear all events for a specific entity.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-022
    pub fn clear_entity(&self, entity_id: &str) {
        let mut events = self.events.write().unwrap();
        events.remove(entity_id);
    }

    /// Clear all events from the store.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-023
    pub fn clear(&self) {
        let mut events = self.events.write().unwrap();
        events.clear();
    }

    /// Get all entity IDs in the store.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-024
    pub fn entity_ids(&self) -> Vec<String> {
        let events = self.events.read().unwrap();
        events.keys().cloned().collect()
    }
}

/// A typed configuration store for testing.
///
/// Provides type-safe configuration storage with default value support.
///
/// Traces to: FR-TESTFIXTURES-MEMORY-025
#[derive(Debug, Clone)]
pub struct InMemoryConfigStore {
    values: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for InMemoryConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryConfigStore {
    /// Create a new empty config store.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-026
    pub fn new() -> Self {
        Self {
            values: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set a configuration value.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-027
    pub fn set(&self, key: &str, value: impl Into<String>) {
        let mut values = self.values.write().unwrap();
        values.insert(key.to_string(), value.into());
    }

    /// Get a configuration value.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-028
    pub fn get(&self, key: &str) -> Option<String> {
        let values = self.values.read().unwrap();
        values.get(key).cloned()
    }

    /// Get a value with a default if not found.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-029
    pub fn get_or_default(&self, key: &str, default: impl Into<String>) -> String {
        self.get(key).unwrap_or_else(|| default.into())
    }

    /// Check if a key exists.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-030
    pub fn has(&self, key: &str) -> bool {
        let values = self.values.read().unwrap();
        values.contains_key(key)
    }

    /// Load configuration from a string (TOML format).
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-031
    pub fn load_toml(&self, content: &str) -> Result<(), String> {
        let values: HashMap<String, String> = toml::from_str(content)
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;
        
        let mut store = self.values.write().unwrap();
        for (k, v) in values {
            store.insert(k, v);
        }
        Ok(())
    }

    /// Export configuration to TOML string.
    ///
    /// Traces to: FR-TESTFIXTURES-MEMORY-032
    pub fn to_toml(&self) -> Result<String, String> {
        let values = self.values.read().unwrap();
        toml::to_string_pretty(&*values)
            .map_err(|e| format!("Failed to serialize: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-TESTFIXTURES-MEMORY-033
    #[test]
    fn test_inmemory_store_basic_operations() {
        let store: InMemoryStore<String, i32> = InMemoryStore::new();

        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        store.insert("key1".to_string(), 100);
        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);

        assert_eq!(store.get(&"key1".to_string()), Some(100));
        assert_eq!(store.get(&"nonexistent".to_string()), None);

        store.remove(&"key1".to_string());
        assert!(store.is_empty());
    }

    // Traces to: FR-TESTFIXTURES-MEMORY-034
    #[test]
    fn test_inmemory_store_thread_safety() {
        use std::thread;

        let store: InMemoryStore<u64, String> = InMemoryStore::new();
        let store_ref = Arc::new(store);

        let mut handles = vec![];

        // Spawn multiple threads writing
        for i in 0..10 {
            let store_clone = store_ref.clone();
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    store_clone.insert(i * 100 + j, format!("value-{}", j));
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(store_ref.len(), 100);
    }

    // Traces to: FR-TESTFIXTURES-MEMORY-035
    #[test]
    fn test_inmemory_event_store() {
        let store: InMemoryEventStore<String> = InMemoryEventStore::new();

        store.append("entity1", "event1".to_string());
        store.append("entity1", "event2".to_string());
        store.append("entity2", "event3".to_string());

        assert_eq!(store.get_events("entity1").len(), 2);
        assert_eq!(store.get_events("entity2").len(), 1);
        assert_eq!(store.total_event_count(), 3);

        store.clear_entity("entity1");
        assert_eq!(store.get_events("entity1").len(), 0);
        assert_eq!(store.total_event_count(), 1);
    }

    // Traces to: FR-TESTFIXTURES-MEMORY-036
    #[test]
    fn test_inmemory_config_store() {
        let config = InMemoryConfigStore::new();

        config.set("key1", "value1");
        assert_eq!(config.get("key1"), Some("value1".to_string()));

        assert_eq!(
            config.get_or_default("key2", "default"),
            "default".to_string()
        );

        assert!(config.has("key1"));
        assert!(!config.has("key2"));
    }

    // Traces to: FR-TESTFIXTURES-MEMORY-037
    #[test]
    fn test_inmemory_store_update() {
        let store: InMemoryStore<String, i32> = InMemoryStore::new();

        store.insert("key1".to_string(), 100);

        let updated = store.update(&"key1".to_string(), |v| *v += 50);
        assert_eq!(updated, Some(150));
        assert_eq!(store.get(&"key1".to_string()), Some(150));

        let not_found = store.update(&"nonexistent".to_string(), |v| *v += 1);
        assert_eq!(not_found, None);
    }

    // Traces to: FR-TESTFIXTURES-MEMORY-038
    #[test]
    fn test_inmemory_store_entries() {
        let store: InMemoryStore<String, i32> = InMemoryStore::new();

        store.insert("a".to_string(), 1);
        store.insert("b".to_string(), 2);
        store.insert("c".to_string(), 3);

        let entries = store.entries();
        assert_eq!(entries.len(), 3);

        let keys = store.keys();
        assert_eq!(keys.len(), 3);

        let values = store.values();
        assert_eq!(values.len(), 3);
    }
}
