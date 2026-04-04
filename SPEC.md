# SPEC: AgilePlus

## Overview

AgilePlus is a project management system with AI agent integration — a 24-crate Rust monorepo with hexagonal architecture, Python MCP server, and Plane.so/GitHub integration.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PRESENTATION LAYER                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                    │
│   │   pheno-cli  │    │  MCP Server  │    │   REST API   │                    │
│   │  (TypeScript)│    │   (Python)   │    │   (axum)     │                    │
│   └──────┬───────┘    └──────┬───────┘    └──────┬───────┘                    │
│          │                   │                   │                           │
└──────────┼───────────────────┼───────────────────┼───────────────────────────┘
           │                   │                   │
           ▼                   ▼                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                               DOMAIN LAYER                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────────────────────────────────────────────────────────┐    │
│   │                    agileplus-domain                               │    │
│   │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐          │    │
│   │  │ Feature  │  │WorkPackage│  │  Cycle   │  │  Module  │          │    │
│   │  │ StateMachine  StateMachine  StateMachine  StateMachine         │    │
│   │  └──────────┘  └──────────┘  └──────────┘  └──────────┘          │    │
│   └──────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                             ADAPTER LAYER                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│   │  SQLite     │  │   Git VCS   │  │  Plane.so   │  │   GitHub    │      │
│   │  Adapter    │  │   Adapter   │  │   Sync      │  │ Integration │      │
│   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
│                                                                              │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│   │    gRPC     │  │    NATS     │  │  Telemetry  │  │    Cache    │      │
│   │  (tonic)    │  │   Adapter   │  │  (OTel)     │  │   Adapter   │      │
│   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           INFRASTRUCTURE LAYER                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│   │   SQLite    │  │   Git Repo  │  │ Plane API   │  │  GitHub API │      │
│   │   (local)   │  │             │  │  (remote)   │  │  (remote)   │      │
│   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Components Table

| Component | Crate | Purpose | Dependencies |
|-----------|-------|---------|--------------|
| CLI | `agileplus-cli` | Command-line interface for all operations | domain, sqlite, api |
| Domain | `agileplus-domain` | Core business logic, state machines, events | hexagonal-rs |
| API | `agileplus-api` | REST API server (axum-based) | domain, auth, telemetry |
| gRPC | `agileplus-grpc` | Protocol buffer service definitions | tonic, prost |
| SQLite | `agileplus-sqlite` | Hexagonal storage adapter | rusqlite, domain |
| Plane | `agileplus-plane` | Plane.so sync (push/pull) | api-client, domain |
| GitHub | `agileplus-github` | GitHub integration | octocrab, gix |
| Telemetry | `agileplus-telemetry` | OpenTelemetry tracing/metrics | opentelemetry |
| Events | `agileplus-events` | Event sourcing infrastructure | domain, hexkit |
| Cache | `agileplus-cache` | In-memory caching layer | moka, domain |
| NATS | `agileplus-nats` | Message bus adapter | async-nats |
| Sync | `agileplus-sync` | Bidirectional sync engine | plane, github |
| Dashboard | `agileplus-dashboard` | Metrics and visualization | metrics, telemetry |
| Triage | `agileplus-triage` | Automated issue triage | ai-sdk, domain |
| Graph | `agileplus-graph` | Dependency graph operations | petgraph |
| P2P | `agileplus-p2p` | Peer-to-peer collaboration | libp2p |
| Fixtures | `agileplus-fixtures` | Test data generation | fake, domain |
| Subcommands | `agileplus-subcmds` | CLI subcommand implementations | clap |
| Tests | `agileplus-integration-tests` | Integration test suite | all crates |
| Contracts | `agileplus-contract-tests` | Pact contract tests | pact-consumer |
| Benchmarks | `agileplus-benchmarks` | Performance benchmarks | criterion |

---

## Data Models

### Core Entities

```rust
// Feature - Primary work unit
struct Feature {
    id: FeatureId,              // ULID
    slug: String,               // URL-friendly identifier
    title: String,              // Display name
    description: Option<String>,
    state: FeatureState,        // State machine
    priority: Priority,
    created_at: Timestamp,
    updated_at: Timestamp,
    cycle_id: Option<CycleId>,
    module_id: Option<ModuleId>,
    parent_id: Option<FeatureId>, // For hierarchies
}

enum FeatureState {
    Draft,
    Planned,
    InProgress,
    InReview,
    Done,
    Cancelled,
}

// WorkPackage - Granular task unit
struct WorkPackage {
    id: WorkPackageId,
    feature_id: FeatureId,
    title: String,
    description: Option<String>,
    state: WorkPackageState,
    assigned_to: Option<UserId>,
    effort_estimate: Option<StoryPoints>,
    created_at: Timestamp,
    updated_at: Timestamp,
}

enum WorkPackageState {
    Draft,
    Todo,
    InProgress,
    Blocked,
    Done,
}

// Cycle - Time-boxed iteration
struct Cycle {
    id: CycleId,
    name: String,
    start_date: Date,
    end_date: Date,
    state: CycleState,
    goals: Vec<String>,
}

// Module - Organizational grouping
struct Module {
    id: ModuleId,
    name: String,
    description: Option<String>,
    color: ColorCode,
    lead_id: Option<UserId>,
}

// Event - Audit trail entry
struct DomainEvent {
    id: EventId,
    aggregate_id: String,
    aggregate_type: String,
    event_type: String,
    payload: serde_json::Value,
    occurred_at: Timestamp,
    hash_chain: String,         // SHA-256 chain
}
```

### Event Sourcing Schema

```rust
// Event store table (SQLite)
CREATE TABLE events (
    id TEXT PRIMARY KEY,        -- ULID
    stream_id TEXT NOT NULL,  -- aggregate identifier
    stream_type TEXT NOT NULL,-- aggregate type
    version INTEGER NOT NULL, -- sequence within stream
    event_type TEXT NOT NULL, -- event discriminator
    payload BLOB NOT NULL,    -- MessagePack encoded
    metadata BLOB,            -- Additional context
    occurred_at INTEGER NOT NULL, -- Unix timestamp
    hash_chain TEXT NOT NULL, -- SHA-256 of prev hash + payload
    UNIQUE(stream_id, version)
);

CREATE INDEX idx_events_stream ON events(stream_id, version);
CREATE INDEX idx_events_type ON events(event_type, occurred_at);
```

---

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| CLI cold start | < 50ms | `time pheno-cli --help` |
| API request p99 | < 100ms | HTTP latency histogram |
| Event write | < 5ms | SQLite INSERT duration |
| Event read (1000) | < 50ms | Query + deserialize |
| Plane sync | < 5s | Full project sync |
| GitHub sync | < 10s | Issue/PR sync |
| Memory footprint | < 128MB | RSS at idle |
| SQLite operations | > 10K TPS | `INSERT`/`SELECT` |
| Binary size | < 20MB | Stripped release build |
| Test suite | < 60s | `cargo test --workspace` |

### Scaling Limits

- **SQLite**: Single-node, ~100K features per project
- **Event store**: Append-only, partition by date
- **Sync**: Rate-limited to API quotas (Plane/GitHub)
- **Cache**: LRU with 10K entry limit per type

---

## Security Considerations

- API keys: HS256-signed JWTs
- SQLite: Encrypted at rest (SQLCipher)
- Git: GPG-signed commits for audit
- Events: Immutable hash chain for tamper detection
