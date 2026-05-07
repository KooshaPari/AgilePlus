# Plan: Template Repo Cleanup — Consolidate 27 Template Repositories

> Phased WBS with DAG. Agent-led; wall-clock assumes orchestrator with parallel subagent fan-out per language family. Per-WP subtasks in `tasks.md`.

## Phase 0: Discovery

| WP | Description | Predecessors | Est. effort | Status |
|----|-------------|--------------|-------------|--------|
| WP-000 | Walk all 27 template repos. Capture: file tree, dependency manifests, last-touched commit, declared purpose, downstream consumers (find via GitHub code search). Output: `research/018-template-inventory.md` per-repo card and a duplicate-pair table. | — | 4 parallel explore subagents (one per family), ~8 min | Partial — see prior progress below |

### Prior progress (preserved from earlier plan)
- 6 local repos audited; 21 GitHub-only repos discovered.
- 3 fixable issues found and fixed (CI version mismatch, type errors, placeholder CI).
- 1 broken repo identified (Hexacore — all workspace members missing).
- 1 archived repo confirmed (phenotype-design).
- Duplicate pairs documented in worklog.
- Open PRs already in flight: HexaGo#2, HexaType#3, phenotype-design#28.

## Phase 1: Hexagonal Family Merges (siblings — fan-out)

Each pair below is a small, repeatable migration: pick canonical repo (prefer the more recently active one), port unique files from the duplicate, update README, mark duplicate as archived with redirect.

| WP | Description | Predecessors | Repos | Est. effort |
|----|-------------|--------------|-------|-------------|
| WP-101 | Merge `hexagon-rust` ⇄ `HexaRust` | WP-000 | 2 | 5–7 tool calls, 3–5 min |
| WP-102 | Merge `hexagon-go` ⇄ `HexaGo` (open PR HexaGo#2) | WP-000 | 2 | 5–7 tool calls, 3–5 min |
| WP-103 | Merge `hexagon-python` ⇄ `HexaPython` (HexaPy → hexagon-python target) | WP-000 | 2 | 5–7 tool calls, 3–5 min |
| WP-104 | Merge `hexagon-typescript` ⇄ `HexaTS` (open PR HexaType#3) | WP-000 | 2 | 5–7 tool calls, 3–5 min |
| WP-105 | Merge `hexagon-zig` ⇄ `HexaZig` | WP-000 | 2 | 5–7 tool calls, 3–5 min |
| WP-106 | Merge `hexagon-cpp` ⇄ `HexaCPP` | WP-000 | 2 | 5–7 tool calls, 3–5 min |
| WP-107 | Archive `hexagon-odin` (legacy language, no consumer); fix `Hexacore` workspace members or archive | WP-000 | 2 | 3–5 tool calls, ~2 min |

WP-101..WP-107 are siblings — dispatch as up to 7 parallel subagents.

## Phase 2: Language-Lang Consolidation

| WP | Description | Predecessors | Repos | Est. effort |
|----|-------------|--------------|-------|-------------|
| WP-201 | Consolidate `template-lang-{rust,go,python,typescript,zig,cpp,web}` into a normalized layout (shared baseline files extracted) | WP-000 | 7 active | 10–14 tool calls, 6–10 min |
| WP-202 | Archive `template-lang-{java,swift,kotlin,ruby,php}` (no active consumer); leave README pointer | WP-000 | 5 inactive | 4–6 tool calls, 2–3 min |
| WP-203 | Move `template-lang-commons` and `hexagon-shared` into the generator (Phase 3) | WP-101..WP-107, WP-201 | 2 shared | 4–6 tool calls, 3–5 min |

## Phase 3: Generator

| WP | Description | Predecessors | Est. effort |
|----|-------------|--------------|-------------|
| WP-301 | Build a single template generator (Rust, per scripting policy) consuming the consolidated set: language flag, hex/non-hex flag, options matrix; ingests `template-lang-commons` + `hexagon-shared` baselines | WP-101..WP-107, WP-201, WP-202, WP-203 | 15–20 tool calls, 10–14 min |
| WP-302 | Generator validation: run against every consolidated template, run `cargo check` / `go build` / `pnpm install` on each generated project | WP-301 | 8–12 tool calls, 5–8 min |

## Phase 4: Migration + Docs

| WP | Description | Predecessors | Est. effort |
|----|-------------|--------------|-------------|
| WP-401 | Per-template README rewrite + central `templates.kooshapari.com` index entry (per Org Pages standing policy) | WP-301 | 6–10 tool calls, 4–6 min |
| WP-402 | Migration guide for downstream projects pointing to old templates; auto-redirect via archived-repo README | WP-301 | 4–6 tool calls, 2–4 min |

## DAG

```
                        ┌─► WP-101 ─┐
                        ├─► WP-102 ─┤
                        ├─► WP-103 ─┤
                        ├─► WP-104 ─┤
                        ├─► WP-105 ─┤
WP-000 ─────────────────┼─► WP-106 ─┤
                        ├─► WP-107 ─┤
                        ├─► WP-201 ─┤
                        ├─► WP-202 ─┤
                        └─► WP-203 ─┘
                                    │
                                    ▼
                                  WP-301 ─► WP-302 ─┬─► WP-401
                                                     └─► WP-402
```

## Cross-Project Reuse Opportunities

- Generator should reuse `phenotype-config-core` for option layering rather than hand-rolling TOML/JSON merge.
- Consolidated baseline files duplicate concerns already governed by `phenotype-infrakit` (Make/Task targets, lint config). Pull from there rather than copy in.
- Migration scaffolding overlaps with spec 019 (private-repo-catalog) — share archive-redirect tooling between the two.

## Risks

| Risk | Mitigation |
|------|-----------|
| Dropping a feature during a hex-pair merge | WP-000 produces per-pair feature diff; merge PR includes diff-before-delete checklist |
| Generator complexity drift | WP-302 is a hard gate — every consolidated template must round-trip cleanly |
| Downstream project breakage | WP-402 ships before any archive; archived repos keep README + last release tag |

## Total estimated effort

~10–14 orchestrator-hours wall-clock, dominated by parallel Phase 1 fan-out (~5 min) and Phase 3 generator build (~15 min). No human checkpoints.
