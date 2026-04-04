# ADR-017: Local-First Architecture with SQLite and CRDT Synchronization

**Date**: 2026-04-04
**Status**: Accepted
**Deciders**: AgilePlus Core Team

---

## Context

AgilePlus requires offline-capable, privacy-respecting data storage with multi-device synchronization. Users work in environments with intermittent connectivity (airplane, remote locations, conference venues with poor WiFi), and many teams have strict data residency requirements that prohibit cloud-only solutions.

The system must:
1. Work fully offline with local SQLite storage
2. Sync bidirectionally when connectivity is available
3. Handle conflicts gracefully without data loss
4. Support P2P sync between team members' devices
5. Maintain ACID guarantees for local operations

### Constraints

- SQLite must be the primary storage engine (no PostgreSQL/MySQL dependency)
- Sync must work over unreliable networks (high latency, packet loss)
- Conflict resolution must be deterministic
- P2P must work through NATs without port forwarding
- Must integrate with existing git-backed workflow

---

## Decision

### Primary Storage: SQLite with WAL Mode

SQLite in WAL (Write-Ahead Logging) mode serves as the single source of truth:

```rust
// Connection configuration for WAL mode
let config = rusqlite::config::ConfigBuilder::new()
    .journal_mode(rusqlite::DatabaseName::Main, rusqlite::JournalMode::Wal)
    .synchronous(rusqlite::Synchronous::Normal)
    .busy_timeout(Duration::from_secs(5))
    .lock_shared_cache(true)
    .build()?;
```

**Schema Design**:
- All domain entities stored in SQLite with ULID primary keys
- Event store as append-only table with hash chains
- Full-text search via SQLite FTS5 extension
- JSON fields for flexible metadata (via serde_json)

### Sync Architecture: Delta-CRDT Based

We adopt **delta-CRDT** (Diamond Types algorithm) for conflict-free sync:

```
┌──────────────────────────────────────────────────────────────────┐
│                     Sync Architecture                              │
│                                                                   │
│  Device A ──────┐                                                 │
│  SQLite         │     ┌─────────────┐     ┌─────────────┐         │
│  WAL + CRDT    │◄───►│  Sync Log   │◄───►│  Device B   │         │
│                 │     │  (delta)    │     │  SQLite     │         │
│  Device C ──────┼────►│             │◄────│  WAL + CRDT │         │
│  SQLite         │◄───►└─────────────┘     │             │         │
│  WAL + CRDT     │     │  Tailscale  │     │  Device D   │         │
│                 │◄───►│  WireGuard  │◄────│  SQLite     │         │
└─────────────────┘     │  (P2P mesh) │     │  WAL + CRDT │         │
                        └─────────────┘     └─────────────┘         │
                                │                                    │
                                ▼                                    │
                        ┌─────────────┐                              │
                        │  Git Repo   │  (optional git-backed sync)   │
                        │  (remote)   │                              │
                        └─────────────┘                              │
└──────────────────────────────────────────────────────────────────┘
```

### Conflict Resolution: Operation-Based CRDT

**Key insight**: For domain entities (Features, WorkPackages), we use Last-Write-Wins (LWW) with vector clocks for causality. For structured data (specs, descriptions), we use RGA (Replicated Growable Array) for text.

**Resolution Rules**:

| Entity Type | CRDT Type | Conflict Resolution |
|-------------|-----------|---------------------|
| Feature state | LWW-Register | Most recent state wins |
| Feature title | LWW-Register | Most recent wins |
| WorkPackage state | LWW-Register | Most recent wins |
| Spec content | RGA (text) | Semantic merge |
| Dependencies | OR-Set | Union with tombstones |
| Audit entries | Append-only | No conflicts (immutable) |

### P2P Transport: libp2p + Tailscale

**Two-tier networking**:
1. **Tailscale (primary)**: Zero-config VPN for team mesh. Automatic NAT traversal via DERP relay.
2. **libp2p (fallback)**: Direct P2P for advanced users. Full connection upgradeability.

```rust
// Tailscale integration (simplied)
let ts_client = tailscale::Client::new().await?;
let peer_id = ts_client.device_id()?;
let peers = ts_client.list_peers().await?;

// libp2p fallback for non-Tailscale users
let transport = libp2p::development_transport({
    let tls_config = libp2p_tls::Config::new()?;
    let yamux = libp2p::yamux::Config::default();
    (tls_config, yamux)
}).await?;
```

### Git-Backed Sync (Optional)

For teams preferring git as the sync substrate:

```
git-backed sync flow:
1. Local changes committed to feature branch
2. `agileplus sync push` → pushes to remote
3. `agileplus sync pull` → merges from remote
4. SQLite events serialized to git LFS if >10MB
5. Conflict detection via git's 3-way merge
```

---

## Options Considered

### Option A: PostgreSQL + NATS (Rejected)

**Description**: Central PostgreSQL database with NATS for pub/sub sync.

**Pros**:
- Familiar to most developers
- Excellent tooling ecosystem
- Strong consistency

**Cons**:
- ❌ Requires running database server
- ❌ No offline capability
- ❌ NATS adds infrastructure complexity
- ❌ Single point of failure

**Assessment**: ❌ Rejected — violates local-first requirement

### Option B: CouchDB/PouchDB (Rejected)

**Description**: Use CouchDB's built-in sync protocol.

**Pros**:
- Built-in sync protocol
- Works offline
- Conflict resolution built-in

**Cons**:
- ❌ Requires CouchDB instance
- ❌ Not ideal for complex queries
- ❌ JavaScript-centric ecosystem
- ❌ Limited Rust client support

**Assessment**: ❌ Rejected — infrastructure requirement conflicts with local-first

### Option C: Electric SQL + Neon (Rejected)

**Description**: Use Electric SQL's local-first SQLite with Neon server.

**Pros**:
- Excellent local-first story
- PostgreSQL compatibility
- Automatic sync

**Cons**:
- ❌ Still requires Neon (cloud)
- ❌ Limited Rust SDK
- ❌ New project, evolving API
- ❌ SSPL license concerns

**Assessment**: ❌ Rejected — still requires cloud dependency

### Option D: Custom SQLite + Delta-CRDT + libp2p (Selected)

**Description**: Build custom sync layer on SQLite with delta-CRDT and libp2p/Tailscale.

**Pros**:
- ✅ True local-first (no required server)
- ✅ Full control over sync protocol
- ✅ Works with git-backed workflow
- ✅ Deterministic conflict resolution
- ✅ Privacy-preserving

**Cons**:
- ⚠️ More implementation work
- ⚠️ No existing Rust delta-CRDT library
- ⚠️ P2P NAT traversal complexity

**Assessment**: ✅ Selected — best balance of local-first and flexibility

---

## Implementation Details

### Database Schema

```sql
-- Core tables
CREATE TABLE features (
    id TEXT PRIMARY KEY,           -- ULID
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    state TEXT NOT NULL,
    priority INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,            -- Soft delete for OR-Set
    vector_clock TEXT NOT NULL,    -- JSON vector clock
    last_writer TEXT NOT NULL      -- Device ID
);

CREATE TABLE events (
    id TEXT PRIMARY KEY,
    stream_id TEXT NOT NULL,
    stream_type TEXT NOT NULL,
    version INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    payload BLOB NOT NULL,
    metadata BLOB,
    occurred_at INTEGER NOT NULL,
    actor_id TEXT,
    actor_type TEXT,
    hash_chain TEXT NOT NULL,
    prev_hash TEXT,
    delta TEXT,                    -- For delta-CRDT sync
    UNIQUE(stream_id, version)
);

CREATE TABLE sync_state (
    device_id TEXT PRIMARY KEY,
    last_sync INTEGER NOT NULL,
    vector_clock TEXT NOT NULL,
    pending_deltas BLOB
);

CREATE TABLE work_packages (
    id TEXT PRIMARY KEY,
    feature_id TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    state TEXT NOT NULL,
    assigned_to TEXT,
    effort_estimate INTEGER,
    file_scope TEXT,               -- JSON array
    dependencies TEXT,             -- JSON array
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    vector_clock TEXT NOT NULL,
    FOREIGN KEY (feature_id) REFERENCES features(id)
);
```

### Sync Protocol

```rust
// Delta-CRDT sync message
struct SyncDelta {
    device_id: String,
    vector_clock: VectorClock,
    operations: Vec<DeltaOperation>,
    timestamp: u64,
}

enum DeltaOperation {
    FeatureCreate { id: String, data: Feature },
    FeatureUpdate { id: String, field: String, value: serde_json::Value },
    FeatureDelete { id: String },
    WorkPackageCreate { id: String, data: WorkPackage },
    WorkPackageUpdate { id: String, field: String, value: serde_json::Value },
    EventAppend { event: DomainEvent },
}

struct VectorClock {
    clock: HashMap<DeviceId, u64>,
}
```

### Tailscale Integration

```rust
// tailscale.rs
pub struct TailscaleSync {
    client: tailscale::Client,
    devices: HashMap<DeviceId, tailscale::Peer>,
}

impl TailscaleSync {
    pub async fn connect(&self) -> Result<()> {
        let status = self.client.status().await?;
        for peer in status.peers {
            if peer.online && peer.allow_sync {
                self.devices.insert(peer.device_id, peer);
            }
        }
        Ok(())
    }

    pub async fn send_delta(&self, peer: &tailscale::Peer, delta: SyncDelta) -> Result<()> {
        self.client.send(peer.ip, &bincode::serialize(&delta)?).await
    }
}
```

---

## Consequences

### Positive

1. **True offline capability**: Users can work fully offline with no required server
2. **Privacy**: Data never leaves user's devices unless explicitly shared
3. **Performance**: SQLite local operations are <5ms
4. **Git integration**: Syncs naturally with existing VCS workflow
5. **No vendor lock-in**: No proprietary cloud service required

### Negative

1. **Implementation complexity**: Delta-CRDT sync is nontrivial
2. **No real-time collaboration**: Sync is eventual, not live
3. **Conflict UX**: Must design clear conflict resolution UI
4. **Storage limits**: SQLite single-file limit ~281TB (theoretical), ~10GB practical for performance

### Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Delta-CRDT implementation bugs | Medium | High | Extensive testing with property-based tests |
| Sync performance at scale | Low | Medium | Archive old events, paginate sync |
| NAT traversal failures | Medium | Medium | Tailscale fallback always works |
| Schema migrations offline | Low | High | Forward-only migrations, versioned schema |

---

## References

- [LF-001] Kleppmann, M. et al. (2019). "A Conflict-Free Replicated JSON Datatype" - arxiv.org/abs/1608.03960
- [LF-002] Ink & Switch (2021). "Local-First Software" - inkandswitch.com/local-first
- [LF-004] Yjs - CRDT-based shared editing - docs.yjs.dev
- [LF-005] Automerge - JSON-like CRDT library - automerge.org
- [LF-008] libp2p - Modular P2P networking - docs.libp2p.io
- [LF-009] Tailscale - Zero-config VPN - tailscale.com
- [BENCH-001] hyperfine - Command-line benchmark tool - github.com/sharkdp/hyperfine

---

*Decision made 2026-04-04 based on local-first requirements and git-backed sync goals.*
