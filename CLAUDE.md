# CLAUDE.md — AgilePlus

## Identity

AgilePlus is a Rust monorepo implementing a next-generation project management system with hexagonal architecture, event sourcing, and multi-VCS support.

**Repository:** https://github.com/KooshaPari/AgilePlus
**Architecture:** 24-crate Rust workspace with hexagonal/ports-and-adapters pattern
**Stack:** Rust (Axum, Tonic/gRPC, SQLite), Python MCP server
**Integrations:** Plane.so, GitHub, NATS

## Project Structure

```
AgilePlus/
├── crates/              # Core Rust domain crates (currently commented out in workspace)
├── libs/                # Active Rust libraries
│   ├── nexus/          # Core messaging/coordination
│   ├── intent-registry/
│   ├── health-monitor/
│   └── plugin-*/       # Plugin implementations (git, grpc, sqlite, etc.)
├── agileplus/          # Python MCP server for AgilePlus
├── agileplus-agents/   # Agent tooling
├── agileplus-mcp/      # MCP protocol implementation
├── apps/               # Application entry points
├── docs/               # Documentation
│   ├── agents/         # Agent governance and prompts
│   ├── adr/            # Architecture decision records
│   ├── specs/          # Feature specifications
│   └── guides/         # How-to guides
├── python/             # Python packages
├── proto/              # gRPC protocol definitions
└── tests/             # Integration tests
```

## Key Concepts

- **Work Packages (WPs):** Atomic units of work with explicit file scopes
- **Event Sourcing:** All state changes recorded as immutable events with hash chains
- **Hexagonal Architecture:** Domain logic isolated from infrastructure via ports
- **Plugin System:** Storage and VCS adapters loaded dynamically
- **Evidence Ledger:** Audit trail for governance compliance

## Development Commands

```bash
# Build the workspace
cargo build

# Run tests
cargo test

# Run clippy lints
cargo clippy --all

# Format code
cargo fmt --all

# Build gRPC stubs (requires buf)
buf generate

# Run the Python MCP server
cd agileplus && python -m agileplus
```

## Documentation

| What you need | Where to look |
|---------------|---------------|
| Agent rules | `docs/agents/governance-constraints.md` |
| Agent prompts | `docs/agents/prompt-format.md` |
| Architecture decisions | `docs/adr/` |
| Feature specifications | `docs/specs/` |
| Implementation plan | `PLAN.md` |
| Functional requirements | `FUNCTIONAL_REQUIREMENTS.md` |

## Agent Workflow

1. Read `docs/agents/governance-constraints.md` for safety rules
2. Read `docs/agents/prompt-format.md` for work package format
3. Work in isolated git worktrees (never commit directly to main)
4. Follow spec-driven development: implement exactly what spec says
5. All changes must be traceable to a work package

## Workspace Status

Many crates are currently commented out in `Cargo.toml` pending implementation:
- agileplus-domain, agileplus-cli, agileplus-api, agileplus-grpc
- agileplus-sqlite, agileplus-git, agileplus-plane, agileplus-telemetry
- And others...

Active development happens in `libs/` and `agileplus/` directories.
