# ADR-001: Hexagonal Architecture for AgilePlus

## Status
**Accepted** — Implemented across all 24 crates

## Context

AgilePlus is a 24-crate Rust monorepo with multiple interface types (CLI, API, gRPC, MCP server) and multiple storage backends (SQLite, Git, Plane.so, GitHub). We needed an architecture pattern that would:

1. **Enable testing without external dependencies** — CI/CD must run without database
2. **Allow swapping implementations** — SQLite today, perhaps PostgreSQL tomorrow
3. **Maintain consistent domain logic** — Same rules regardless of interface
4. **Support multiple interfaces** — CLI, REST, gRPC, MCP all use same core

### Alternatives Considered

| Pattern | Pros | Cons | Verdict |
|---------|------|------|---------|
| **Layered (MVC)** | Simple, familiar | Business logic leaks between layers | ❌ Rejected |
| **Microservices** | Independent scaling | Operational complexity, network latency | ❌ Rejected |
| **Clean Architecture** | Testability, boundaries | Ceremony overhead | ⚠️ Partial |
| **Hexagonal (Ports & Adapters)** | Testability, flexibility, clear boundaries | Learning curve | ✅ Selected |
| **Event Sourcing (full)** | Complete audit trail | Complexity, CQRS overhead | ⚠️ Hybrid |

## Decision

We will implement **Hexagonal Architecture** (Ports & Adapters) with the following structure:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         DRIVING ADAPTERS                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐            │
│  │   CLI    │  │  REST    │  │   gRPC   │  │   MCP    │            │
│  │ Adapter  │  │ Adapter  │  │ Adapter  │  │ Server   │            │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘            │
│       │             │             │             │                  │
│       └─────────────┴─────────────┴─────────────┘                  │
│                         │                                            │
│                         ▼                                            │
├─────────────────────────────────────────────────────────────────────┤
│                         DOMAIN LAYER                                 │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Application Services                      │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │   │
│  │  │ Feature  │  │   Work   │  │  Cycle   │  │  Agent   │  │   │
│  │  │ Service  │  │ Package  │  │ Service  │  │ Dispatch │  │   │
│  │  └──────────┘  │ Service  │  └──────────┘  └──────────┘  │   │
│  │                └──────────┘                                │   │
│  │                                                             │   │
│  │  ┌────────────────────────────────────────────────────────┐ │   │
│  │  │              Domain Model (Entities, Value Objects)       │ │   │
│  │  │  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐     │ │   │
│  │  │  │Feature │  │  Work  │  │  Cycle │  │  Agent │     │ │   │
│  │  │  │Entity  │  │Package │  │Entity  │  │Entity  │     │ │   │
│  │  │  └────────┘  └────────┘  └────────┘  └────────┘     │ │   │
│  │  │                                                         │ │   │
│  │  │  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐     │ │   │
│  │  │  │ Feature│  │  Work  │  │  Cycle │  │  Agent │     │ │   │
│  │  │  │ State  │  │Package │  │ State  │  │ State  │     │ │   │
│  │  │  │Machine │  │State   │  │Machine │  │Machine │     │ │   │
│  │  │  └────────┘  └────────┘  └────────┘  └────────┘     │ │   │
│  │  └────────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                         │                                            │
│                         ▼                                            │
├─────────────────────────────────────────────────────────────────────┤
│                      DRIVEN ADAPTERS                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐            │
│  │  SQLite  │  │   Git    │  │  Plane   │  │  GitHub  │            │
│  │ Adapter  │  │ Adapter  │  │ Adapter  │  │ Adapter  │            │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘            │
│                                                                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐            │
│  │   NATS   │  │   Neo4j  │  │   MinIO  │  │  Cache   │            │
│  │ Adapter  │  │ Adapter  │  │ Adapter  │  │ Adapter  │            │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘            │
└─────────────────────────────────────────────────────────────────────┘
```

### Port Definitions

Each domain entity has a corresponding port trait:

```rust
// Port: FeatureRepository
#[async_trait]
pub trait FeatureRepository: Send + Sync {
    async fn get(&self, id: &FeatureId) -> Result<Feature, RepositoryError>;
    async fn list(&self, query: FeatureQuery) -> Result<Vec<Feature>, RepositoryError>;
    async fn save(&self, feature: &Feature) -> Result<(), RepositoryError>;
    async fn delete(&self, id: &FeatureId) -> Result<(), RepositoryError>;
    async fn exists(&self, id: &FeatureId) -> Result<bool, RepositoryError>;
}

// Port: EventStore
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, stream_id: &str, events: &[DomainEvent]) -> Result<u64, EventStoreError>;
    async fn read(&self, stream_id: &str, from_version: u64) -> Result<Vec<DomainEvent>, EventStoreError>;
    async fn get_snapshot(&self, stream_id: &str) -> Result<Option<Snapshot>, EventStoreError>;
    async fn save_snapshot(&self, snapshot: &Snapshot) -> Result<(), EventStoreError>;
}

// Port: SyncAdapter
#[async_trait]
pub trait SyncAdapter: Send + Sync {
    async fn push(&self, entity: &SyncEntity) -> Result<SyncResult, SyncError>;
    async fn pull(&self, since: Timestamp) -> Result<Vec<SyncEntity>, SyncError>;
    async fn resolve_conflict(&self, local: &SyncEntity, remote: &SyncEntity) -> Result<SyncEntity, SyncError>;
}
```

## Consequences

### Positive

1. **Testability**: Domain logic tests use in-memory adapters, no Docker required
2. **Flexibility**: SQLite → PostgreSQL migration requires only adapter changes
3. **Clarity**: Dependencies point inward; domain has no external dependencies
4. **Parallel Development**: CLI and API teams work independently via shared domain

### Negative

1. **Indirection Overhead**: Additional trait layers vs direct calls
2. **Learning Curve**: Team must understand port/adapter concepts
3. **Boilerplate**: Each adapter requires trait implementation

### Mitigations

- **Code Generation**: `agileplus-codegen` crate generates adapter stubs
- **Documentation**: Comprehensive examples in `agileplus-examples`
- **Testing Utilities**: `agileplus-test-helpers` provides mock adapters

## Implementation

### Crate Structure

```
agileplus/
├── crates/
│   ├── agileplus-domain/          # Domain layer (ports + entities)
│   │   ├── src/
│   │   │   ├── entities/          # Feature, WorkPackage, Cycle, Agent
│   │   │   ├── value_objects/     # FeatureId, Priority, Timestamp
│   │   │   ├── state_machines/    # FeatureStateMachine, etc.
│   │   │   └── ports/             # Repository traits
│   │   └── Cargo.toml
│   │
│   ├── agileplus-application/     # Application services
│   │   ├── src/
│   │   │   ├── services/          # Use case implementations
│   │   │   └── commands/          # Command handlers
│   │   └── Cargo.toml
│   │
│   ├── agileplus-sqlite/          # Driven adapter: SQLite
│   ├── agileplus-git/             # Driven adapter: Git
│   ├── agileplus-plane/           # Driven adapter: Plane.so
│   ├── agileplus-github/          # Driven adapter: GitHub
│   ├── agileplus-nats/            # Driven adapter: NATS
│   ├── agileplus-neo4j/           # Driven adapter: Neo4j
│   ├── agileplus-cache/           # Driven adapter: In-memory cache
│   │
│   ├── agileplus-cli/             # Driving adapter: CLI
│   ├── agileplus-api/             # Driving adapter: REST API
│   ├── agileplus-grpc/            # Driving adapter: gRPC
│   └── agileplus-mcp/             # Driving adapter: MCP Server
```

### Dependency Rules

```
Domain ───────┐
              │
Application ──┤── NO external dependencies
              │    (only std lib + domain)
Driving ──────┤
Adapters      │── Can depend on domain
              │    Can use external crates
Driven ───────┘    (tokio, axum, sqlx, etc.)
```

## Related Decisions

- **ADR-002**: Event Sourcing with SQLite for audit trail
- **ADR-003**: Rust for performance and safety

## Notes

- Pattern inspired by Alistair Cockburn's original hexagonal architecture paper
- Influenced by "Implementing Domain-Driven Design" by Vaughn Vernon
- Hexagonal crates provided by `hexagonal-rs` internal library

---

*Proposed: 2025-01-15*  
*Accepted: 2025-01-20*  
*Implemented: 2025-02-01*
