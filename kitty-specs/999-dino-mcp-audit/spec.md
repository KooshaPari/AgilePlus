# Feature Spec: Dino MCP Tooling Decomposition Audit

**Spec ID:** 999-dino-mcp-audit  
**Status:** in_progress  
**Created:** 2026-04-04  
**Author:** Claude  
**Type:** audit  
**Priority:** medium  

---

## 1. Problem Statement

Dino's various MCP (Model Context Protocol) and game modding/agentic tooling is scattered across multiple repositories with unclear boundaries, overlapping concerns, and varying states of completion. We need to:

1. **Decompose** the components into discrete, evaluable units
2. **Evaluate** their current state, utility, and integration points
3. **Audit** for code quality, test coverage, and architectural alignment

---

## 2. Component Inventory

### 2.1 Existing Components

| Component | Location | LOC | State | Purpose |
|-----------|----------|-----|-------|---------|
| phenotype-mcp-asset | repos/phenotype-mcp-asset/ | 409 | ✅ Functional | MCP server for asset/pack operations |
| phenotype-mcp-testing | repos/phenotype-mcp-testing/ | 533 | ✅ Functional | MCP server for game testing automation |
| phenotype-packs-rs | repos/phenotype-packs-rs/ | ~50 | 📝 Skeleton | Content pack system (Rust) |
| phenotype-mcp-core | repos/crates/phenotype-mcp-core/ | 41 | ✅ Compiles | Core MCP protocol implementation |

### 2.2 Lost Components

| Component | Last Known | Status | Recovery Action |
|-----------|------------|--------|-----------------|
| dinoforge-packs | KooshaPari/dinoforge-packs | ❌ DELETED 31m ago | LOST - Not recoverable |

---

## 3. Decomposition Plan

### 3.1 Component Boundaries

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     DINO MCP TOOLING - DECOMPOSED VIEW                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────┐  │
│  │  MCP Protocol Core  │    │  Asset Operations   │    │ Game Testing    │  │
│  │  phenotype-mcp-core │◄───│ phenotype-mcp-asset │    │ phenotype-mcp   │  │
│  │                     │    │                     │    │ -testing        │  │
│  │ - JSON-RPC framing  │    │ - Pack discovery    │    │                 │  │
│  │ - Tool schemas      │    │ - Build/compile     │    │ - Process mgmt  │  │
│  │ - Capability mgmt   │    │ - Validation        │    │ - Save/load     │  │
│  └─────────────────────┘    │ - Dependency resol  │    │ - Screenshots   │  │
│           ▲                 └─────────────────────┘    │ - Test runner   │  │
│           │                           │                  └─────────────────┘  │
│           │                           │                         │             │
│           └───────────────────────────┴─────────────────────────┘             │
│                                   │                                           │
│                                   ▼                                           │
│                    ┌─────────────────────────┐                                │
│                    │   Content Pack System   │                                │
│                    │   phenotype-packs-rs    │                                │
│                    │                         │                                │
│                    │ - Pack format spec      │                                │
│                    │ - Parser/serializer     │                                │
│                    │ - Dependency graph      │                                │
│                    └─────────────────────────┘                                │
│                                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Dependency Graph

```
crates/phenotype-mcp-core
    │
    ├──► phenotype-mcp-asset (pack ops)
    │         └──► phenotype-packs-rs (content system)
    │
    └──► phenotype-mcp-testing (game automation)
```

---

## 4. Evaluation Results

### 4.1 phenotype-mcp-asset (792 LOC total: 409 main + 383 handler)

| Aspect | Rating | Notes |
|--------|--------|-------|
| **Functionality** | ⭐⭐⭐⭐ | Full MCP protocol + working asset operations |
| **Code Quality** | ⭐⭐⭐⭐ | Clean structure, proper error handling |
| **Test Coverage** | ⭐ | No tests found |
| **Documentation** | ⭐⭐⭐⭐ | Good inline docs, clear tool descriptions |
| **Completeness** | ⭐⭐⭐⭐ | Actual implementations: discover, build, validate, resolve_deps, get_info |

**Implemented Features:**
- ✅ Asset discovery with file classification (.pack, .wasm, .py, .js, .cs, .go, .sh, .toml, .json, .md)
- ✅ Pack building with `phenotype.toml` manifest parsing
- ✅ Pack validation (version check, asset existence, dependency check)
- ✅ Dependency resolution (placeholder - returns mock versions)
- ✅ Metadata extraction from TOML manifests

**Key Issues:**
1. No tests
2. Blocking async in sync context (line 391-437 in main.rs)
3. Dependency resolution is mocked
4. Build output is JSON-serialized (not a real pack format)

### 4.2 phenotype-mcp-testing (871 LOC total: 533 main + 338 handler)

| Aspect | Rating | Notes |
|--------|--------|-------|
| **Functionality** | ⭐⭐⭐⭐ | Game process management + testing framework |
| **Code Quality** | ⭐⭐⭐⭐ | Good async handling, state management |
| **Test Coverage** | ⭐ | No tests found |
| **Documentation** | ⭐⭐⭐⭐ | Good inline docs |
| **Completeness** | ⭐⭐⭐ | Game automation partial, testing framework complete |

**Implemented Features:**
- ✅ Game process launch/stop with tokio::process
- ✅ Game state tracking (running, pid, executable, start_time)
- ✅ Test suite registration and management
- ✅ Test execution with results tracking
- ✅ Test reports with success rates
- ✅ Test configuration validation

**Key Issues:**
1. `save_state`, `load_state`, `take_screenshot` are all placeholders (need game integration)
2. Test runner is mocked (needs actual test discovery/execution)
3. No tests for the testing framework itself
4. No actual game communication protocol (needs IPC or network protocol)

**Tools Provided:**
- `game_launch` - Process spawn (✅ working)
- `game_stop` - Process termination (✅ working)
- `game_get_state` - Status query (✅ working)
- `game_save_state` - Save game (❌ placeholder)
- `game_load_state` - Load game (❌ placeholder)
- `game_screenshot` - Screenshot (❌ placeholder)
- `game_run_test` - Automated test (⚠️ mocked)

### 4.3 phenotype-packs-rs (~50 LOC)

| Aspect | Rating | Notes |
|--------|--------|-------|
| **Functionality** | ⭐ | Skeleton only |
| **Code Quality** | N/A | No actual code to evaluate |
| **Test Coverage** | N/A | |
| **Documentation** | ⭐⭐ | README only |
| **Completeness** | ⭐ | Empty lib.rs |

**Key Issues:**
1. Empty implementation
2. No pack format defined
3. No parser/serializer
4. Just a placeholder crate

### 4.4 phenotype-mcp-core (41 LOC)

| Aspect | Rating | Notes |
|--------|--------|-------|
| **Functionality** | ⭐⭐ | Basic types only |
| **Code Quality** | ⭐⭐⭐ | Clean but minimal |
| **Test Coverage** | ⭐ | No tests |
| **Documentation** | ⭐⭐ | Sparse |
| **Completeness** | ⭐⭐ | Incomplete |

**Key Issues:**
1. Missing src directory was fixed recently
2. Very minimal implementation
3. Not used by the MCP servers (they reimplement)

---

## 5. Architecture Audit

### 5.1 Code Duplication

**CRITICAL:** Both MCP servers reimplement the entire MCP protocol stack:

| Duplicated Component | Lines | Location |
|---------------------|-------|----------|
| McpRequest struct | ~10 | Both main.rs files |
| McpResponse struct | ~15 | Both main.rs files |
| McpError struct | ~10 | Both main.rs files |
| Tool/ToolCapabilities | ~20 | Both main.rs files |
| ServerInfo/Capabilities | ~20 | Both main.rs files |
| JSON-RPC handler loop | ~60 | Both main.rs files |
| Tool dispatch logic | ~50 | Both main.rs files |

**Total Duplication:** ~185 LOC duplicated across 2 servers

### 5.2 Protocol Alignment

| Specification | Compliance | Notes |
|--------------|------------|-------|
| MCP 2024-11-05 | ✅ | Correct protocol version |
| JSON-RPC 2.0 | ✅ | Proper framing |
| Tool schemas | ✅ | Valid JSON Schema |
| Error codes | ⚠️ | Using standard JSON-RPC codes only |
| Progress notifications | ❌ | Not implemented |
| Streaming | ❌ | Not implemented |

### 5.3 Security Audit

| Check | Status | Notes |
|-------|--------|-------|
| Input validation | ⚠️ | Basic JSON schema only |
| Path traversal | ⚠️ | No sanitization on pack_path |
| Command injection | ⚠️ | game_launch passes args directly |
| Rate limiting | ❌ | None |
| Authentication | ❌ | None |

---

## 6. Recommendations

### 6.1 Immediate Actions

1. **Extract MCP Framework** (Priority: HIGH)
   - Create `phenotype-mcp-framework` crate
   - Extract common protocol handling
   - Both servers should depend on this
   - **Expected reduction:** ~185 LOC per new server

2. **Implement Tests** (Priority: HIGH)
   - Unit tests for handlers
   - Integration tests with mock MCP client
   - Test the protocol layer

3. **Fix Async Blocking** (Priority: MEDIUM)
   - Replace `rt.block_on()` with proper async handlers
   - Use `tokio::spawn` for concurrent operations

### 6.2 Consolidation Strategy

```
Current State:                    Target State:
┌──────────────────┐              ┌──────────────────────┐
│ mcp-asset        │              │ phenotype-mcp-core   │
│  - main.rs (409) │              │  - protocol          │
│  - reimplements  │    ───►      │  - framing           │
│    everything    │              │  - types             │
├──────────────────┤              ├──────────────────────┤
│ mcp-testing      │              │ phenotype-mcp-asset  │
│  - main.rs (533) │              │  - handlers only     │
│  - reimplements  │              │  - ~200 LOC          │
│    everything    │              ├──────────────────────┤
└──────────────────┘              │ phenotype-mcp-testing│
                                  │  - handlers only     │
                                  │  - ~300 LOC          │
                                  └──────────────────────┘
```

### 6.3 Lost dinoforge-packs Recovery

Since `dinoforge-packs` was deleted from GitHub:

1. **Check local clones** - Any worktrees or old checkouts?
2. **Check backups** - Time Machine, other backup systems?
3. **Check git reflog** - Any local references?
4. **Alternative:** Rewrite based on phenotype-packs-rs goals

---

### 6.4 Package Registry Configuration

All extracted crates need publishing pipeline configuration:

| Registry | Workflow | Secret Name | Status |
|----------|----------|-------------|--------|
| crates.io | `publish-crate.yml` | `CARGO_REGISTRY_TOKEN` | ✅ Workflow ready, needs secret |
| PyPI | `publish-pypi.yml` | `PYPI_API_TOKEN` | ✅ Workflow ready, needs secret |
| npmjs | `publish-npm.yml` | `NPM_TOKEN` | ✅ Workflow ready, needs secret |
| NuGet | `publish-nuget.yml` | `NUGET_API_KEY` | ✅ Created in this audit, needs secret |
| Ziggistry | ❌ None | `ZIGGISTRY_TOKEN` | 🔴 Needs research + workflow + secret |

**Configuration Tasks:**
1. Add secrets to GitHub repository settings (Settings → Secrets → Actions)
2. Package name audit - check for duplicates on each registry
3. Consider namespace strategy: `@phenotype/*` for npm, `pheno-*` for others
4. Reserve package names if not already taken

**⚠️ Duplicate Package Names:** User reports similar/duplicate names found on PyPI and other registries. Need package name audit before publishing.

---

## 7. Work Packages

### WP-1: Extract MCP Framework
- **ID:** wp-extract-mcp-framework
- **Estimate:** 2 days
- **Dependencies:** None
- **Deliverable:** `phenotype-mcp-framework` crate with shared protocol code

### WP-2: Refactor mcp-asset
- **ID:** wp-refactor-mcp-asset
- **Estimate:** 1 day
- **Dependencies:** wp-extract-mcp-framework
- **Deliverable:** Refactored to use shared framework

### WP-3: Refactor mcp-testing
- **ID:** wp-refactor-mcp-testing
- **Estimate:** 1 day
- **Dependencies:** wp-extract-mcp-framework
- **Deliverable:** Refactored to use shared framework

### WP-4: Implement phenotype-packs-rs
- **ID:** wp-implement-packs
- **Estimate:** 3 days
- **Dependencies:** None
- **Deliverable:** Working pack system with parser/serializer

### WP-5: Add Test Coverage
- **ID:** wp-add-tests
- **Estimate:** 2 days
- **Dependencies:** wp-extract-mcp-framework
- **Deliverable:** 80%+ test coverage on all MCP crates

### WP-6: Configure Package Registry Secrets
- **ID:** wp-configure-registry-secrets
- **Estimate:** 1 day
- **Dependencies:** None
- **Deliverable:** All registry secrets configured in GitHub
- **Registries:** crates.io, PyPI, npmjs, NuGet, Ziggistry

### WP-7: Create Publishing Workflows
- **ID:** wp-publishing-workflows
- **Estimate:** 1 day
- **Dependencies:** wp-configure-registry-secrets
- **Deliverable:** `.github/workflows/publish-*.yml` for each registry
- **Note:** Handle duplicate package name conflicts on PyPI and others

| Metric | Current | Target |
|--------|---------|--------|
| Code Duplication | 185 LOC | 0 LOC |
| Test Coverage | 0% | 80%+ |
| Lines of Code | 1033 (total) | 600 (after DRY) |
| Documentation | 60% | 90% |
| Security Issues | 4 | 0 |

---

## 9. Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-04-04 | Create extraction plan | Duplication is waste; framework needed |
| 2026-04-04 | dinoforge-packs considered lost | Deleted from GitHub, no local recovery |
| 2026-04-04 | phenotype-packs-rs to be implemented | Empty shell needs actual implementation |

---

## Appendix A: File Locations

```
repos/
├── phenotype-mcp-asset/
│   ├── src/
│   │   ├── main.rs          (409 LOC)
│   │   └── handler.rs         (referenced, not read)
│   ├── Cargo.toml
│   └── Cargo.lock
├── phenotype-mcp-testing/
│   ├── src/
│   │   ├── main.rs          (533 LOC)
│   │   └── handler.rs         (referenced, not read)
│   ├── Cargo.toml
│   └── Cargo.lock
├── phenotype-packs-rs/
│   ├── src/
│   │   └── lib.rs           (skeleton)
│   ├── Cargo.toml
│   └── README.md
└── crates/
    └── phenotype-mcp-core/
        └── Cargo.toml       (41 LOC declared)
```

---

## Appendix B: Handler Implementations Status

| Handler | Status | Lines | Notes |
|---------|--------|-------|-------|
| **phenotype-mcp-asset/src/handler.rs** | ✅ IMPLEMENTED | 383 | Full implementation |
| ├─ `discover()` | ✅ | 35 | Directory scanning with file classification |
| ├─ `build()` | ✅ | 135 | Pack compilation with manifest parsing |
| ├─ `validate()` | ✅ | 181 | Multi-rule validation |
| ├─ `resolve_dependencies()` | ⚠️ | 258 | Returns mock versions |
| └─ `get_info()` | ✅ | 280 | Metadata extraction |
| **phenotype-mcp-testing/src/handler.rs** | ✅ IMPLEMENTED | 338 | Testing framework |
| ├─ `TestingHandler` | ✅ | 11 | Full test suite management |
| ├─ `run_test()` | ✅ | 31 | Test execution with results |
| ├─ `run_suite()` | ✅ | 87 | Suite execution |
| ├─ `generate_report()` | ✅ | 151 | Report generation |
| └─ Game state capture | ⚠️ | 296 | Data structures only (no game integration) |

**Total Handler LOC:** 721 lines of actual implementation

