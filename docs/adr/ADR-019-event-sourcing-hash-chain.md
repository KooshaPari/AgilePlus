# ADR-019: Event Sourcing with SHA-256 Hash Chain Audit

**Date**: 2026-04-04
**Status**: Accepted
**Deciders**: AgilePlus Core Team

---

## Context

AgilePlus requires a complete, tamper-evident audit trail for all domain changes. This serves multiple purposes:

1. **Compliance**: Governance requirements demand immutable audit records
2. **Debugging**: Full history enables root cause analysis
3. **Accountability**: Agent actions must be attributable
4. **Integrity**: Hash chains provide cryptographic proof of sequence
5. **Eventual Consistency**: Event log enables sync between devices

Unlike traditional PM tools where audit is an afterthought, AgilePlus bakes it into the core architecture via **event sourcing**.

### Requirements

- Every domain mutation produces an event
- Events are append-only (immutable)
- Events form a cryptographic hash chain (SHA-256)
- Events can be replayed to reconstruct state
- Query patterns: "What happened to feature X?", "Who did Y?"
- Performance: <5ms event write, <50ms event replay for 1000 events

---

## Decision

### Architecture: Event Sourcing with Hash Chains

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Event Sourcing Architecture                            │
│                                                                         │
│  Domain Operation                                                         │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐     Produce      ┌──────────────────┐                 │
│  │   Feature    │ ──────────────► │  Domain Event    │                 │
│  │   Service    │                 │  (immutable)     │                 │
│  └──────────────┘                 └────────┬─────────┘                 │
│                                            │                            │
│                                            ▼                            │
│                                  ┌──────────────────┐                   │
│                                  │  Hash Chain      │                   │
│                                  │  Computation     │                   │
│                                  └────────┬─────────┘                   │
│                                           │                             │
│                                           ▼                             │
│                                  ┌──────────────────┐                   │
│                                  │  SQLite Event    │                   │
│                                  │  Store (append)  │                   │
│                                  └────────┬─────────┘                   │
│                                           │                             │
│                    ┌──────────────────────┼──────────────────────┐      │
│                    ▼                      ▼                      ▼      │
│           ┌──────────────┐      ┌──────────────┐      ┌──────────────┐ │
│           │  Projection: │      │  Projection: │      │  Projection: │ │
│           │  Current     │      │  Audit       │      │  Sync        │ │
│           │  State       │      │  Report      │      │  State       │ │
│           └──────────────┘      └──────────────┘      └──────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

### Event Structure

```rust
// Core event structure
struct DomainEvent {
    id: EventId,                    // ULID - globally unique
    stream_id: String,              // e.g., "feature:user-auth-flow"
    stream_type: StreamType,        // Feature, WorkPackage, Cycle
    version: u64,                   // Sequence number within stream
    event_type: EventType,          // Discriminator
    payload: serde_json::Value,    // Event-specific data
    metadata: EventMetadata,         // Context (who, why, how)
    occurred_at: Timestamp,         // When it happened
    actor_id: Option<ActorId>,      // Who triggered (user, agent, system)
    actor_type: ActorType,          // human, agent, system
    prev_hash: Option<Hash>,         // Previous event hash in chain
    hash: Hash,                     // SHA-256(this event's data)
}

// Event types
enum EventType {
    // Feature events
    FeatureCreated { slug: String, title: String },
    FeatureTitleUpdated { old: String, new: String },
    FeatureStateTransitioned { from: FeatureState, to: FeatureState },
    FeaturePrioritized { priority: Priority },
    FeatureAssigned { to: Option<AgentId> },
    FeatureSpecAttached { spec_hash: Hash },
    FeatureCancelled { reason: String },

    // WorkPackage events
    WorkPackageCreated { slug: String, title: String },
    WorkPackageAssigned { to: Option<AgentId> },
    WorkPackageStateTransitioned { from: WorkPackageState, to: WorkPackageState },
    WorkPackageEffortEstimated { points: StoryPoints },
    WorkPackageDependencyAdded { depends_on: WorkPackageId },
    WorkPackageCompleted,

    // Cycle events
    CycleStarted { start_date: Date, end_date: Date },
    CycleCompleted,
    CycleFeatureAdded { feature_id: FeatureId },
    CycleFeatureRemoved { feature_id: FeatureId },

    // Governance events
    GovernanceCheckPassed { rule: String },
    GovernanceCheckFailed { rule: String, reason: String },
    EvidenceAttached { evidence_id: EvidenceId },
    ReviewRequested { reviewer: AgentId },
    ReviewApproved { reviewer: AgentId },
    ReviewRejected { reviewer: AgentId, reason: String },

    // Agent events
    AgentDispatched { agent_id: AgentId, task: String },
    AgentCompleted { result: AgentResult },
    AgentFailed { error: String },
    AgentProgress { message: String },

    // Sync events
    SyncedFromRemote { source: String },
    SyncedToRemote { destination: String },
    ConflictResolved { strategy: String },
}
```

### Hash Chain Computation

Every event's hash includes:
1. The event's own data (id, stream_id, version, event_type, payload, metadata, occurred_at, actor_id, actor_type)
2. The previous event's hash (prev_hash)

```rust
// Hash chain computation
fn compute_event_hash(event: &DomainEvent) -> Hash {
    let mut hasher = Sha256::new();

    // Domain event data
    hasher.update(event.id.as_bytes());
    hasher.update(event.stream_id.as_bytes());
    hasher.update(&event.version.to_le_bytes());
    hasher.update(event.event_type.discriminant().as_bytes());
    hasher.update(&serde_json::to_vec(&event.payload).unwrap());
    hasher.update(&serde_json::to_vec(&event.metadata).unwrap());
    hasher.update(&event.occurred_at.timestamp().to_le_bytes());

    // Actor info
    if let Some(actor_id) = &event.actor_id {
        hasher.update(actor_id.as_bytes());
    }
    hasher.update(event.actor_type.discriminant().as_bytes());

    // Previous hash in chain
    if let Some(prev_hash) = &event.prev_hash {
        hasher.update(prev_hash.as_bytes());
    }

    Hash(hasher.finalize())
}

// Verification
fn verify_chain(events: &[DomainEvent]) -> Result<()> {
    for (i, event) in events.iter().enumerate() {
        let computed_hash = compute_event_hash(event);
        if computed_hash != event.hash {
            return Err(AuditError::HashMismatch {
                event_id: event.id,
                expected: computed_hash,
                actual: event.hash,
            });
        }

        if i > 0 {
            if event.prev_hash != Some(events[i-1].hash) {
                return Err(AuditError::BrokenChain {
                    event_id: event.id,
                });
            }
        }
    }
    Ok(())
}
```

### Event Store Schema

```sql
CREATE TABLE events (
    id TEXT PRIMARY KEY,           -- ULID
    stream_id TEXT NOT NULL,       -- Aggregate identifier
    stream_type TEXT NOT NULL,     -- feature, work_package, cycle
    version INTEGER NOT NULL,      -- Sequence within stream
    event_type TEXT NOT NULL,      -- Event discriminator
    payload BLOB NOT NULL,         -- MessagePack encoded
    metadata BLOB,                 -- MessagePack encoded
    occurred_at INTEGER NOT NULL,   -- Unix timestamp (ms)
    actor_id TEXT,                  -- User or agent ID
    actor_type TEXT NOT NULL,      -- human, agent, system
    hash_chain TEXT NOT NULL,       -- SHA-256 of this event
    prev_hash TEXT,                 -- Previous event's hash

    -- Performance indexes
    UNIQUE(stream_id, version)
);

CREATE INDEX idx_events_stream ON events(stream_id, version);
CREATE INDEX idx_events_type ON events(event_type, occurred_at);
CREATE INDEX idx_events_actor ON events(actor_id, occurred_at);
CREATE INDEX idx_events_occurred ON events(occurred_at);

-- Snapshots for fast reads
CREATE TABLE snapshots (
    id TEXT PRIMARY KEY,
    stream_id TEXT NOT NULL,
    stream_type TEXT NOT NULL,
    version INTEGER NOT NULL,
    state BLOB NOT NULL,           -- MessagePack encoded
    created_at INTEGER NOT NULL,

    UNIQUE(stream_id, version)
);

-- Event projection tracking
CREATE TABLE projections (
    name TEXT PRIMARY KEY,         -- e.g., "feature_state", "audit_report"
    last_processed_event_id TEXT,
    last_processed_at INTEGER,
);
```

### Projection Pattern

Events are projected into readable models:

```rust
// Projection: Feature current state
struct FeatureProjection {
    id: FeatureId,
    slug: String,
    title: String,
    description: Option<String>,
    state: FeatureState,
    priority: Priority,
    created_at: Timestamp,
    updated_at: Timestamp,
    spec_hash: Option<Hash>,
}

impl FeatureProjection {
    fn apply(event: &DomainEvent) -> Option<Self> {
        match event.event_type {
            EventType::FeatureCreated { slug, title } => Some(Self {
                id: event.stream_id.parse().unwrap(),
                slug,
                title,
                state: FeatureState::Created,
                ..Default::default()
            }),
            EventType::FeatureStateTransitioned { to, .. } => {
                // Load existing, update state
            },
            _ => None, // Events that don't affect feature projection
        }
    }
}

// Rebuild projection from events
async fn rebuild_projection(stream_id: &str) -> Result<FeatureProjection> {
    let events = event_store.get_events_for_stream(stream_id).await?;
    let mut projection = FeatureProjection::default();

    for event in events {
        if let Some(updated) = projection.apply(&event) {
            projection = updated;
        }
    }
    Ok(projection)
}
```

### Snapshot Strategy

For streams with many events, snapshots accelerate replay:

```rust
// Snapshot every 100 events
const SNAPSHOT_INTERVAL: u64 = 100;

async fn append_event_and_snapshot(
    event_store: &EventStore,
    stream_id: &str,
    event: DomainEvent,
) -> Result<()> {
    // Append event
    event_store.append(event.clone()).await?;

    // Check if snapshot needed
    if event.version % SNAPSHOT_INTERVAL == 0 {
        let projection = rebuild_projection(stream_id).await?;
        event_store.create_snapshot(
            stream_id,
            event.version,
            &projection,
        ).await?;
    }

    Ok(())
}
```

### Audit Trail API

```rust
// Audit query interface
#[derive(Debug, Clone)]
pub struct AuditQuery {
    pub feature_id: Option<FeatureId>,
    pub actor_id: Option<ActorId>,
    pub event_types: Vec<EventType>,
    pub from_date: Option<DateTime>,
    pub to_date: Option<DateTime>,
    pub limit: usize,
    pub offset: usize,
}

impl EventStore {
    // Query audit trail
    pub async fn query_audit(&self, query: AuditQuery) -> Result<Vec<AuditEntry>> {
        let mut sql = String::from(
            "SELECT e.*, a.name as actor_name
             FROM events e
             LEFT JOIN actors a ON e.actor_id = a.id
             WHERE 1=1"
        );

        if query.feature_id.is_some() {
            sql.push_str(" AND e.stream_id = ?");
        }
        if query.actor_id.is_some() {
            sql.push_str(" AND e.actor_id = ?");
        }
        // ... build query dynamically

        self.query_audit_impl(sql, query).await
    }

    // Generate audit report
    pub async fn generate_audit_report(
        &self,
        feature_id: &FeatureId,
    ) -> Result<AuditReport> {
        let events = self.get_events_for_stream(feature_id.as_str()).await?;

        let mut entries = Vec::new();
        for event in events {
            entries.push(AuditEntry {
                timestamp: event.occurred_at,
                actor: event.actor_id.map(|id| id.to_string()),
                action: event.event_type.description(),
                details: serde_json::to_value(&event.payload)?,
                hash: event.hash,
            });
        }

        Ok(AuditReport {
            feature_id: feature_id.clone(),
            entries,
            chain_valid: verify_chain(&events).is_ok(),
        })
    }
}
```

---

## Options Considered

### Option A: Traditional Audit Table (Rejected)

**Description**: Append-only audit table alongside regular CRUD.

```sql
CREATE TABLE audit_log (
    id SERIAL PRIMARY KEY,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    old_values JSONB,
    new_values JSONB,
    user_id TEXT,
    timestamp TIMESTAMP DEFAULT NOW()
);
```

**Pros**:
- Simple to implement
- Familiar pattern

**Cons**:
- ❌ Not cryptographically verifiable
- ❌ No replay capability
- ❌ Separate from domain model
- ❌ Query patterns limited

**Assessment**: ❌ Rejected — does not meet hash chain or replay requirements

### Option B: Append-Only Log with UUID Chain (Rejected)

**Description**: Events use sequential UUIDs but no cryptographic hash.

**Pros**:
- Simple
- Supports replay

**Cons**:
- ❌ No tamper evidence
- ❌ UUIDs can be predicted/guessed
- ❌ No integrity verification

**Assessment**: ❌ Rejected — fails integrity requirement

### Option C: Event Sourcing with SHA-256 Hash Chain (Selected)

**Description**: Full event sourcing with cryptographic hash chains.

**Pros**:
- ✅ Cryptographically tamper-evident
- ✅ Full replay capability
- ✅ Native to domain model
- ✅ Enables sync
- ✅ Compliance-ready

**Cons**:
- ⚠️ More complex implementation
- ⚠️ Storage overhead
- ⚠️ Event schema evolution challenges

**Assessment**: ✅ Selected — best integrity + capability combination

---

## Performance Benchmarks

| Operation | Target | Methodology |
|-----------|--------|-------------|
| Single event write | <5ms | INSERT with synchronous commit |
| Batch event write (100) | <100ms | Transaction with batch INSERT |
| Event read (stream, 1000 events) | <50ms | Query + MessagePack decode |
| Full projection rebuild | <200ms | Replay 1000 events |
| Snapshot create | <10ms | Serialize projection state |
| Chain verification (1000 events) | <100ms | Sequential SHA-256 |

```bash
# Benchmark command
cargo bench --package agileplus-benchmarks -- event_sourcing

# Expected output:
# event_write_single    time: [4.2ms 4.5ms 4.8ms]
# event_write_batch     time: [85ms 90ms 95ms]
# event_read_1000       time: [42ms 45ms 48ms]
# projection_rebuild     time: [180ms 195ms 210ms]
# chain_verify_1000      time: [90ms 95ms 100ms]
```

---

## Consequences

### Positive

1. **Complete audit**: Every change is recorded with full context
2. **Tamper-evident**: Hash chain makes tampering obvious
3. **Replay capability**: State can be reconstructed at any point
4. **Debugging**: Full history enables root cause analysis
5. **Sync foundation**: Event log enables delta sync
6. **Agent attribution**: Agent actions clearly recorded

### Negative

1. **Storage growth**: Event store grows indefinitely (mitigated by archival)
2. **Schema evolution**: Changing event schema requires migration strategy
3. **Query complexity**: Event store queries differ from traditional CRUD
4. **Learning curve**: Team must understand event sourcing

### Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Storage exhaustion | Low | Medium | Archival policy, snapshot strategy |
| Chain corruption | Very Low | Critical | Redundant hash verification |
| Schema migration bugs | Medium | High | Versioned events, upcasting |
| Performance degradation | Low | Medium | Snapshot strategy, indexing |

---

## References

- [ARCH-002] Fowler, M. "Event Sourcing" - martinfowler.com/eaaDev/EventSourcing.html
- [LF-001] Kleppmann, M. et al. (2019). "A Conflict-Free Replicated JSON Datatype" - arxiv.org/abs/1608.03960
- [BENCH-002] Criterion.rs - Rust benchmark framework - bheisner.github.io/criterion.rs
- [BENCH-001] hyperfine - Command-line benchmark tool - github.com/sharkdp/hyperfine

---

*Decision made 2026-04-04 based on compliance requirements and audit chain design goals.*
