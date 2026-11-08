---
work_package_id: WP24
title: 'KISS Simplification Pass (Cross-Repo)'
lane: planned
dependencies: []
subtasks: [T072, T073, T074]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
---

# WP24: KISS Simplification Pass (Cross-Repo)

**Implementation command**: `spec-kitty implement WP24`

## Objective

Address KISS (Keep It Simple, Stupid) violations across the ecosystem: god modules, committed artifacts that belong in .gitignore or .archive/, duplicated directory structures, namespace explosions, and unclear module boundaries.

## Context

- heliosCLI: `helios-rs/cli/src/main.rs` is a 1,485L god module combining CLI dispatch, desktop detection, WSL handling, MCP, cloud features, feature toggles, shell completions, and login; `perf-results/` has 40+ benchmark dirs committed to source; multi-model connectors (ollama/lmstudio/chatgpt/backend-client) each independently implement list_models/connect/close/call_tool
- cliproxy++: dual executor directories (`internal/runtime/executor/` AND `pkg/llmproxy/executor/`) are identical copies; `sdk/cliproxy/service.go` is a 1,724L god class
- thegent: 150+ flat entries in `src/thegent/` namespace; `cliproxy_*.py` (6 files) not consolidated into `cliproxy/` subdir; `govern/` vs `governance/` naming collision (1,465L vs 13,252L); `clode_model_routing.py` + `clode_glm_policy.py` should be `routing/` subpackage; `swarm/` (380L) vs `orchestration/` (13K) unclear boundary
- heliosApp: desktop/renderer/runtime three-way split lacks ownership boundary documentation
- trace: 4 NATS debug binaries should consolidate or move to tests/tooling; root-level lint artifacts committed; 43 Go files exceed 500 LOC
- portage: `terminus_2.py` main class still overloaded after prior extraction

## Subtasks

### T072: heliosCLI KISS simplification

**Steps**:
1. Decompose `helios-rs/cli/src/main.rs` (1,485L) into submodules:
   - `cli/dispatch.rs` — command dispatch and argument parsing
   - `cli/desktop.rs` — desktop detection and WSL handling
   - `cli/mcp.rs` — MCP server configuration
   - `cli/cloud.rs` — cloud feature toggles
   - `cli/completions.rs` — shell completion generation
   - `cli/auth.rs` — login and authentication flow
   - `main.rs` — thin entry point composing the above
2. Move `perf-results/` to `.archive/perf-results/` or add to `.gitignore` — benchmark artifacts should not be in source tree
3. Deduplicate multi-model connector interfaces (overlaps with T055 ModelConnector trait — coordinate or defer if T055 handles this)

**Validation**:
- `main.rs` is <200 LOC
- Each submodule is <400 LOC
- `perf-results/` is no longer tracked in git (either archived or gitignored)
- `cargo build --workspace` and `cargo test --workspace` pass

### T073: cliproxy++ and thegent KISS simplification

**Steps**:
1. cliproxy++: Delete one of the dual executor directories (`internal/runtime/executor/` or `pkg/llmproxy/executor/`) — keep the canonical one, update all imports
2. cliproxy++: Split `sdk/cliproxy/service.go` (1,724L) into focused service files by responsibility
3. thegent: Consolidate `cliproxy_*.py` (6 files) into existing `cliproxy/` subdirectory
4. thegent: Resolve `govern/` vs `governance/` naming collision — merge into single `governance/` package
5. thegent: Move `clode_model_routing.py` + `clode_glm_policy.py` into `routing/` subpackage
6. thegent: Document `swarm/` vs `orchestration/` boundary or merge if boundary is not meaningful
7. thegent: Organize top-level `src/thegent/` — group 150+ flat entries into logical subdirectories

**Validation**:
- Only one executor directory exists in cliproxy++
- `service.go` split into files each <500 LOC
- `src/thegent/` top-level has <30 direct entries (rest in subdirectories)
- No `govern/` directory (merged into `governance/`)
- All test suites pass

### T074: trace + portage + heliosApp KISS simplification

**Steps**:
1. trace: Consolidate 4 NATS debug binaries into a single `cmd/nats-debug/` tool or move to `tests/tooling/`
2. trace: Remove root-level lint artifacts from source control (add to .gitignore)
3. trace: Identify and split the worst offenders among 43 Go files >500 LOC (target top 10)
4. portage: Further decompose `terminus_2.py` main class — extract session management, command dispatch, and output processing into separate modules
5. heliosApp: Document desktop/renderer/runtime ownership boundaries in a `docs/architecture/` file listing which modules belong to which layer and the allowed dependency directions

**Validation**:
- trace: single NATS debug entry point or moved to tests/
- trace: no lint artifacts in root
- trace: top 10 largest files each reduced below 500 LOC
- portage: `terminus_2.py` main class <300 LOC
- heliosApp: boundary documentation exists and matches actual import graph
- All test suites pass

## Definition of Done

- [ ] No god module >500 LOC across heliosCLI CLI, cliproxy++ service, thegent top-level
- [ ] No committed benchmark/lint artifacts in source trees
- [ ] No duplicate directory structures (cliproxy++ executor dirs)
- [ ] thegent namespace organized into logical subdirectories
- [ ] All naming collisions resolved
- [ ] All test suites pass across all affected repos

## Risks

- **Namespace reorganization breaks imports**: Particularly thegent with 150+ entries. Use automated refactoring tools (rope, jedi) and run full test suites after each move.
- **Deleting duplicate executor dir may have subtle differences**: Diff the two directories thoroughly before deleting one. If differences exist, reconcile first.
- **perf-results may be referenced by CI or docs**: Check for references before archiving.

## Reviewer Guidance

- Verify that decomposed modules have clear single responsibilities
- Check that deleted duplicates are truly identical (or differences reconciled)
- Confirm namespace reorganization preserves all public imports via re-exports
- Verify .gitignore additions are correct and do not exclude needed files
