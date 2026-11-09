# Plan: Plugin System Completion — Complete Architecture Across 4 Repositories

> Phased WBS with explicit DAG. Agent-led execution; wall-clock estimates assume single-orchestrator with parallel subagent batches. Per-WP subtasks live in `tasks.md` (T001–T049).

## Phase 0: Discovery (predecessor: none)

| WP | Description | Predecessors | Repos | Est. agent effort |
|----|-------------|--------------|-------|-------------------|
| WP-000a | Audit existing `agileplus-plugin-core` interfaces; capture trait surface, version markers, lifecycle gaps | — | agileplus-plugin-core | 4–6 tool calls, ~3 min |
| WP-000b | Audit `agileplus-plugin-git`, `agileplus-plugin-sqlite`, `thegent-plugin-host` against the proposed contract; record FFI seams | — | 3 plugin repos | 2 parallel subagents, ~5 min |

Discovery output: `research/015-audit.md` capturing existing trait names, lifecycle states already in code, version metadata fields, and FFI/IPC entry points used by the Go host today. Folded into Phase 1 design.

## Phase 1: Interface and Backends

| WP | Description | Predecessors | Repos | Est. agent effort |
|----|-------------|--------------|-------|-------------------|
| WP-001 | Stabilize plugin trait, lifecycle FSM, version-compat checker, `PluginRegistry`, error taxonomy, rustdoc, ≥80% coverage (T001–T012) | WP-000a, WP-000b | agileplus-plugin-core | 12–18 tool calls, 8–12 min |
| WP-002 | Git VCS adapter implementing the trait: clone/fetch/commit/push/branch/worktree + auth + retries, integration tests with temp repos (T013–T024) | WP-001 | agileplus-plugin-git | 12–16 tool calls, 8–12 min |
| WP-003 | SQLite adapter implementing the trait: CRUD, migrations (up/down), transactions, backup, query, integration tests (T025–T036) | WP-001 | agileplus-plugin-sqlite | 12–16 tool calls, 8–12 min |

WP-002 and WP-003 are independent siblings — dispatch as 2 parallel subagents once WP-001 publishes a tagged interface version.

## Phase 2: Host Integration

| WP | Description | Predecessors | Repos | Est. agent effort |
|----|-------------|--------------|-------|-------------------|
| WP-004 | Wire `thegent-plugin-host` to plugin-core via Rust C-ABI or gRPC IPC, plugin discovery, lifecycle manager, config, register plugins as thegent subcommands (T037–T049) | WP-001, WP-002, WP-003 | thegent-plugin-host, thegent | 18–25 tool calls, 12–18 min |

## Phase 3: Validate / Handoff

| WP | Description | Predecessors | Repos | Est. agent effort |
|----|-------------|--------------|-------|-------------------|
| WP-005 | End-to-end: Go host loads both Rust plugins via FFI/IPC; lifecycle transitions verified; plugin development guide + skeleton example plugin published in plugin-core docs | WP-004 | all 4 | 8–12 tool calls, 5–8 min |

## DAG

```
WP-000a ┐
        ├─► WP-001 ─┬─► WP-002 ─┐
WP-000b ┘           │            ├─► WP-004 ─► WP-005
                    └─► WP-003 ─┘
```

## Cross-Project Reuse Opportunities

- Lifecycle FSM logic is a candidate for `phenotype-state-machine` (in `phenotype-shared`). Check before reimplementing — extract or reuse rather than hand-roll.
- Plugin error taxonomy should be defined on `phenotype-error-core` foundations.
- Versioning scheme aligns with org-wide CalVer/SemVer policy (002-org-wide-release-governance-dx-automation).

## Dependencies (external specs)

- 001-spec-driven-development-engine — provides spec/WP scaffolding consumed by plugins.
- 003-agileplus-platform-completion — registry consumer.
- 007-thegent-completion — host runtime.

## Total estimated effort

~6–10 orchestrator-hours wall-clock, decomposed into 5 sequenced WPs (WP-002 + WP-003 dispatchable in parallel after WP-001). No human checkpoints; merge gates are quality checks (`cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`) and integration test pass rate per WP.
