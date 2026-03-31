# AgilePlus — Claude Code Context

## Project

**AgilePlus** is a spec-driven development engine built in Rust (Tokio/Axum/Tonic), with a Python FastMCP bridge and Go CLI stub. It manages the full feature lifecycle from specification to shipping, with bidirectional Plane.so sync, event sourcing, and a hexagonal architecture.

## Stack

- **Rust** (primary): Axum 0.8 HTTP, Tonic 0.13 gRPC, rusqlite, redis/Dragonfly, neo4rs, async-nats
- **Python**: FastMCP (`agileplus-mcp/`) — MCP protocol bridge
- **Go**: Thin CLI stub (`pheno-cli/`)
- **Templates**: Askama (server-side HTML), htmx + Alpine.js (interactivity)

## Key Paths

```
crates/                    # 22 Rust crates (most commented out — uncomment when implementing)
libs/                      # 8 active shared libraries
agileplus-mcp/             # Python FastMCP server
kitty-specs/               # Feature specs (source of truth)
  003-agileplus-platform-completion/tasks/  # WP01-WP21 active tasks
.agileplus/
  agileplus.db             # SQLite state database (use this for memory/state)
  kilo-sync.json           # Gastown convoy tracking
```

## Build & Quality Gates

```bash
cargo check --workspace      # Must pass
cargo test --workspace       # Must pass
cargo clippy -- -D warnings  # Must pass before any PR
cargo fmt --all              # Keep formatting clean
```

## Sub-Agent Routing

When invoked via slash commands:

- `/spec` → Run the AgilePlus `specify` command to create or update a kitty-spec. Spec files go in `kitty-specs/<NNN>-<name>/`. Always create the spec before implementing.
- `/plan` → Enter planning mode: analyze the relevant kitty-spec WP task file, identify dependencies, draft implementation steps, check which crates are commented out in `Cargo.toml`.
- `/implement` → Implementation mode: execute the planned WP task. Follow polecat rules (uncomment Cargo.toml, write tests, run clippy).

## Memory & State

- Use `.agileplus/agileplus.db` for persistent session state and cross-conversation memory
- Spec progress is tracked in `kitty-specs/` — check `tasks/` directories for TODO/DONE markers
- `evidence_ledger.jsonl` at root tracks implementation evidence

## Architecture Notes

- **Hexagonal**: Domain crate (`agileplus-domain`) must never import infrastructure crates
- **Event-Sourced**: All mutations go through `EventStore::append()` — no direct DB writes for entities
- **CQRS**: Commands → event append; Queries → read from projections/cache
- **Hash chains**: SHA-256 over `{prev_hash}:{entity_id}:{sequence}:{event_type}:{payload}` — immutable

## Governance

- Every change must reference a kitty-spec WP task ID (e.g., `WP01`, `WP03`)
- Check `kitty-specs/003-agileplus-platform-completion/tasks/` for active work packages
- Never modify domain entities without a spec update

## Gastown Integration

- **Town**: `78a8d430-a206-4a25-96c0-5cd9f5caf984`
- **Rig**: `297c736c-f6b1-43c7-8167-db647ae94c53`
- **Active convoy**: `5c87e234-87df-4c1e-8b1a-7a697ce67706` (Spec 003 platform completion)
- **Methodology**: `agileplus+kilo-parallel`
