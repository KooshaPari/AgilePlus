# ADR-019: Local-First SQLite State Only

## Status
Proposed

## Context
AgilePlus is fundamentally a client-side tool: a semantic process engine for humans and agents. All other tools (CLI, desktop, API, MCP) are interfaces to the same local state.

Evidence from architecture and history:
- `crates/agileplus-sqlite` provides SQLite persistence via `rusqlite` (bundled, zero external deps)
- `.agileplus/agileplus.db` is the single source of truth for all features, work packages, tasks, audit trails
- The CLI works standalone with SQLite — no platform services needed
- User confirmed: "AgilePlus is a CLI, a semantic process, an app, a tray app — not a platform"
- User confirmed: "State lives in repo and device; Tracera is the platform"
- Electrobun desktop client (`desktop/`) uses Electron-bun with local repo access — designed for offline-first operation

The platform services (NATS, Dragonfly, Neo4j, MinIO, API) belong to Tracera (the platform), not AgilePlus (the client).

## Decision
AgilePlus state MUST live in `.agileplus/agileplus.db` (SQLite) only. No external services. No platform dependencies. Every interface (CLI, desktop, MCP) reads and writes the same SQLite database.

### `.agileplus/` directory structure (target):
```
.agileplus/
├── agileplus.db           # SQLite database (single source of truth)
├── agileplus.db-shm       # SQLite WAL shared memory
├── agileplus.db-wal       # SQLite WAL log
├── config.toml            # Client configuration (agent model, paths, etc.)
├── audit/                 # Audit trail exports (optional)
│   └── chain.jsonl
├── plugins/               # Local plugin extensions
└── state/                 # Derived/cached state (rebuildable from DB)
```

## Consequences

### Positive
- ✅ **VSCode-like architecture**: `.agileplus/` is to AgilePlus what `.vscode/` is to VS Code — local state, version-controllable
- ✅ **Zero external deps** for core operation
- ✅ **Portable**: Copy `.agileplus/` between machines, git clone it
- ✅ **Version-controllable**: `git add .agileplus/` (excluding `.db` lock files)
- ✅ **Offline-first**: Works without network, without services
- ✅ **Simple recovery**: Copy `.db` file, done
- ✅ **MCP clean**: gRPC core can use SQLite directly, no service hops
- ✅ **Desktop clean**: Electrobun reads `.db` directly via rusqlite FFI or HTTP proxy

### Negative
- ❌ Multi-device sync requires git or external sync (not handled by AgilePlus)
- ❌ Graph traces (Neo4j) and artifact storage (MinIO) need separate infrastructure if needed
- ❌ Concurrent writes across processes need SQLite WAL discipline

## Implementation Plan
1. **Document** the `.agileplus/` layout as canonical
2. **Enforce** `agileplus --db .agileplus/agileplus.db` as the only state path for all interfaces
3. **Add** config file support (`config.toml`) in `.agileplus/`
4. **Exclude** `.db-shm`, `.db-wal` from git via `.gitignore`
5. **Add** `.agileplus/` to `.gitignore` except for versionable artifacts (`config.toml`, audit exports)

## Related
- ADR-018: Remove Platform Services from AgilePlus Core
- `crates/agileplus-sqlite/` — the SQLite adapter
- `desktop/` — Electrobun client, must read `.agileplus/agileplus.db`
