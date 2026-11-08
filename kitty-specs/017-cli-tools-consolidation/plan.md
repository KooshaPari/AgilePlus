# Plan: CLI Tools Consolidation — Across 7 Repositories

> Phased WBS with DAG. Agent-led; wall-clock assumes orchestrator with parallel subagent fan-out. Per-WP subtasks live in `tasks.md`.

## Phase 0: Discovery

| WP | Description | Predecessors | Repos | Est. effort |
|----|-------------|--------------|-------|-------------|
| WP-000 | Inventory commands, flags, exit codes, transport surfaces, and dependency graph across all 7 repos. Detect duplicated logic (proxy↔agent API, sharecli↔cli-share). Output: `research/017-cli-inventory.md` | — | all 7 | 3 parallel explore subagents, ~6 min |

## Phase 1: Foundations (LLM proxy + agent API + framework)

| WP | Description | Predecessors | Repos | Est. effort |
|----|-------------|--------------|-------|-------------|
| WP-001 | Complete `cliproxyapi-plusplus`: 8+ provider adapters, streaming, retry/backoff, auth, conformance suite | WP-000 | cliproxyapi-plusplus | 15–20 tool calls, 10–14 min |
| WP-002 | Complete `agentapi-plusplus`: agent surface that delegates to WP-001 proxy (no provider duplication); contract test harness | WP-001 | agentapi-plusplus | 12–16 tool calls, 8–12 min |
| WP-003 | Stabilize `Cmdra` framework: command tree, flags, completions, plugin hook, doc generator, semver-stable v1 surface | WP-000 | Cmdra | 12–16 tool calls, 8–12 min |

WP-001 and WP-003 are independent — dispatch as parallel siblings.

## Phase 2: Workflows + Subprocess

| WP | Description | Predecessors | Repos | Est. effort |
|----|-------------|--------------|-------|-------------|
| WP-004 | Complete `forgecode` git workflows; refactor entry points onto Cmdra | WP-003 | forgecode | 10–14 tool calls, 6–10 min |
| WP-005 | Migrate `thegent-subprocess` to expose Cmdra-compatible command surface; integrate with framework process supervisor | WP-003 | thegent-subprocess | 8–12 tool calls, 5–8 min |

## Phase 3: Deduplication

| WP | Description | Predecessors | Repos | Est. effort |
|----|-------------|--------------|-------|-------------|
| WP-006 | Diff `thegent-sharecli` vs `thegent-cli-share`; merge into single canonical repo (other becomes thin redirect/archive); preserve every flag/exit code | WP-000 | thegent-sharecli, thegent-cli-share | 10–14 tool calls, 6–10 min |

## Phase 4: Integration + Migration

| WP | Description | Predecessors | Repos | Est. effort |
|----|-------------|--------------|-------|-------------|
| WP-007 | Migrate WP-001/WP-002/WP-004/WP-005/WP-006 outputs onto Cmdra runtime; cross-tool conformance suite; deprecation notices on old binaries | WP-001, WP-002, WP-003, WP-004, WP-005, WP-006 | all 7 | 18–25 tool calls, 12–18 min |
| WP-008 | Author migration guides + unified `phenotype-cli` index doc; update org docsites | WP-007 | docs only | 6–10 tool calls, 4–6 min |

## DAG

```
WP-000 ─┬─► WP-001 ──► WP-002 ──┐
        │                        │
        ├─► WP-003 ─┬─► WP-004 ──┤
        │           └─► WP-005 ──┤
        │                        │
        └─► WP-006 ──────────────┴─► WP-007 ──► WP-008
```

## Cross-Project Reuse Opportunities

- Provider adapter registry from WP-001 should reuse `phenotype-llm` abstractions (cheap-llm-mcp + thegent-dispatch live there) — do not re-roll.
- Cmdra plugin hook ought to align with the contract from spec 015 (plugin-system-completion). Keep both designs in sync; ideally both consume `agileplus-plugin-core`.
- Conformance test scaffolding is a candidate to extract into a shared `phenotype-cli-conformance` crate/library.

## Risks

| Risk | Mitigation |
|------|-----------|
| Two consolidations land breaking changes simultaneously | Stage WP-007 last; pin Cmdra v1 surface during Phase 1 |
| Cross-language (TS/Rust/Go) drift | Generate tool surface from a single schema in WP-000; regenerate per-language wrappers |
| Sharecli merge loses functionality | WP-006 must produce a feature-parity diff before delete; archived repo gets a redirect README |

## Total estimated effort

~10–14 orchestrator-hours wall-clock, 8 WPs, fan-out factor up to 3 in Phase 1. No human gates.
