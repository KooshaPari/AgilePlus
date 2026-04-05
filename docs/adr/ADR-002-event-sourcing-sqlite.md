# ADR-002: Event Sourcing with SQLite for Audit Trail

## Status
**Accepted** — Implemented in `agileplus-sqlite` and `agileplus-events`

## Context

AgilePlus requires a tamper-evident audit trail for:
1. **Compliance requirements** — SOC 2, ISO 27001, future FedRAMP
2. **Agent accountability** — Track what AI agents did and when
3. **Governance enforcement** — Prove state transitions followed rules
4. **Debugging** — Reconstruct state at any point in time

We evaluated multiple approaches for maintaining this audit trail.

### Alternatives Considered

| Approach | Pros | Cons | Verdict |
|----------|------|------|---------|
| **Standard CRUD + Audit Log Table** | Simple, familiar | Separate from state, can drift | ❌ Rejected |
| **Write-Ahead Log (WAL) Scraping** | Low overhead | Implementation-specific, opaque | ❌ Rejected |
| **Application-Level Logging** | Flexible | Not tamper-evident, can be skipped | ❌ Rejected |
| **Full Event Sourcing (CQRS)** | Complete history | Complex, learning curve | ⚠️ Partial |
| **Event Sourcing Lite** | Audit + state reconstruction | Balance of both worlds | ✅ Selected |

## Decision

We will implement **Event Sourcing Lite** — a hybrid approach:

1. **Events are source of truth** for audit trail (append-only, immutable)
2. **Current state is materialized** from events for fast reads
3. **Hash chain links events** cryptographically (SHA-256)
4. **SQLite stores events** (simple, fast, local-first)
5. **Snapshots optimize reads** (rebuild state from events periodically)

### Event Store Schema

```sql
-- Events table: append-only, immutable
CREATE TABLE events (
    id TEXT PRIMARY KEY,              -- ULID
    stream_id TEXT NOT NULL,          -- aggregate identifier (feature-123)
    stream_type TEXT NOT NULL,        -- aggregate type (Feature, WorkPackage)
    version INTEGER NOT NULL,         -- sequence within stream
    event_type TEXT NOT NULL,         -- FeatureCreated, FeatureStateChanged
    payload BLOB NOT NULL,            -- MessagePack encoded
    metadata BLOB,                    -- JSON context
    occurred_at INTEGER NOT NULL,     -- Unix timestamp (nanoseconds)
    actor_id TEXT,                    -- Who triggered (user, agent, system)
    actor_type TEXT NOT NULL,         -- human, agent, system
    hash_chain TEXT NOT NULL,         -- SHA-256 of (prev_hash + payload)
    prev_hash TEXT,                   -- Previous event hash (NULL for first)
    
    UNIQUE(stream_id, version)
);

-- Index for fast queries
CREATE INDEX idx_events_stream ON events(stream_id, version);
CREATE INDEX idx_events_type ON events(event_type, occurred_at DESC);
CREATE INDEX idx_events_actor ON events(actor_id, occurred_at DESC);
CREATE INDEX idx_events_hash ON events(hash_chain);

-- Snapshots table: for fast reads
CREATE TABLE snapshots (
    id TEXT PRIMARY KEY,              -- ULID
    stream_id TEXT NOT NULL,          -- aggregate identifier
    stream_type TEXT NOT NULL,        -- aggregate type
    version INTEGER NOT NULL,         -- event version at snapshot
    state BLOB NOT NULL,              -- MessagePack encoded aggregate state
    created_at INTEGER NOT NULL,      -- Unix timestamp
    
    UNIQUE(stream_id, version)
);

-- Index for snapshot retrieval
CREATE INDEX idx_snapshots_stream ON snapshots(stream_id, version DESC);
```

### Hash Chain Algorithm

```rust
/// Calculate hash for event linking
fn calculate_hash(prev_hash: Option<&str>, payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    
    // Include previous hash if exists
    if let Some(prev) = prev_hash {
        hasher.update(prev.as_bytes());
    }
    
    // Include payload
    hasher.update(payload);
    
    // Return hex-encoded hash
    hex::encode(hasher.finalize())
}

/// Verify chain integrity
fn verify_chain(events: &[Event]) -> Result<(), IntegrityError> {
    for (i, event) in events.iter().enumerate() {
        let expected_hash = if i == 0 {
            // First event: hash of payload only
            calculate_hash(None, &event.payload)
        } else {
            // Subsequent: hash of prev_hash + payload
            calculate_hash(Some(&events[i-1].hash_chain), &event.payload)
        };
        
        if event.hash_chain != expected_hash {
            return Err(IntegrityError::BrokenChain {
                at_event: event.id.clone(),
                expected: expected_hash,
                actual: event.hash_chain.clone(),
            });
        }
    }
    Ok(())
}
```

### Event Types

```rust
/// Domain events for Feature aggregate
pub enum FeatureEvent {
    Created {
        id: FeatureId,
        title: String,
        description: Option<String>,
        created_at: Timestamp,
        actor: Actor,
    },
    
    StateChanged {
        from: FeatureState,
        to: FeatureState,
        triggered_by: StateTransitionTrigger,
        evidence_ids: Vec<EvidenceId>,
        occurred_at: Timestamp,
        actor: Actor,
    },
    
    WorkPackageAdded {
        wp_id: WorkPackageId,
        title: String,
        effort_estimate: Option<StoryPoints>,
        file_scope: Vec<FilePath>,
        occurred_at: Timestamp,
        actor: Actor,
    },
    
    SpecAttached {
        spec_hash: Sha256Hash,
        spec_path: PathBuf,
        occurred_at: Timestamp,
        actor: Actor,
    },
    
    EvidenceAttached {
        evidence_id: EvidenceId,
        evidence_type: EvidenceType,
        content_hash: Sha256Hash,
        occurred_at: Timestamp,
        actor: Actor,
    },
    
    Cancelled {
        reason: String,
        occurred_at: Timestamp,
        actor: Actor,
    },
}

/// Domain events for WorkPackage aggregate
pub enum WorkPackageEvent {
    Created {
        id: WorkPackageId,
        feature_id: FeatureId,
        title: String,
        description: Option<String>,
        created_at: Timestamp,
        actor: Actor,
    },
    
    StateChanged {
        from: WorkPackageState,
        to: WorkPackageState,
        occurred_at: Timestamp,
        actor: Actor,
    },
    
    Assigned {
        agent_id: AgentId,
        assigned_at: Timestamp,
        actor: Actor,
    },
    
    AgentReported {
        agent_id: AgentId,
        status: AgentWorkStatus,
        files_modified: Vec<FilePath>,
        tests_status: TestsStatus,
        occurred_at: Timestamp,
    },
    
    Completed {
        completed_at: Timestamp,
        actor: Actor,
    },
}
```

### Snapshot Strategy

```rust
/// Snapshot configuration
pub struct SnapshotConfig {
    /// Create snapshot every N events
    pub events_per_snapshot: usize,
    
    /// Maximum age before forcing snapshot
    pub max_age_seconds: u64,
    
    /// Compress snapshots
    pub compression: CompressionAlgorithm,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            events_per_snapshot: 100,
            max_age_seconds: 86400, // 24 hours
            compression: CompressionAlgorithm::Zstd,
        }
    }
}

/// Snapshot creation logic
pub async fn maybe_create_snapshot(
    event_store: &dyn EventStore,
    stream_id: &str,
    config: &SnapshotConfig,
) -> Result<Option<Snapshot>, EventStoreError> {
    let events = event_store.read(stream_id, 0).await?;
    
    // Check if we need a snapshot
    let should_snapshot = if let Some(latest) = event_store.get_snapshot(stream_id).await? {
        let events_since = events.len() - latest.version as usize;
        let age_seconds = now() - latest.created_at;
        
        events_since >= config.events_per_snapshot 
            || age_seconds > config.max_age_seconds
    } else {
        // No snapshot exists, create first one
        events.len() >= config.events_per_snapshot / 2
    };
    
    if should_snapshot {
        // Rebuild aggregate from events
        let aggregate = rebuild_aggregate(&events)?;
        
        let snapshot = Snapshot {
            id: Ulid::new().to_string(),
            stream_id: stream_id.to_string(),
            stream_type: aggregate.type_name(),
            version: events.len() as u64,
            state: serialize_with_compression(&aggregate, config.compression)?,
            created_at: now(),
        };
        
        event_store.save_snapshot(&snapshot).await?;
        return Ok(Some(snapshot));
    }
    
    Ok(None)
}
```

## Consequences

### Positive

1. **Tamper Evidence**: Hash chain makes undetected modification impossible
2. **Complete History**: Reconstruct state at any point in time
3. **Audit Compliance**: Exceeds SOC 2 / ISO 27001 requirements
4. **Debugging**: Replay events to reproduce bugs
5. **Temporal Queries**: "What was the state on March 1st?"

### Negative

1. **Storage Growth**: Events append-only, storage grows linearly
2. **Read Performance**: Must replay events or use snapshots
3. **Complexity**: Developers must understand event sourcing concepts
4. **Migration**: Schema changes require event upcasting

### Mitigations

| Concern | Mitigation |
|---------|------------|
| Storage | Archive events >1 year to cold storage; compress snapshots |
| Read Perf | Snapshots every 100 events; in-memory caching |
| Complexity | Repository pattern hides event sourcing from domain |
| Migration | Versioned events with upcasters; schema evolution tests |

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Event append | < 5ms | SQLite INSERT + hash calc |
| Event read (100) | < 10ms | Query + deserialize |
| Snapshot read | < 2ms | Single row fetch |
| Chain verification | < 50ms for 1000 events | Batch verification |
| Storage per event | ~200 bytes | MessagePack + metadata |

## Implementation

### Event Store Adapter

```rust
pub struct SqliteEventStore {
    pool: SqlitePool,
    config: EventStoreConfig,
}

#[async_trait]
impl EventStore for SqliteEventStore {
    async fn append(
        &self,
        stream_id: &str,
        events: &[DomainEvent],
    ) -> Result<u64, EventStoreError> {
        let mut tx = self.pool.begin().await?;
        
        // Get current version
        let current_version: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM events WHERE stream_id = ?"
        )
        .bind(stream_id)
        .fetch_one(&mut *tx)
        .await?;
        
        // Get last hash for chain
        let last_hash: Option<String> = sqlx::query_scalar(
            "SELECT hash_chain FROM events 
             WHERE stream_id = ? ORDER BY version DESC LIMIT 1"
        )
        .bind(stream_id)
        .fetch_optional(&mut *tx)
        .await?;
        
        // Insert events with hash chain
        let mut version = current_version;
        let mut prev_hash = last_hash;
        
        for event in events {
            version += 1;
            let hash = calculate_hash(prev_hash.as_deref(), &event.payload);
            
            sqlx::query(
                "INSERT INTO events 
                 (id, stream_id, stream_type, version, event_type, payload, 
                  metadata, occurred_at, actor_id, actor_type, hash_chain, prev_hash)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&event.id)
            .bind(stream_id)
            .bind(&event.stream_type)
            .bind(version)
            .bind(&event.event_type)
            .bind(&event.payload)
            .bind(&event.metadata)
            .bind(event.occurred_at as i64)
            .bind(&event.actor_id)
            .bind(&event.actor_type)
            .bind(&hash)
            .bind(&prev_hash)
            .execute(&mut *tx)
            .await?;
            
            prev_hash = Some(hash);
        }
        
        tx.commit().await?;
        
        // Maybe create snapshot
        if let Some(config) = &self.config.snapshot {
            maybe_create_snapshot(self, stream_id, config).await?;
        }
        
        Ok(version as u64)
    }
    
    async fn read(
        &self,
        stream_id: &str,
        from_version: u64,
    ) -> Result<Vec<DomainEvent>, EventStoreError> {
        // Try to use snapshot as starting point
        let (events, start_version) = if let Some(snapshot) = 
            self.get_snapshot(stream_id).await? {
            if snapshot.version >= from_version {
                // Deserialize snapshot state
                let state = deserialize(&snapshot.state)?;
                return Ok(vec![DomainEvent::from_snapshot(state)]);
            }
            // Read events after snapshot
            let events = sqlx::query_as::<_, EventRow>(
                "SELECT * FROM events 
                 WHERE stream_id = ? AND version > ?
                 ORDER BY version ASC"
            )
            .bind(stream_id)
            .bind(snapshot.version as i64)
            .fetch_all(&self.pool)
            .await?;
            (events, snapshot.version)
        } else {
            // Read all events from beginning
            let events = sqlx::query_as::<_, EventRow>(
                "SELECT * FROM events 
                 WHERE stream_id = ? AND version >= ?
                 ORDER BY version ASC"
            )
            .bind(stream_id)
            .bind(from_version as i64)
            .fetch_all(&self.pool)
            .await?;
            (events, 0)
        };
        
        // Verify hash chain (in production, do periodically, not every read)
        if self.config.verify_on_read {
            verify_chain(&events)?;
        }
        
        Ok(events.into_iter().map(|e| e.into()).collect())
    }
    
    // ... get_snapshot, save_snapshot implementations
}
```

## Related Decisions

- **ADR-001**: Hexagonal architecture (event store is a driven adapter)
- **ADR-004**: SQLite as primary storage (for local-first)

## Notes

- MessagePack chosen over JSON for: smaller size, faster parsing, binary data support
- ULID chosen over UUID for: sortability (time-based prefix), lexicographic ordering
- SHA-256 chosen for: widely available, fast, sufficient for tamper evidence
- Compression (zstd) on snapshots for: 3-5x size reduction

---

*Proposed: 2025-01-18*  
*Accepted: 2025-01-22*  
*Implemented: 2025-02-15*
