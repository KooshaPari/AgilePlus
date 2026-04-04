# AgilePlus

**Project management system with AI agent integration** — 24-crate Rust monorepo with hexagonal architecture, Python MCP server, and Plane.so/GitHub integration.

## Project Overview

AgilePlus is a full-stack project management platform built with:
- **Rust** (24 crates) — Core domain, storage, event sourcing, CLI, REST API
- **Python** — MCP server for AI agent integration
- **TypeScript** — pheno-cli, React components

## Key Features

- Domain model: Feature, WorkPackage, Cycle, Module with state machines
- Event sourcing with audit trails and hash chains
- SQLite storage with hexagonal adapter pattern
- gRPC protocol definitions
- REST API with API key authentication
- OpenTelemetry tracing and metrics
- Git VCS adapter integration
- Plane.so sync (push/pull)
- GitHub integration

## Quick Start

```bash
# Setup
cd AgilePlus
bun install
cargo build --workspace

# Run CLI
cargo run --package pheno-cli -- --help

# Start REST server
cargo run --package pheno-cli -- serve

# Run tests
cargo test --workspace
```

## Documentation

| Document | Purpose |
|----------|---------|
| [PLAN.md](./PLAN.md) | Implementation phases and task tracking |
| [PRD.md](./PRD.md) | Product requirements and user journeys |
| [FUNCTIONAL_REQUIREMENTS.md](./FUNCTIONAL_REQUIREMENTS.md) | Detailed FR traceability |
| [AGENTS.md](./AGENTS.md) | Agent interaction rules |
| [GOVERNANCE.md](./GOVERNANCE.md) | Project governance |

## Architecture

```
AgilePlus/
├── crates/          # 24 Rust crates (workspace)
├── python/          # Python MCP server
├── pheno-cli/       # CLI tool
├── kitty-specs/     # Feature specifications
├── docs/            # Documentation
└── harnesses/       # Agent harness configs
```

## Traceability

`/// @trace FR-XXX-NNN` — All tests trace to functional requirements.