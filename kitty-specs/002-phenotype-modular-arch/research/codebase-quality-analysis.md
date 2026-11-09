# Codebase Quality Analysis — All Repos

**Date**: 2026-03-03
**Purpose**: Direct LOC/complexity/quality metrics to inform work packages.

---

## Cross-Repo Summary

| Repo | Canonical LOC | God Modules (>500L) | Largest File | Worst Function | unwrap/panic | Test Gap |
|------|-------------|-------------------|-------------|---------------|-------------|---------|
| heliosCLI | 422K Rust + 0.6K TS | 70+ | codex.rs (9,571) | — | 934 unwrap() | 41% coverage, 11 crates untested |
| cliproxyapi++ | 166K Go | 61 | kiro_executor.go (4,691) | executeClaudeNonStream (205L) | 3 panics (defensible) | 35+ pkgs untested |
| agentapi++ | 8.4K Go (unique) | 2 | server.go (788) | — | 14 panics (init-time) | 9 pkgs untested |
| thegent | ~160K Python (unique) | 40+ | project.py (2,012) | run_impl_core (1,022L!) | — | 469 dup files, 221 circular dep files |
| portage | 103K Python | 20 | terminus_2.py (1,838) | create_app (1,164L) | — | 47.5% modules untested, 8 dup BaseAdapter |
| heliosApp | 78K TS | 8 | index.ts (1,089) | — | 0 any | audit subsystem (2,577L) untested |
| trace | 678K (Go+Py+TS) | 98+ | api/main.py (9,274) | — | 390 type suppressions | 1,847L hand-rolled parsers |

---

## heliosCLI (422K Rust)

### Critical Findings
- **65 crates** in workspace, 956 .rs files
- **Top god modules**: codex.rs (9,571), chat_composer.rs (9,499), codex_message_processor.rs (8,460), chatwidget.rs (8,146), config/mod.rs (6,218), app.rs (5,463)
- **934 `unwrap()` in production code** — crash risk surface. Worst: apply-patch (74), network-proxy/runtime.rs (38), turn_diff_tracker.rs (37)
- **54 `#[allow(dead_code)]`** suppressions, 100 `// TODO` comments
- **Connector duplication**: ollama, lmstudio, chatgpt, backend-client crates implement near-identical model list/connect interfaces
- **41.3% file-level test coverage**, 11 crates with zero tests (incl. codex-backend-openapi-models at 14 files)

### Complexity Hotspots (branch keyword density)
1. chat_composer.rs: 469 branches
2. chatwidget.rs: 457
3. codex_message_processor.rs: 423
4. codex.rs: 369
5. app.rs: 270

---

## cliproxyapi++ (166K Go)

### Critical Findings
- **15 executor files** totaling 15,403 LOC — each reimplements HTTP/retry/streaming independently
- **21,441 LOC of auth code** across 14 provider subdirectories, 6 distinct auth mechanisms
- **kiro_executor.go at 4,691 LOC** — 3x the next largest executor
- **Functions exceeding 200 lines**: executeClaudeNonStream (205L), ExecuteStream (198L), Execute (149L)
- **Zero shared executor base** — only logging_helpers.go + usage_helpers.go (612L combined) are extracted
- **internal/auth/ (~600L)** appears dead — superseded by pkg/llmproxy/auth/
- Error handling excellent: 0 suppressed errors, 3 defensible panics

### Executor Size Distribution
| Executor | LOC |
|----------|-----|
| kiro | 4,691 |
| antigravity | 1,783 |
| codex_websockets | 1,432 |
| claude | 1,352 |
| github_copilot | 1,173 |
| gemini_vertex | 1,032 |
| gemini_cli | 961 |
| codex | 827 |
| kimi | 596 |
| iflow | 557 |
| gemini | 539 |
| aistudio | 495 |
| kilo | 432 |
| openai_compat | 377 |
| qwen | 356 |

---

## agentapi++ (8.4K Go unique)

### Critical Findings
- **Inner duplication**: repo contains a full copy of itself under `agentapi-plusplus/agentapi-plusplus/` — ~50% of LOC is duplicate
- Only 2 god modules (server.go 788L, pty_conversation.go 523L) — healthy
- **Not a reimplementation of cliproxy**: thin agent lifecycle manager + SSE bridge that proxies to cliproxy
- 9 packages with zero tests but all small (1-2 files each)

---

## thegent (~160K Python unique)

### Critical Findings
- **469 exact duplicate files** between src/thegent/ and packages/thegent-*/ — migration artifact, not cleaned up
- **run_impl_core at 1,022 lines** — single largest function in entire ecosystem
- **221 files (15%) use TYPE_CHECKING** — massive circular dependency surface
- **Migration 32% complete**: 469/1,467 src files migrated but not deleted
- **run_execution_core_helpers.py exists in 3 locations** — diverged tri-copy
- **Packages by LOC**: thegent-cli (89K), thegent-agents (37K), thegent-audit (20K), thegent-protocols (18K), thegent-sync (16K), thegent-mcp (16K)
- Test files (1,309) are mostly auto-generated coverage-wave tests, not organic unit tests
- Type annotation coverage excellent at 96%

---

## portage (103K Python)

### Critical Findings
- **8 independent BaseAdapter definitions** across 41 adapters — no shared base infrastructure
- **create_app at 1,164 lines** — god function (entire FastAPI app in one function)
- **jobs.py::start at 856 lines** — another god function
- **47.5% of source modules untested** including all environment backends (gke, e2b, modal, runloop)
- **41 benchmark adapters** totaling 34,939 LOC with heavy pattern duplication
- Type annotation coverage at 72% (lower than thegent)
- Only 4 TYPE_CHECKING files — clean layering despite other issues
- **terminus_2.py (1,838L)** needs split into parser/renderer/state

---

## heliosApp (78K TypeScript)

### Critical Findings
- **Bun-native, NOT Electron** — 3 processes: desktop (13K), renderer (1.3K), runtime (63K)
- **Audit subsystem (2,577L) entirely untested** — 12 source files, 0 tests
- **Protocol bus (805L) untested** — critical IPC layer
- **48 IPC topics** in pub/sub channel registry
- **Zero `any` usage**, zero `console.log` in prod — excellent TypeScript discipline
- **11 of 17 runtime modules have zero tests**
- 10 meaningful TODOs (integration stubs, not throwaway)
- All deps current, no deprecated packages

---

## trace (678K Go+Python+TS)

### Critical Findings
- **api/main.py at 9,274 LOC** — most severe god module in entire ecosystem
- **1,847 LOC of hand-rolled parsers** (python.go 653L, golang_declarations.go 391L, typescript_declarations.go 307L) — tree-sitter replacement candidates
- **43 Go files exceed 500 LOC**, 55 Python files exceed 500 LOC
- **390 Python type suppressions** (type: ignore + noqa) — retrofitted typing
- **gorilla/mux (archived/deprecated)** + echo both in use — redundant routers
- **182 HTTP routes** in Go backend
- **No tree-sitter dependency** — all parsing is regex/string manipulation
- **5 `.go.bak` files** in handlers/ — dead backup artifacts
- Go test coverage excellent at 92% test-to-source ratio
- Neo4j client wrappers: 3,416 LOC in internal/graph/
- NATS integration: 1,262 LOC in internal/nats/
- Embeddings: 2,228 LOC using pgvector (not chromem-go)

---

## Top Priority Actions by Impact

### Tier 1: Highest LOC/Quality Impact
1. **thegent**: Delete 469 duplicate files (immediate ~100K LOC reduction)
2. **cliproxy**: Bifrost adoption replaces 9+ executors (25-40K LOC)
3. **trace**: tree-sitter replaces hand-rolled parsers (1,847L direct + enables scip-go)
4. **trace**: Split api/main.py (9,274L god module)
5. **heliosCLI**: Split codex.rs (9,571L), chat_composer.rs (9,499L), codex_message_processor.rs (8,460L)

### Tier 2: Quality/Safety Critical
6. **heliosCLI**: Fix 934 unwrap() in production → `?` propagation
7. **portage**: Extract shared BaseAdapter (eliminates 8-way duplication)
8. **thegent**: Decompose run_impl_core (1,022L function)
9. **cliproxy**: Split kiro_executor.go (4,691L)
10. **heliosApp**: Test audit subsystem (2,577L untested)

### Tier 3: Structural Health
11. **thegent**: Break 221-file circular dep surface
12. **portage**: Split create_app (1,164L), jobs::start (856L)
13. **trace**: Remove gorilla/mux, consolidate on echo
14. **agentapi++**: Remove inner directory duplication
15. **cliproxy**: Consolidate 21K LOC auth → x/oauth2+Goth
