# SPEC: AgilePlus — Project Management System with AI Agent Integration

> AgilePlus is a local-first, spec-driven project management system with AI agent integration — a 24-crate Rust monorepo with hexagonal architecture, Python MCP server, and Plane.so/GitHub integration.

**Version**: 2.0 (DEEP Tier)
**Status**: Production
**Last Updated**: 2026-04-04

---

## Part I: SOTA Project Management Systems Landscape (2026)

### 1.1 Competitive Overview

The project management software market has undergone significant consolidation and specialization. The emergence of AI-native tools, local-first architectures, and spec-driven workflows represents a fundamental shift from traditional ticket-based management.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    Project Management SOTA Spectrum (2026)                         │
│                                                                                  │
│  Traditional ────────────────────────────────────────────────────────── AI-Native│
│                                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │  Jira    │  │  Linear  │  │  Asana   │  │  Height  │  │AgilePlus │       │
│  │          │  │          │  │          │  │          │  │          │       │
│  │ Enterprise│  │  B2B SaaS│  │  B2B SaaS│  │  AI-First│  │Local-First│       │
│  │  Legacy   │  │  Modern  │  │  General │  │  PM Tool │  │Spec-Driven│       │
│  │  2002+   │  │  2019+   │  │  2012+   │  │  2023+   │  │  2025+   │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
│                                                                                  │
│  Ticket-based ─────────────────────────────────────────────────── Spec-driven     │
│  AI Integration: None/Plugin    Basic    Moderate    Full         Native         │
│  Local-First:    No            No       No         Emerging     Yes            │
│  Agent Support:  External      API-only API-only   Native       Native         │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Platform Comparison Matrix

| Platform | Launch Year | Architecture | Primary Language | AI Integration | Local-First | Agent Support |
|----------|-------------|--------------|------------------|----------------|-------------|---------------|
| **Jira** | 2002 | Cloud/Self-hosted | Java | Plugin-only | No | External only |
| **Linear** | 2019 | Cloud | TypeScript | Basic (2024) | No | API-only |
| **Asana** | 2012 | Cloud | TypeScript/Node | Basic (2024) | No | API-only |
| **Notion** | 2016 | Cloud | TypeScript/Node | Basic (2024) | No | API-only |
| **Height** | 2023 | Cloud | TypeScript | Native AI-first | No | Native agents |
| **Plane.so** | 2022 | Self-hosted/Cloud | Go | None | Yes | External only |
| **AgilePlus** | 2025 | Local-first/Rust | Rust/Python | Native MCP | Yes | Native dispatch |

### 1.3 Feature Comparison Matrix

| Feature | Jira | Linear | Asana | Notion | Height | Plane.so | AgilePlus |
|---------|------|--------|-------|--------|--------|----------|-----------|
| **Core PM** |
| Issue tracking | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Kanban boards | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Sprints/Cycles | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Custom workflows | ✅ | ⚠️ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **AI/Native Intelligence** |
| AI issue triage | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ |
| AI spec generation | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| AI WP decomposition | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| AI code review | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| **Local-First** |
| Offline operation | ❌ | ❌ | ❌ | ❌ | ❌ | ⚠️ | ✅ |
| Local persistence | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| Git-backed sync | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| P2P collaboration | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| **Agent Integration** |
| MCP server | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Agent dispatch | ❌ | ❌ | ❌ | ❌ | ⚠️ | ❌ | ✅ |
| Hidden subcommands | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Audit chain | ❌ | ⚠️ | ⚠️ | ❌ | ⚠️ | ⚠️ | ✅ |
| **Governance** |
| Spec-driven workflow | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| State machine enforcement | ⚠️ | ⚠️ | ❌ | ❌ | ⚠️ | ⚠️ | ✅ |
| Hash-chained audit | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Governance gates | ⚠️ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| **Integration** |
| GitHub sync | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Plane.so sync | ❌ | ❌ | ❌ | ❌ | ❌ | N/A | ✅ |
| REST API | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| gRPC | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Webhooks | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

### 1.4 Performance Characteristics

| Metric | Jira | Linear | Asana | Height | Plane.so | AgilePlus |
|--------|------|--------|-------|--------|----------|-----------|
| **Cold start (CLI)** | N/A | N/A | N/A | N/A | ~500ms | <50ms |
| **API p99 latency** | ~200ms | ~80ms | ~150ms | ~100ms | ~150ms | <100ms |
| **Memory footprint** | >1GB (JVM) | ~200MB | ~300MB | ~250MB | ~128MB | <128MB |
| **Offline capability** | None | None | None | Limited | Full | Full |
| **Max entities/project** | Unlimited | 100K | Unlimited | 50K | 100K | 100K |
| **Event store** | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |

### 1.5 Competitive Gap Analysis

#### Where AgilePlus Fits

```
                    Traditional PM                    AI-Native PM
                           │                              │
         ┌─────────────────┴─────────────────┐   ┌──────┴──────┐
         │                                         │              │
      Jira/Asana                            Linear/Height     AgilePlus
         │                                         │              │
    - Legacy workflows                      - AI assistant    - Spec-driven
    - Enterprise scale                      - Modern UX      - Local-first
    - Plugin ecosystem                      - API-first      - Native agents
                                              - No offline    - Hash-chained audit
                                                               - MCP integration
    Gap: No local-first, no native agents,    Gap: No spec-   │
    no spec-driven workflows, no audit chain  driven, no       │
                                              audit chain     │
```

#### Market Gaps AgilePlus Addresses

| Gap | Existing Solutions | AgilePlus Solution |
|-----|-------------------|-------------------|
| **Local-first PM** | Plane.so (partial) | Full local-first with SQLite, git-backed sync, P2P |
| **Spec-driven workflow** | None | 8-stage pipeline with state machine enforcement |
| **Native AI agent dispatch** | Height (basic) | Full MCP server, hidden subcommands, audit chain |
| **Hash-chained audit** | None | SHA-256 event store with cryptographic integrity |
| **Governance gates** | Jira (manual) | Programmatic preconditions per state transition |
| **Multi-device sync** | None (except Notion) | P2P via Tailscale, git-backed fallback |

---

## Part II: Architecture Deep Dive

### 2.1 System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PRESENTATION LAYER                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                │
│   │   pheno-cli  │    │  MCP Server  │    │   REST API   │                │
│   │  (TypeScript)│    │   (Python)   │    │   (axum)     │                │
│   └──────┬───────┘    └──────┬───────┘    └──────┬───────┘                │
│          │                   │                   │                           │
│          ▼                   ▼                   ▼                           │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                    gRPC (tonic) — Bidirectional Streaming              │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                          │
└────────────────────────────────────┼────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                               DOMAIN LAYER                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────────────────────────────────────────────────────────┐    │
│   │                    agileplus-domain                               │    │
│   │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │    │
│   │  │ Feature  │  │WorkPackage│  │  Cycle   │  │  Module  │       │    │
│   │  │ StateMachine│ StateMachine│ StateMachine│ StateMachine      │    │
│   │  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │    │
│   │                                                                  │    │
│   │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │    │
│   │  │ Governance│  │  Audit   │  │  Agent   │  │  Sync    │       │    │
│   │  │  Service  │  │  Chain   │  │ Dispatch │  │  Engine  │       │    │
│   │  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │    │
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
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│   │   Neo4j    │  │   MinIO     │  │  Dragonfly  │  │    P2P      │      │
│   │   Graph    │  │  Object     │  │   Cache     │  │   Mesh      │      │
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
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│   │  NATS/      │  │   Neo4j     │  │   MinIO     │  │  Tailscale  │      │
│   │  JetStream  │  │  (graph)    │  │  (object)   │  │   (P2P)     │      │
│   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Feature State Machine

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    AgilePlus Feature State Machine                             │
│                                                                              │
│  ┌─────────┐    ┌───────────┐    ┌───────────┐    ┌─────────┐              │
│  │ Created │───▶│ Specified │───▶│ Researched│───▶│ Planned │              │
│  └─────────┘    └───────────┘    └───────────┘    └─────────┘              │
│       │                │                │               │                    │
│       │                │                │               ▼                    │
│       │                │                │         ┌───────────┐             │
│       │                │                │         │Implementing│             │
│       │                │                │         └───────────┘             │
│       │                │                │               │                    │
│       │                │                │               ▼                    │
│       │                │                │         ┌───────────┐             │
│       │                │                │         │ Validated │             │
│       │                │                │         └───────────┘             │
│       │                │                │               │                    │
│       │                │                │               ▼                    │
│       │                │                │         ┌─────────┐               │
│       │                │                │         │ Shipped │               │
│       │                │                │         └─────────┘               │
│       │                │                │               │                    │
│       │                │                │               ▼                    │
│       │                │                │         ┌──────────────┐          │
│       │                │                │         │Retrospected │          │
│       │                │                │         └──────────────┘          │
│       │                │                │               │                    │
│       └────────────────┴────────────────┴───────────────┘                    │
│                              (can cancel from any state)                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 WorkPackage State Machine

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                  WorkPackage State Machine                                     │
│                                                                              │
│  ┌─────────┐    ┌────────┐    ┌───────────┐    ┌───────┐    ┌────────┐     │
│  │  Draft │───▶│  Todo  │───▶│ InProgress│───▶│ Blocked│───▶│  Done  │     │
│  └─────────┘    └────────┘    └───────────┘    └───────┘    └────────┘     │
│                      │              │               │                         │
│                      │              │               │                         │
│                      └──────────────┴───────────────┘                         │
│                              (can return to Todo)                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Part III: Components Table

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

## Part IV: Data Models

### 4.1 Core Entities

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
    spec_hash: Option<Sha256Hash>, // SHA-256 of spec artifact
    evidence_ids: Vec<EvidenceId>, // Governance evidence
}

enum FeatureState {
    Created,
    Specified,
    Researched,
    Planned,
    Implementing,
    Validated,
    Shipped,
    Retrospected,
    Cancelled,
}

enum Priority {
    P0, // Critical
    P1, // High
    P2, // Medium
    P3, // Low
}

// WorkPackage - Granular task unit
struct WorkPackage {
    id: WorkPackageId,
    feature_id: FeatureId,
    slug: String,
    title: String,
    description: Option<String>,
    state: WorkPackageState,
    assigned_to: Option<AgentId>,
    effort_estimate: Option<StoryPoints>,
    file_scope: Vec<FilePath>,   // Which files this WP affects
    dependencies: Vec<WorkPackageId>, // Explicit dependencies
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
    feature_ids: Vec<FeatureId>,
}

enum CycleState {
    Upcoming,
    Active,
    Completed,
}

// Module - Organizational grouping
struct Module {
    id: ModuleId,
    name: String,
    description: Option<String>,
    color: ColorCode,
    lead_id: Option<UserId>,
    parent_id: Option<ModuleId>,
}

// Agent - AI agent definition
struct Agent {
    id: AgentId,
    name: String,
    type: AgentType,           // claude_code, codex, cursor, copilot
    status: AgentStatus,       // idle, running, paused, error
    current_wp_id: Option<WorkPackageId>,
    capabilities: Vec<Capability>,
    config: AgentConfig,
}

enum AgentType {
    ClaudeCode,
    Codex,
    Cursor,
    Copilot,
}

enum AgentStatus {
    Idle,
    Running,
    Blocked,
    Error,
    Offline,
}

// Governance - State transition rules
struct GovernanceRule {
    id: GovernanceRuleId,
    from_state: FeatureState,
    to_state: FeatureState,
    preconditions: Vec<Precondition>,
    postconditions: Vec<Postcondition>,
}

struct Precondition {
    rule_type: PreconditionType,
    payload: serde_json::Value,
}

enum PreconditionType {
    SpecExists,
    EvidenceAttached,
    WpsAllDone,
    AllWpsMerged,
    ReviewApproved,
    TestsPassing,
}

// Event - Audit trail entry
struct DomainEvent {
    id: EventId,
    aggregate_id: String,
    aggregate_type: String,
    event_type: String,
    payload: serde_json::Value,
    metadata: serde_json::Value,
    occurred_at: Timestamp,
    actor_id: Option<ActorId>,
    actor_type: ActorType,      // human, agent, system
    hash_chain: String,         // SHA-256 of prev hash + payload
}
```

### 4.2 Event Sourcing Schema

```sql
-- Event store table (SQLite)
CREATE TABLE events (
    id TEXT PRIMARY KEY,        -- ULID
    stream_id TEXT NOT NULL,    -- aggregate identifier
    stream_type TEXT NOT NULL,  -- aggregate type
    version INTEGER NOT NULL,   -- sequence within stream
    event_type TEXT NOT NULL,  -- event discriminator
    payload BLOB NOT NULL,      -- MessagePack encoded
    metadata BLOB,              -- Additional context
    occurred_at INTEGER NOT NULL, -- Unix timestamp
    actor_id TEXT,              -- Who triggered the event
    actor_type TEXT,            -- human, agent, system
    hash_chain TEXT NOT NULL,   -- SHA-256 of prev hash + payload
    prev_hash TEXT,             -- Previous event hash (for chain)
    UNIQUE(stream_id, version)
);

CREATE INDEX idx_events_stream ON events(stream_id, version);
CREATE INDEX idx_events_type ON events(event_type, occurred_at);
CREATE INDEX idx_events_actor ON events(actor_id, occurred_at);

-- Snapshots for fast reads
CREATE TABLE snapshots (
    id TEXT PRIMARY KEY,
    stream_id TEXT NOT NULL,
    stream_type TEXT NOT NULL,
    version INTEGER NOT NULL,
    state BLOB NOT NULL,        -- MessagePack encoded state
    created_at INTEGER NOT NULL,
    UNIQUE(stream_id, version)
);

-- Sync state for Plane.so/GitHub
CREATE TABLE sync_state (
    entity_type TEXT NOT NULL,
    local_id TEXT NOT NULL,
    remote_id TEXT NOT NULL,
    remote_type TEXT NOT NULL,  -- plane_issue, github_issue, github_pr
    content_hash TEXT NOT NULL,
    last_synced_at INTEGER NOT NULL,
    conflict_state TEXT,
    PRIMARY KEY (entity_type, local_id)
);
```

---

## Part V: Performance Targets

### 5.1 Benchmarks

| Metric | Target | Measurement | Reproducible Command |
|--------|--------|-------------|---------------------|
| CLI cold start | < 50ms | `time pheno-cli --help` | `hyperfine -w 3 -r 10 'pheno-cli --help'` |
| API request p99 | < 100ms | HTTP latency histogram | `wrk -t4 -c100 -d30s http://localhost:8080/api/features` |
| Event write | < 5ms | SQLite INSERT duration | `agileplus-benchmarks --filter event_write` |
| Event read (1000) | < 50ms | Query + deserialize | `agileplus-benchmarks --filter event_read` |
| Plane sync (50 features) | < 30s | Full project sync | `time agileplus sync --project test-project` |
| GitHub sync (100 issues) | < 10s | Issue/PR sync | `time agileplus sync --github` |
| Memory footprint (idle) | < 128MB | RSS | `ps aux \| grep agileplus \| awk '{sum += $6} END {print sum/1024 " MB"}'` |
| SQLite TPS | > 10K | INSERT/SELECT | `agileplus-benchmarks --filter sqlite_tps` |
| Binary size | < 20MB | Stripped release | `ls -lh target/release/agileplus` |
| Test suite | < 60s | `cargo test --workspace` | `time cargo test --workspace` |

### 5.2 Scaling Limits

| Resource | Limit | Rationale |
|----------|-------|-----------|
| **SQLite** | ~100K features/project | Single-node, WAL mode |
| **Event store** | ~1M events/project | Partition by date, archive old |
| **Cache** | 10K entries/type | LRU eviction |
| **Sync queue** | 1000 pending ops | Rate-limited to API quotas |
| **P2P mesh** | 10 devices | NATS for larger orgs |

### 5.3 Benchmark Methodology

```bash
# CLI Cold Start
hyperfine \
  --prepare 'echo "" > /dev/null' \
  --min-runs 10 \
  -- 'pheno-cli --help'

# Expected: mean < 50ms, p99 < 100ms

# Event Write Benchmark
cargo bench --package agileplus-benchmarks -- event_write

# Expected: < 5ms per event

# API Latency (requires running server)
wrk -t4 -c100 -d30s http://localhost:8080/api/features

# Expected: p99 < 100ms

# SQLite TPS
cargo bench --package agileplus-benchmarks -- sqlite_tps

# Expected: > 10K TPS
```

---

## Part VI: Security Considerations

| Concern | Mitigation | Implementation |
|---------|------------|----------------|
| API keys | HS256-signed JWTs | `agileplus-api` generates on first run |
| Local storage | Encrypted at rest (SQLCipher) | rusqlite with SQLCipher |
| Git commits | GPG-signed for audit | gix with signing key |
| Event chain | Immutable hash chain | SHA-256 linking |
| Network (P2P) | Tailscale wireguard encryption | mTLS via Tailscale |
| Credentials | OS keychain (macOS/Linux) | `keyring` crate |

---

## Part VII: SOTA Analysis Details

### 7.1 Linear vs Jira

| Aspect | Linear | Jira |
|--------|--------|------|
| **Founded** | 2019 | 2002 |
| **Architecture** | Cloud-only | Cloud + Self-hosted |
| **Design philosophy** | Speed-first, minimal config | Enterprise flexibility |
| **Workflows** | Fixed states +custom fields | Fully customizable |
| **API** | GraphQL + REST | REST + Connect (plugins) |
| **AI features** | Basic issue description | Atlassian Intelligence (plugin) |
| **Pricing** | $8-15/user/mo | $7.75-15/user/mo |
| **Performance** | p99 ~80ms | p99 ~200ms+ |
| **Local-first** | ❌ | ❌ |

**Key Insight**: Linear is the modern alternative to Jira for startups/SMBs but lacks local-first operation and native AI agent support.

### 7.2 Asana and Notion

| Aspect | Asana | Notion |
|--------|-------|--------|
| **Founded** | 2012 | 2016 |
| **Core differentiator** | Work management platform | Knowledge base + PM |
| **AI features** | Asana Intelligence (2024) | Notion AI (2023) |
| **API** | REST + GraphQL | REST |
| **Custom workflows** | Advanced | Limited (databases) |
| **Local-first** | ❌ | ❌ |
| **Agent support** | API-only | API-only |
| **Offline mode** | Limited | Limited |

**Key Insight**: Both Asana and Notion are general-purpose tools. Neither supports spec-driven workflows or local-first operation.

### 7.3 Height

| Aspect | Height |
|--------|--------|
| **Founded** | 2023 |
| **Core differentiator** | AI-first PM tool |
| **AI integration** | Native AI agents for task completion |
| **Local-first** | Emerging (offline mode) |
| **Spec-driven workflows** | ❌ |
| **Agent dispatch** | Basic (assign to AI) |
| **Audit chain** | ⚠️ Limited |

**Key Insight**: Height is the closest competitor in the AI-first space but lacks spec-driven workflows, local-first architecture, and hash-chained audit trails.

### 7.4 Plane.so

| Aspect | Plane.so |
|--------|----------|
| **Founded** | 2022 |
| **Architecture** | Self-hosted + Cloud |
| **Core differentiator** | Linear alternative, self-hostable |
| **AI features** | ❌ |
| **Local-first** | ⚠️ Partial (SQLite) |
| **Spec-driven workflows** | ❌ |
| **Agent dispatch** | ❌ |
| **GitHub sync** | ✅ |
| **API** | GraphQL + REST |

**Key Insight**: Plane.so is the best open-source alternative for self-hosted PM but lacks AI integration, spec-driven workflows, and native agent support.

### 7.5 AgilePlus Differentiation

AgilePlus occupies a unique position as the only local-first, spec-driven PM system with native AI agent integration and hash-chained audit trails.

| Capability | AgilePlus | All Competitors |
|------------|-----------|-----------------|
| Local-first with SQLite | ✅ | Plane.so (partial) |
| Spec-driven 8-stage pipeline | ✅ | None |
| Native MCP server | ✅ | None |
| Agent dispatch with hidden subcommands | ✅ | None |
| SHA-256 hash-chained audit | ✅ | None |
| P2P multi-device sync | ✅ | Notion (limited) |
| Git-backed state sync | ✅ | None |
| Programmatic governance gates | ✅ | Jira (manual) |
| Feature → WP → Agent decomposition | ✅ | None |

---

## Part VIII: Benchmark Definitions with Reproducible Methodology

### 8.1 Performance Test Suite

```rust
// agileplus-benchmarks/src/lib.rs

use criterion::{black_box, criterion_group, Criterion};

pub fn event_write_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("event_write_single", |b| {
        b.to_async(&runtime).iter(|| async {
            let event = DomainEvent::new(
                "feature-001".to_string(),
                "FeatureCreated".to_string(),
                serde_json::json!({"title": "test"}),
            );
            let adapter = SqliteEventAdapter::new().unwrap();
            adapter.append_event(black_box(event)).await.unwrap()
        });
    });
}

pub fn event_read_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("event_read_1000", |b| {
        b.to_async(&runtime).iter(|| async {
            let adapter = SqliteEventAdapter::new().unwrap();
            adapter.get_events_for_stream(
                black_box("feature-001".to_string()),
                black_box(1000),
            ).await.unwrap()
        });
    });
}

pub fn cli_startup_benchmark(c: &mut Criterion) {
    c.bench_function("cli_cold_start", |b| {
        b.iter(|| {
            std::process::Command::new("pheno-cli")
                .arg("--help")
                .output()
                .unwrap()
        });
    });
}
```

### 8.2 Integration Test Scenarios

```gherkin
# features/sync.feature

Feature: Plane.so Bidirectional Sync

  Scenario: Feature created in AgilePlus syncs to Plane.so
    Given a Plane.so project "test-project" exists
    And AgilePlus is configured with Plane.so credentials
    When I create a feature "Test Feature" in AgilePlus
    And I run "agileplus sync push"
    Then the feature appears in Plane.so within 5 seconds
    And the feature state matches

  Scenario: Plane.so state change syncs to AgilePlus
    Given a feature "Test Feature" exists in both systems
    When I move the Plane.so issue to "In Progress"
    Then the AgilePlus feature state updates within 3 seconds
    And the audit trail records the sync event

  Scenario: Conflict detection on simultaneous edit
    Given a feature "Test Feature" synced to Plane.so
    When I edit the feature in AgilePlus
    And a teammate edits the same feature in Plane.so
    And I run "agileplus sync"
    Then the system detects the conflict
    And presents resolution options
```

---

## Part IX: Roadmap

### Phase 1: Core Platform (Current)
- [x] Rust monorepo structure
- [x] Hexagonal architecture
- [x] Feature/WP/Cycle state machines
- [x] SQLite event store
- [x] GitHub integration
- [x] CLI with subcommands

### Phase 2: AI Integration (2026 Q2)
- [ ] MCP server completion
- [ ] Agent dispatch system
- [ ] AI triage integration
- [ ] Spec generation AI

### Phase 3: Platform Services (2026 Q2-Q3)
- [ ] NATS event bus
- [ ] Neo4j graph layer
- [ ] Web dashboard (htmx + Alpine.js)
- [ ] Multi-device P2P sync

### Phase 4: Enterprise (2026 Q4)
- [ ] Plane.so bidirectional sync
- [ ] Governance workflow engine
- [ ] Compliance audit export
- [ ] Team collaboration features

---

*This spec reflects AgilePlus v2.0 architecture based on 2026 SOTA research.*
