# ADR-018: Remove Platform Services from AgilePlus Core

## Status
Accepted (partially implemented)

## Context
AgilePlus was architected as a client-side, device-level spec-driven development system with state living in `.agileplus/agileplus.db` (SQLite). However, the original implementation included platform services (NATS, Dragonfly/Neo4j/MinIO/API) as core dependencies via `process-compose.yml` and the `agileplus platform` subcommand.

These services created unnecessary complexity for the core client-side use case:
- Required external dependencies (Docker, nats-server, redis-server, minio, neo4j)
- Complicated local development and CI
- Introduced failure modes (timeouts, port conflicts) that block core functionality
- Contradicted the local-first, VSCode-like architecture principle

**Note:** The original claim that "MCP server and gRPC core have hard dependencies on these services" was inaccurate. The MCP server (`agileplus-mcp`) already uses SQLite directly via `StoragePort`. The agent dispatch (`agileplus-agents`) does not depend on platform services. Only the REST API (`agileplus-api`) and the `agileplus platform` subcommand depend on external services.

## Decision
Remove platform services (NATS, Dragonfly, Neo4j, MinIO, API) from AgilePlus core. AgilePlus will function purely as a client-side SQLite-backed system with:
- Core CLI (`agileplus` command) working with only `.agileplus/agileplus.db`
- MCP server updated to use SQLite directly (no service dependencies)
- Desktop client (Electrobun) reading/writing the SQLite file directly
- Platform services moved to optional extensions or separate repositories

## Consequences

### Positive
- ✅ Simplified local development: `cargo build --release` + `./target/release/agileplus` works immediately
- ✅ Reduced CI complexity: No Docker, no external service setup
- ✅ Improved reliability: No timeouts, no port conflicts, no service failures
- ✅ Aligns with VSCode-like architecture: State in `.agileplus/`, no external deps
- ✅ Faster startup: No waiting for service health checks
- ✅ True offline-first: Works completely disconnected from network
- ✅ Clear separation: Core is client-only, platform features are opt-in

### Negative
- ❌ Loss of optional features: Event streaming (NATS), caching (Dragonfly), graph traces (Neo4j), artifact storage (MinIO) require manual re-addition
- ❌ MCP server changes needed: Must remove service dependencies from gRPC core/domain layer
- ❌ Documentation updates: Remove platform references from guides
- ❌ Potential duplication: If platform features are needed elsewhere, they may be reimplemented

## Implementation Plan
1. ✅ **Remove** `process-compose.yml` from repo root — completed
2. ✅ **Update** `agileplus-mcp` to use SQLite storage directly — completed (tools use `StoragePort`)
3. ✅ **Verify** CLI works without platform services — completed
4. ❌ **Remove** `platform` subcommand from CLI (`crates/agileplus-subcmds/src/platform/`) — still present
5. ❌ **Update** documentation to remove platform references — partially done
6. ❌ **Extract** optional platform extension repo if needed — not started

The core requirement (CLI + MCP working with SQLite only) is satisfied. The `platform` subcommand is legacy infrastructure that can be removed in a follow-up.

## Related
- ADR-019: Local-First SQLite State Only
- ADR-020: Rust Core + Electrobun Desktop as Primary Interface
- Research: `/docs/research/agent-spec-alternatives-2026-09-03.md`

## Acceptance Criteria
- [x] `./target/release/agileplus dashboard` works without any platform services running
- [x] MCP server tools work with only SQLite (no service dependencies)
- [x] `process-compose.yml` deleted from repo
- [ ] `agileplus platform` command removed (returns "unrecognized subcommand")
- [ ] Desktop client can read/write `.agileplus/agileplus.db` directly (see ADR-020 for Tauri migration)
