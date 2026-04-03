# Phase 2.1: Repo Consolidation Plan

**Date:** 2026-04-02  
**Agent:** Forge  
**Scope:** Repository consolidation across the repos shelf

---

## Executive Summary

This document provides a detailed merge plan for consolidating 15 duplicate/split repositories into 8 target repositories. Analysis of the current shelf state shows **only 5 of the 15 merge pairs exist locally** - the remaining repos either don't exist or are not yet created.

### Existence Matrix

| # | Merge Pair | Source Exists | Target Exists | Status |
|---|------------|---------------|---------------|--------|
| 1 | phenotype-contract → phenotype-contracts | ❌ No | ❌ No | **BLOCKED** - Neither repo exists |
| 2 | phenotype-error-* → phenotype-error-core | ❌ No | ❌ No | **BLOCKED** - Neither repo exists |
| 3 | phenotype-ports-* → phenotype-contracts | ❌ No | ❌ No | **BLOCKED** - Neither repo exists |
| 4 | thegent-plugin-host → thegent/apps/plugin-host | ✅ Yes | ✅ Yes | **READY** |
| 5 | forgecode-fork → forgecode | ✅ Yes | ✅ Yes | **READY** |
| 6 | hexagon-rust → hexagon-rs | ❌ No | ✅ Yes | **PARTIAL** - Source missing |
| 7 | agileplus-agents → AgilePlus/packages/agents | ✅ Yes | ✅ Yes | **READY** |
| 8 | agileplus-mcp → AgilePlus/packages/mcp | ✅ Yes | ✅ Yes | **READY** |
| 9 | router-docs → phenotype-hub/docs | ❌ No | ✅ Yes | **PARTIAL** - Source missing |
| 10 | FixitGo + FixitRs → fixit | ❌ No | ❌ No | **BLOCKED** - Neither repo exists |
| 11 | phenotype-config-loader → phenotype-config-core | ❌ No | ❌ No | **BLOCKED** - Neither repo exists |
| 12 | phenotype-shared-config → phenotype-config-core | ❌ No | ❌ No | **BLOCKED** - Neither repo exists |
| 13 | phenotype-async-traits → phenotype-contracts | ❌ No | ❌ No | **BLOCKED** - Neither repo exists |
| 14 | bifrost-routing* → bifrost | ❌ No | ✅ Yes | **PARTIAL** - Source missing |
| 15 | vibeproxy-monitoring-unified | ✅ Yes | N/A | **ARCHIVED** - Already done |

### Quick Stats
- **Ready to merge:** 4 pairs
- **Partial (target only):** 3 pairs  
- **Blocked (neither exists):** 8 pairs
- **Already archived:** 1

---

## High Priority Merges (Ready for Execution)

### Merge #1: thegent-plugin-host → thegent

**Priority:** HIGH  
**Effort:** Medium  
**Complexity:** Medium (architectural alignment required)

#### Source Analysis (thegent-plugin-host)
```
thegent-plugin-host/
├── Cargo.toml              # PrismRs crate, standalone
├── src/
│   ├── lib.rs              # Main lib exports (hexagonal architecture)
│   ├── main.rs             # CLI entry point
│   ├── domain/             # DDD: entities, events, value_objects
│   ├── application/        # Commands, queries, use_cases
│   ├── ports/              # Hexagonal: driven, driving ports
│   ├── adapters/           # WASM, dynamic, inmemory adapters
│   └── specs/              # SpecDD module
├── tests/
└── docs/
```

**Key Technical Details:**
- Crate name: `PrismRs`
- Architecture: Hexagonal (Ports & Adapters) + Clean Architecture
- Pattern: Plugin host and loader
- Dependencies: serde, tokio, thiserror, parking_lot, wasm support

#### Target Analysis (thegent)
```
thegent/
├── crates/                 # 28 existing crates in workspace
│   ├── thegent-resources
│   ├── thegent-parser
│   ├── thegent-shm
│   ├── thegent-runtime     # <-- May overlap!
│   └── ...
├── apps/
│   └── byteport/           # Target location: thegent/apps/plugin-host
└── Cargo.toml            # Workspace config
```

**Current thegent workspace members:**
- thegent-resources, thegent-parser, thegent-crypto, thegent-shm, thegent-git
- thegent-discovery, thegent-hooks, thegent-docs, thegent-utils, thegent-router
- thegent-maif, thegent-shims, thegent-zmx-interop, thegent-zmx, thegent-jsonl
- thegent-policy, thegent-metrics, thegent-fs, thegent-offload
- thegent-tui, thegent-memory, thegent-subprocess, harness-native
- thegent-cache (temporarily disabled), thegent-watcher (excluded)

#### Conflict Analysis
| Aspect | Conflict Risk | Notes |
|--------|---------------|-------|
| **Crate naming** | LOW | `thegent-plugin-host` doesn't exist in target yet |
| **Module overlap** | MEDIUM | `thegent-runtime` may have plugin-related code |
| **Architecture** | LOW | Both use Rust 2021, similar patterns |
| **Dependencies** | LOW | Compatible versions (tokio 1.x, serde 1.x) |
| **CLI binary** | MEDIUM | Potential binary name conflicts |

#### Merge Strategy

**Option A: Crate Integration (Recommended)**
```
thegent/
└── crates/
    └── thegent-plugin-host/      # New workspace member
        ├── Cargo.toml            # Rename package: thegent-plugin-host
        ├── src/
        └── tests/
```

**Steps:**
1. Move `thegent-plugin-host` → `thegent/crates/thegent-plugin-host`
2. Update `Cargo.toml`:
   - Change package name: `PrismRs` → `thegent-plugin-host`
   - Update repository URL
   - Keep version at 0.1.0 (new crate)
3. Add to workspace `Cargo.toml` members list
4. Merge `apps/` content:
   - If `thegent/apps/` doesn't exist, create it
   - Move binary/distribution code to `thegent/apps/plugin-host/`
5. Preserve hexagonal architecture structure
6. Update all internal imports
7. Run full workspace build: `cargo build --workspace`

**Files to Keep from Source:**
- All source files in `src/` (domain, application, ports, adapters)
- `tests/integration.rs`
- `benches/` if exists
- Documentation in `docs/`

**Files to Discard:**
- Standalone `.git/` (history preserved in merge commit)
- Duplicate CI configs (use thegent's `.github/`)
- Top-level README (merge into thegent's)

#### Migration Notes
- No breaking changes for external consumers (new crate name)
- Internal thegent crates can now depend on `thegent-plugin-host`
- CLI binary becomes `thegent-plugin-host` (from `PrismRs`)

---

### Merge #2: forgecode-fork → forgecode

**Priority:** HIGH  
**Effort:** Small  
**Complexity:** Low

#### Source Analysis (forgecode-fork)
```
forgecode-fork/
├── Cargo.toml              # Minimal workspace stub
├── src/
│   └── lib.rs              # Placeholder only
└── .github/
```

**Cargo.toml:**
```toml
[package]
name = "forgecode-fork"
version.workspace = true
# ... minimal deps
```

#### Target Analysis (forgecode)
```
forgecode/
├── forgecode-core/         # Rust core
├── config/                 # Git configs
├── scripts/                # Setup aliases, sync repos
├── skills/                 # Shared agent skills
├── docs/                   # Documentation
└── README.md               # Well-documented
```

**Key finding:** `forgecode` is primarily a **documentation/tooling repo**, not a Rust crate.
The Rust component (`forgecode-core`) is already inside.

#### Conflict Analysis
| Aspect | Conflict Risk | Notes |
|--------|---------------|-------|
| **Content** | NONE | forgecode-fork is essentially empty |
| **Purpose** | LOW | Both are worktree management tools |

#### Merge Strategy

**Recommendation: DELETE forgecode-fork**

**Rationale:**
- `forgecode-fork` has no meaningful code (just a stub)
- `forgecode` is fully functional with:
  - Complete SPEC.md with architecture
  - Scripts for worktree management
  - Git configuration templates
  - Documentation

**Steps if merge needed:**
1. Archive `forgecode-fork` (no content worth keeping)
2. Verify `forgecode` has all functionality
3. Update any references to `forgecode-fork` in docs

#### Migration Notes
- No breaking changes (deletion of empty repo)
- Verify no external references to `forgecode-fork`

---

### Merge #3: agileplus-agents → AgilePlus

**Priority:** HIGH  
**Effort:** Large  
**Complexity:** High (workspace integration, proto alignment)

#### Source Analysis (agileplus-agents)
```
agileplus-agents/
├── Cargo.toml              # Workspace with 3 crates
├── crates/
│   ├── agileplus-agent-dispatch/      # Spawns Claude/Codex
│   ├── agileplus-agent-review/        # CodeRabbit integration
│   └── agileplus-agent-service/       # gRPC server
└── .github/
```

**Workspace Dependencies:**
- tokio 1.x, tonic 0.12, prost 0.13
- dashmap, uuid, thiserror, anyhow
- tracing, async-trait

**Key Crates:**

1. **agileplus-agent-dispatch**
   - Purpose: AgentPort adapter (spawns subprocesses)
   - Features: PR loops, Codex/Claude Code adapters
   - Size: ~10 source files

2. **agileplus-agent-review**
   - Purpose: CodeRabbit integration, CI status
   - Features: Fallback mechanisms, integration tests
   - Size: ~5 source files

3. **agileplus-agent-service**
   - Purpose: gRPC server for agents
   - Features: Tonic-based, health checks
   - Size: ~3 source files + build.rs

#### Target Analysis (AgilePlus)
```
AgilePlus/
├── Cargo.toml              # Large workspace (45+ members)
├── crates/                 # 19 application crates
│   ├── agileplus-api
│   ├── agileplus-cli
│   ├── agileplus-grpc
│   └── ...
├── libs/                   # 18 library crates
│   ├── nexus
│   ├── plugin-registry
│   ├── plugin-cli
│   ├── plugin-git
│   ├── plugin-grpc
│   ├── plugin-integration
│   └── ...
└── packages/               # Target: AgilePlus/packages/
    # Does not exist yet!
```

**AgilePlus Workspace Dependencies:**
- tokio 1.x, tonic 0.13 (NEWER!), prost 0.13
- gix 0.71, git2 0.20
- rusqlite 0.32
- Already has plugin crates!

#### Conflict Analysis
| Aspect | Conflict Risk | Notes |
|--------|---------------|-------|
| **gRPC versions** | HIGH | agileplus-agents: tonic 0.12, AgilePlus: tonic 0.13 |
| **Workspace structure** | MEDIUM | Need to integrate 3 crates into existing workspace |
| **Plugin overlap** | HIGH | AgilePlus has plugin-* libs already |
| **Binary conflicts** | LOW | agent-service binary name unique |
| **Edition** | MEDIUM | agileplus-agents: 2024, AgilePlus: 2024 (OK) |

**CRITICAL VERSION MISMATCH:**
```
agileplus-agents: tonic = "0.12"
AgilePlus:        tonic = "0.13"
```

#### Merge Strategy

**Target Structure:**
```
AgilePlus/
└── packages/
    └── agents/               # NEW: from agileplus-agents
        ├── Cargo.toml        # Workspace or single crate?
        ├── crates/
        │   ├── agent-dispatch/
        │   ├── agent-review/
        │   └── agent-service/
        └── proto/            # Move .proto files here
```

**Decision: Flatten or Nest?**

Given AgilePlus already has `crates/` and `libs/`, the cleanest approach is:

**Option A: Add to existing structure (Recommended)**
```
AgilePlus/
├── crates/
│   ├── agileplus-agent-dispatch/    # NEW
│   ├── agileplus-agent-review/      # NEW
│   ├── agileplus-agent-service/     # NEW
│   └── ...existing crates...
```

**Steps:**
1. **Version Alignment (CRITICAL):**
   - Update agileplus-agents crates to tonic 0.13
   - Update prost to 0.13
   - Update tonic-build to 0.13
   - Test compilation

2. **Move crates:**
   - `agileplus-agents/crates/*` → `AgilePlus/crates/`
   - Update crate names if needed:
     - `agileplus-agent-dispatch` → `agileplus-agent-dispatch` (keep)
     - `agileplus-agent-review` → `agileplus-agent-review` (keep)
     - `agileplus-agent-service` → `agileplus-agent-service` (keep)

3. **Update AgilePlus/Cargo.toml:**
   ```toml
   members = [
       # ...existing...
       "crates/agileplus-agent-dispatch",
       "crates/agileplus-agent-review",
       "crates/agileplus-agent-service",
   ]
   ```

4. **Resolve plugin conflicts:**
   - Review `AgilePlus/libs/plugin-*` vs agent dispatch
   - May need to consolidate plugin abstractions

5. **Build and test:**
   ```bash
   cargo build --workspace
   cargo test --workspace
   ```

**Files to Keep:**
- All source code from 3 crates
- Integration tests
- Proto definitions (if any) - align with existing proto structure

**Files to Discard/Merge:**
- Standalone `Cargo.toml` (merge into workspace)
- Duplicate `.github/` configs
- Separate README (merge into AgilePlus docs)

#### Migration Notes
- **Breaking change**: Requires tonic 0.13 upgrade in source
- **Plugin consolidation**: May reveal duplication with existing plugin-* crates
- **gRPC alignment**: Ensures consistent proto handling across AgilePlus

---

### Merge #4: agileplus-mcp → AgilePlus

**Priority:** HIGH  
**Effort:** Medium  
**Complexity:** Medium (Python → Python integration)

#### Source Analysis (agileplus-mcp)
```
agileplus-mcp/
├── pyproject.toml          # Python package, FastMCP 3.0
├── src/
│   └── agileplus_mcp/
│       ├── __init__.py
│       ├── __main__.py
│       ├── server.py       # MCP server
│       ├── grpc_client.py  # gRPC bridge
│       ├── tools/          # MCP tools
│       ├── resources/      # MCP resources
│       ├── prompts/        # MCP prompts
│       └── sampling/       # MCP sampling
├── tests/
│   ├── unit/
│   ├── contract/
│   └── bdd/                # Behave tests
└── docs/
```

**pyproject.toml:**
```toml
[project]
name = "agileplus-mcp"
requires-python = ">=3.12"
dependencies = [
    "fastmcp>=3.0",
    "grpcio>=1.68",
    # ...
]
```

**Key Components:**
- FastMCP 3.0 server
- gRPC bridge to Rust core
- Tools: features, status, governance
- OpenTelemetry integration

#### Target Analysis (AgilePlus)
```
AgilePlus/
├── Cargo.toml              # Rust workspace
├── crates/                 # Rust crates
├── libs/                   # Rust libs
├── python/                 # Python packages?
├── packages/               # Target: NEW packages dir
└── ...
```

**CRITICAL FINDING:** AgilePlus is primarily a **Rust monorepo**. The Python integration needs careful placement.

Looking at existing structure:
- `AgilePlus/python/` - May exist (not in listing)
- `AgilePlus/packages/` - Does not exist yet
- `AgilePlus/src/` - Present but unclear purpose

#### Conflict Analysis
| Aspect | Conflict Risk | Notes |
|--------|---------------|-------|
| **Language mismatch** | MEDIUM | Python in Rust repo - needs clear location |
| **FastMCP version** | LOW | fastmcp>=3.0 is current |
| **gRPC versions** | MEDIUM | grpcio 1.68 vs Rust tonic 0.13 compatibility |
| **Build system** | MEDIUM | hatchling vs Rust cargo |

#### Merge Strategy

**Recommended Structure:**
```
AgilePlus/
└── packages/
    └── mcp/                  # NEW: Python package
        ├── pyproject.toml
        ├── src/
        │   └── agileplus_mcp/
        └── tests/
```

**Steps:**
1. Create `AgilePlus/packages/` directory
2. Move `agileplus-mcp/` → `AgilePlus/packages/mcp/`
3. Keep `pyproject.toml` mostly intact
4. Update package references:
   - Repository URL
   - Any internal path references
5. Ensure gRPC compatibility:
   - Verify proto files align with Rust tonic 0.13
   - Test Python grpcio 1.68 with Rust tonic 0.13
6. Add Python CI workflow to AgilePlus `.github/`

**Alternative (if Python already exists in AgilePlus):**
- If `AgilePlus/python/` exists, move to `AgilePlus/python/agileplus-mcp/`

**Files to Keep:**
- All Python source
- Tests (unit, contract, BDD)
- Documentation
- Proto definitions

**Files to Update:**
- `pyproject.toml`: repository URL, version alignment
- CI workflows: integrate into AgilePlus CI

#### Migration Notes
- Creates hybrid Rust/Python workspace
- Requires dual CI (cargo + Python pytest)
- gRPC proto files must be shared between Rust and Python

---

## Medium Priority Merges (Partial Existence)

### Merge #6: hexagon-rust → hexagon-rs

**Priority:** MEDIUM  
**Effort:** Unknown  
**Complexity:** Unknown  
**Status:** BLOCKED - Source repo missing

**Current state:**
- `hexagon-rs` exists (Rust implementation)
- `hexagon-rust` does not exist locally

**Analysis:** The name similarity suggests `hexagon-rust` might be:
1. A deprecated/renamed version that became `hexagon-rs`
2. An older implementation to be merged
3. A planned repo that was never created

**Recommendation:** Archive this merge plan until `hexagon-rust` is located or confirmed obsolete.

---

### Merge #9: router-docs → phenotype-hub/docs

**Priority:** MEDIUM  
**Effort:** Small  
**Complexity:** Low  
**Status:** BLOCKED - Source repo missing

**Current state:**
- `phenotype-hub` exists
- `router-docs` does not exist locally

**Analysis:** `phenotype-hub` is very small (11 entries). If `router-docs` exists elsewhere:
- Simple content merge
- Move docs to `phenotype-hub/docs/`

**Recommendation:** Search for `router-docs` in other locations or archives.

---

### Merge #14: bifrost-routing* → bifrost

**Priority:** MEDIUM  
**Effort:** Unknown  
**Complexity:** Unknown  
**Status:** BLOCKED - Source repos missing

**Current state:**
- `bifrost` exists (very small: 4 entries)
- `bifrost-routing` and `bifrost-routing-backup` do not exist

**Analysis:** The small size of `bifrost` suggests it may be a stub. The routing-specific repos likely contain the actual implementation.

**Recommendation:** Search for routing repos or check if `bifrost-extensions` (exists, 90 entries) contains this code.

---

## Blocked Merges (Neither Repo Exists)

The following merge pairs have **NO local presence** and cannot proceed:

| Merge | Source | Target | Recommendation |
|-------|--------|--------|----------------|
| #1 | phenotype-contract | phenotype-contracts | Create from scratch or locate externally |
| #2 | phenotype-error-core | phenotype-error-core | Docs exist in `docs/adoption/` - create crate |
| #3 | phenotype-ports-canonical | phenotype-contracts | Docs exist in `docs/adoption/` - create crate |
| #10 | FixitGo + FixitRs | fixit | Create unified fixit repo |
| #11 | phenotype-config-loader | phenotype-config-core | Create from scratch |
| #12 | phenotype-shared-config | phenotype-config-core | Create from scratch |
| #13 | phenotype-async-traits | phenotype-contracts | Create from scratch |

**Note:** The documentation for some of these exists in `docs/adoption/`, suggesting they were planned but never implemented.

---

## Already Completed

### Merge #15: vibeproxy-monitoring-unified

**Status:** ✅ ARCHIVED  
**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/vibeproxy-monitoring-unified/`

**Contents:**
- Only `.editorconfig`, `.github/`, `.pre-commit-config.yaml`
- No active code
- Properly archived state

---

## Consolidation Execution Order

### Phase A: Immediate (Ready Now)
1. **forgecode-fork deletion** - Zero risk, immediate cleanup
2. **thegent-plugin-host → thegent** - Medium effort, adds capability

### Phase B: After Version Alignment
3. **agileplus-mcp → AgilePlus** - After verifying Python structure
4. **agileplus-agents → AgilePlus** - After tonic 0.13 upgrade

### Phase C: Pending Discovery
5. **hexagon-rust, router-docs, bifrost-routing*** - Search archives/external repos

### Phase D: New Implementation
6. **Blocked merges (#1-3, #10-13)** - Create repos from documentation specs

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **Version conflicts (tonic)** | High | High | Upgrade agileplus-agents to 0.13 first |
| **Plugin architecture clash** | Medium | Medium | Review AgilePlus plugin-* before merging |
| **Workspace bloat** | Medium | Low | Monitor AgilePlus build times |
| **gRPC proto misalignment** | Medium | High | Establish single proto source |
| **Binary name collisions** | Low | Medium | Verify before merge |
| **Import path breakage** | Medium | Medium | Update all `use` statements |

---

## Appendix A: File Inventory

### thegent-plugin-host
- 19 Rust source files
- Hexagonal architecture
- 51-line Cargo.toml
- Integration tests

### agileplus-agents
- 3 crates
- ~20 Rust source files
- gRPC proto definitions
- Tonic 0.12 dependency

### agileplus-mcp
- Python package
- FastMCP 3.0
- ~15 Python files
- gRPC client

### forgecode-fork
- Essentially empty (stub only)

---

## Appendix B: Dependency Matrix

| Repo | tokio | tonic | serde | thiserror | notes |
|------|-------|-------|-------|-----------|-------|
| thegent-plugin-host | 1.x | - | 1.x | 2 | No tonic |
| agileplus-agents | 1.x | 0.12 | 1.x | 2 | Needs upgrade |
| AgilePlus | 1.x | 0.13 | 1.x | 2 | Reference |
| thegent (ws) | - | - | - | 1.0 | Uses gix 0.79 |

---

**End of Consolidation Plan**
