# AgilePlus — Gastown Polecat Instructions

## Project

**AgilePlus** is a spec-driven development engine that bridges software engineering workflows with structured project management. It orchestrates feature lifecycle from specification through validation, with bidirectional sync to Plane.so and full event-sourced audit trails.

## Stack

- **Rust** (primary): Tokio async runtime, Axum 0.8 HTTP, Tonic 0.13 gRPC, rusqlite (SQLite), redis/bb8-redis (Dragonfly), neo4rs (Neo4j), async-nats (NATS JetStream)
- **Python**: FastMCP server (`agileplus-mcp/`) — MCP protocol bridge for Claude/Gemini
- **Go**: CLI stub (`pheno-cli/`) — thin wrapper delegating to Rust binary

## Architecture

- **Hexagonal Architecture**: Domain → Application → Ports → Adapters (no domain deps on infrastructure)
- **Event-Sourced**: All mutations create immutable `Event` records with SHA-256 hash chains
- **CQRS**: Commands mutate state via event append; queries read projections (SQLite views or cache)
- **gRPC contracts**: Protobuf definitions in `buf.yaml`; generated with `buf generate`

## Key Paths

```
crates/                    # 22 Rust crates (most commented out in Cargo.toml — implement before uncommenting)
  agileplus-domain/        # Core domain entities (Event, Snapshot, SyncMapping, ServiceHealth, DeviceNode, ApiKey)
  agileplus-events/        # Event store trait + InMemory/SQLite implementations
  agileplus-sqlite/        # SQLite adapter (WAL mode, migrations, all tables)
  agileplus-cache/         # Dragonfly/Redis cache layer (CacheStore trait)
  agileplus-graph/         # Neo4j graph layer (GraphStore trait)
  agileplus-plane/         # Plane.so API client + webhook handler
  agileplus-sync/          # Sync orchestrator (push/pull/conflict detection)
  agileplus-api/           # Axum REST API + SSE endpoint
  agileplus-dashboard/     # Askama/htmx/Alpine.js web dashboard
  agileplus-subcmds/       # CLI subcommand handlers
  agileplus-grpc/          # Tonic gRPC service implementations
  agileplus-nats/          # NATS JetStream publisher/subscriber
  agileplus-telemetry/     # OpenTelemetry tracing + metrics
  agileplus-cli/           # Main CLI binary (clap v4)
  agileplus-git/           # Git operations (gix + git2)
  agileplus-github/        # GitHub API client
  agileplus-p2p/           # P2P device sync (CRDTs)
  agileplus-triage/        # AI triage helpers
  agileplus-import/        # Import from external systems
  agileplus-benchmarks/    # Criterion benchmarks
  agileplus-contract-tests/# Contract/API tests
  agileplus-integration-tests/ # Integration test suite

libs/                      # 8 active shared libraries
  nexus/                   # Core nexus library
  plugin-registry/         # Plugin registration and lifecycle
  plugin-sample/           # Sample plugin implementation
  plugin-cli/              # CLI plugin interface
  plugin-git/              # Git plugin
  plugin-grpc/             # gRPC plugin interface
  plugin-integration/      # Integration plugin
  intent-registry/         # Intent-based coordination registry

agileplus-mcp/             # Python FastMCP server (MCP protocol bridge)
agileplus-agents/          # Agent configurations
kitty-specs/               # Feature specifications (source of truth for all work)
  003-agileplus-platform-completion/  # Active convoy specs
    tasks/                 # WP01-WP21 task files

.agileplus/                # Runtime state directory
  agileplus.db             # SQLite database (runtime state)
  specs/                   # Spec snapshots
  kilo-sync.json           # Gastown sync tracking

apps/                      # Application entry points
docs/                      # VitePress documentation
scripts/                   # Build and utility scripts
```

## Build Commands

```bash
# Check workspace compiles (MUST pass before any PR)
cargo check --workspace

# Run all tests
cargo test --workspace

# Check single crate (faster iteration)
cargo check -p <crate-name>
cargo test -p <crate-name>

# Lint (MUST pass before PR)
cargo clippy -- -D warnings

# Format
cargo fmt --all

# Protobuf generation
buf generate
```

## Active Convoy

**Spec 003: AgilePlus Platform Completion**
- Branch: `convoy/agileplus-platform-completion-spec-003/5c87e234/head`
- Work packages WP01–WP21 in progress (see `kitty-specs/003-agileplus-platform-completion/tasks/`)
- WP dependencies: WP01 (domain) → WP02 (events) → WP03 (sqlite) → WP04/WP05/WP06 → WP08/WP09 → WP10/WP11 → WP12/WP13

## Governance

- **Every PR must reference a kitty-spec WP task ID** in the commit message or PR description (e.g., `WP03: implement SQLite schema migrations`)
- Specs in `kitty-specs/` are the authoritative source of truth — implementation must match spec
- `CONSTITUTION.yaml` equivalent: the spec files define behavior contracts

## Polecat Rules

1. **Never break workspace compilation.** `cargo check --workspace` must always pass on your branch before pushing.
2. **Always uncomment the crate in `Cargo.toml`** when implementing a crate. Crates are commented out until they have a working `src/lib.rs`.
3. **Tests required for all new crates.** `cargo test -p <crate>` must pass with meaningful tests (not just empty stubs).
4. **Domain entities are contracts.** Do not modify `agileplus-domain` entities without a corresponding spec update.
5. **Hexagonal boundaries.** Domain crate must have no infrastructure dependencies (no tokio runtime in domain, no DB calls).
6. **Hash chains are immutable.** Never modify the `Event` hash chain computation algorithm once established.
7. **Run `cargo clippy -- -D warnings`** before marking work done.
8. **One crate per WP (roughly).** Keep PRs focused on a single work package.

## Gastown Integration

- **Town**: `78a8d430-a206-4a25-96c0-5cd9f5caf984`
- **Rig**: `297c736c-f6b1-43c7-8167-db647ae94c53`
- **Active convoy ID**: `5c87e234-87df-4c1e-8b1a-7a697ce67706`
- **Methodology**: `agileplus+kilo-parallel`
- **Spec directory**: `kitty-specs/`
