# Plan: Remove Platform Services from AgilePlus Core

**Date:** 2026-09-03  
**Status:** Proposed  
**Owner:** [assignee]  
**Affected Crates:** `agileplus-cli`, `agileplus-mcp`, `agileplus-grpc`, `agileplus-domain`  
**Verification:** `./target/release/agileplus --db .agileplus/agileplus.db dashboard` works without platform services  

---

## Background

AgilePlus is a client-side, VSCode-like spec-driven development engine with state in `.agileplus/agileplus.db` (SQLite). Platform services (NATS, Dragonfly/Redis, Neo4j, MinIO, API) were added as optional extensions but have become hard dependencies in the gRPC core, breaking MCP functionality and contradicting the local-first architecture.

**Evidence:**
- CLI works standalone: `./target/release/agileplus --db .agileplus/agileplus.db dashboard` returns feature data
- MCP server fails: gRPC core timeouts waiting for platform services
- User confirmed: "AgilePlus is a CLI, a semantic process, an app, a tray app — NOT a platform. State lives in repo and device. Tracera is the platform."
- Research confirms: No competing spec-driven tool uses external services as core dependencies

---

## Scope

### In Scope
1. Remove `process-compose.yml` from repo root
2. Remove `platform` subcommand from CLI
3. Update MCP to use SQLite directly (no service deps)
4. Remove service dependencies from domain layer
5. Verify all interfaces work with SQLite only

### Out of Scope
- Building the full desktop client (separate work item)
- Completing the MCP tool surface (separate work item)
- Platform service implementations (Tracera's concern)

---

## Implementation Phases

### Phase 1: Remove Platform Infrastructure (Safe, Low Risk)

**Work Package 1.1: Remove process-compose.yml**
```bash
# Files to delete:
AgilePlus/process-compose.yml
AgilePlus/.deploy/process-compose.yml  # if exists

# Verification:
git ls-files | grep process-compose  # should return nothing
```

**Work Package 1.2: Remove platform subcommand from CLI**
```bash
# Files to modify:
crates/agileplus-subcmds/src/platform/  # entire directory
crates/agileplus-subcmds/src/lib.rs       # remove platform module
crates/agileplus-cli/src/lib.rs           # remove platform from commands

# Verification:
./target/release/agileplus platform status  # "unrecognized subcommand"
./target/release/agileplus platform up     # "unrecognized subcommand"
```

**Work Package 1.3: Remove platform references from Cargo.toml**
```toml
# Remove from workspace dependencies if any reference to:
# - nats, dragonfly, neo4j, minio (these shouldn't exist in core anyway)
# Confirm: grep -r "nats\|dragonfly\|neo4j\|minio" Cargo.toml  # should return nothing
```

### Phase 2: Update MCP to SQLite-Only (Medium Risk)

**Work Package 2.1: Update agileplus-mcp to use SQLite directly**
```python
# Files to modify:
agileplus-mcp/src/server.py           # remove service health checks
agileplus-mcp/src/grpc_client.py      # update to use SQLite adapter
agileplus-mcp/src/tools/              # update all tools to use SQLite

# Verification:
# MCP health check passes without platform services
# `get_feature`, `list_features` work via SQLite
```

**Work Package 2.2: Remove gRPC service dependencies from agileplus-grpc**
```rust
// Files to modify:
crates/agileplus-grpc/src/server.rs    # remove service channel deps
crates/agileplus-domain/src/domain/    # remove ServiceHealth, PlatformStatus

// Keep:
# - Feature, WorkPackage, Task domain models
# - SQLite storage port
# - Governance logic
```

**Work Package 2.3: Update agileplus-grpc daemon**
```bash
# Files to modify:
crates/agileplus-grpc/src/main.rs      # remove service startup
crates/agileplus-grpc/src/lib.rs       # simplify

# Verification:
# gRPC daemon starts without platform services
# MCP connects to gRPC successfully
```

### Phase 3: Verify All Interfaces (Required Before Merge)

**Work Package 3.1: CLI verification**
```bash
# All of these must work without platform services:
./target/release/agileplus --db .agileplus/agileplus.db dashboard
./target/release/agileplus --db .agileplus/agileplus.db list
./target/release/agileplus --db .agileplus/agileplus.db validate
./target/release/agileplus --db .agileplus/agileplus.db cockpit
```

**Work Package 3.2: MCP verification**
```bash
# Health check passes:
curl -s http://127.0.0.1:8765/mcp -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"health_check","arguments":{}},"id":1}'
# Expected: {"result":{"healthy":true,...}}
```

**Work Package 3.3: Desktop verification**
```bash
cd desktop/
bun run dev
# Desktop app launches, reads .agileplus/agileplus.db, shows features
```

---

## WBS (Work Breakdown Structure)

```
REM-PLATFORM-SERVICES
├── 1. Remove platform infrastructure
│   ├── 1.1 Delete process-compose.yml
│   ├── 1.2 Remove platform subcommand
│   └── 1.3 Clean Cargo.toml
├── 2. Update MCP to SQLite-only
│   ├── 2.1 Update agileplus-mcp server
│   ├── 2.2 Remove gRPC service deps
│   └── 2.3 Simplify agileplus-grpc daemon
├── 3. Verification
│   ├── 3.1 CLI verification
│   ├── 3.2 MCP verification
│   └── 3.3 Desktop verification
└── 4. Documentation
    ├── 4.1 Update ARCHITECTURE.md
    ├── 4.2 Update local-first-deployment.md
    └── 4.3 Add ADR-018, ADR-019, ADR-020
```

---

## Dependencies

- **Blocked by:** None — this is the first step in cleaning up AgilePlus
- **Blocks:** MCP completion, desktop client completion, full feature parity

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| MCP tools break during refactor | Medium | High | Extensive verification after each step |
| Desktop client breaks | Low | Medium | Test desktop separately |
| Governance logic depends on services | Low | Medium | Domain logic review |
| Git history of platform files needed | Low | Low | Keep git history, just delete files |

---

## Time Estimate

- **Phase 1:** 2-4 hours (straight deletion)
- **Phase 2:** 4-8 hours (gRPC + MCP refactor)
- **Phase 3:** 2-4 hours (verification)
- **Total:** 8-16 hours

---

## Next Steps

1. **Approval:** Get explicit approval before starting
2. **Branch:** Create `fix/remove-platform-services-20260903`
3. **Execute:** Phase 1 → Phase 2 → Phase 3 in sequence
4. **PR:** Open PR with all changes, verification evidence
5. **Merge:** After review, merge to main
