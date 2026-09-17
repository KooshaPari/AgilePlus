---
spec_id: consolidate-cache-adapter-crate
status: specified
created: 2026-04-25
last_audit: 2026-04-25
owners: phenotype-org / shared-platform
adr: docs/governance/shared-crates-canonical-home-adr-2026-04.md
crate: phenotype-cache-adapter
canonical_home: github.com/KooshaPari/phenoShared/crates/phenotype-cache-adapter
---

# Consolidate `phenotype-cache-adapter` to `phenoShared`

## Meta

- **ID**: consolidate-cache-adapter-crate
- **Title**: Migrate all redundant `phenotype-cache-adapter` copies to canonical `phenoShared` home
- **Scope**: Crate-level (cross-repo); 5 host workspaces; 0 external Cargo consumers
- **Priority**: P1 — High (two-tier cache semantics MUST be invariant across consumers)
- **Depends On**: `shared-crates-canonical-home-adr-2026-04` (Accepted, PhenoKits#31, merged 2026-04-25)
- **Sibling WPs (parallel-mergeable)**: `consolidate-event-sourcing-crate`, `consolidate-state-machine-crate`, `consolidate-policy-engine-crate`

## Context

Per the canonical-home ADR, `phenotype-cache-adapter` (two-tier LRU + DashMap with TTL) is duplicated across **5 host workspaces**:

| Host workspace | Notes |
|----------------|-------|
| `phenoShared` (canonical) | Same-day evolution; ADR-elected canonical |
| `pheno` | snapshot, 2026-03-31 |
| `PhenoProc` (root copy) | 2026-04-03 |
| `PhenoProc` (nested `crates/phenotype-shared/crates/`) | 2026-04-03 |
| `DataKit/rust` | snapshot, 2026-04-05 |
| `PhenoKits/HexaKit` | snapshot, 2026-03-31 |

External (cross-workspace) consumers: **0**. The audit calls out: "API parity check required" — divergent TTL eviction semantics between L1 (LRU) and L2 (DashMap) layers can produce visible behavior differences.

## Problem Statement

1. **Cache semantics drift** — even subtle TTL/eviction divergence between copies invalidates correctness assumptions for downstream consumers (rate limiting, deduplication, idempotency caches).
2. **5 redundant build targets** inflate build time and on-disk artifact churn.
3. **No SemVer signal** — all copies declare the same version line.
4. **Concurrent edits to different copies** create silent races between fixes (e.g. a memory-leak fix landed in `phenoShared` does not propagate to `DataKit`).

## Goals

- Adopt `phenoShared/crates/phenotype-cache-adapter` as the single source of truth.
- Replace 4 redundant in-tree copies (counting both `PhenoProc` copies) with workspace dep references.
- Mark each deprecated copy with `DEPRECATED.md` + a top-of-file `// DEPRECATED` banner.
- Land 1 PR per host workspace (parallel-mergeable).
- Confirm cache-semantics parity (TTL, eviction order, capacity behavior) before swap.

## Non-Goals

- No code in spec, plan, or task documents (Planner-Agents-No-Code rule).
- No new feature work on cache API.
- No SemVer bump simultaneously with consolidation.

## Functional Requirements

| FR ID | Requirement |
|-------|-------------|
| FR-CCA-001 | Each non-canonical host workspace MUST replace its `crates/phenotype-cache-adapter/` member with a workspace dep targeting `phenoShared`. |
| FR-CCA-002 | Each deprecated copy MUST carry a `DEPRECATED.md` per ADR template. |
| FR-CCA-003 | Each deprecated copy's `src/lib.rs` MUST carry a `// DEPRECATED — see DEPRECATED.md` banner during transition. |
| FR-CCA-004 | Host workspace `cargo build --workspace` and `cargo test --workspace` MUST pass after the swap. |
| FR-CCA-005 | Cache-semantics parity tests (TTL expiry ordering, eviction policy, capacity bound) MUST pass on canonical for every consumer's existing behavior. |
| FR-CCA-006 | `cargo metadata --workspace` from any host MUST resolve exactly one `phenotype-cache-adapter` entry. |

## Acceptance Criteria

- AC1: `find repos -path '*/crates/phenotype-cache-adapter' -not -path '*phenoShared*' -not -path '*target*' -not -path '*.archive*' -not -path '*-wtrees*'` returns **0** results once host PRs merge.
- AC2: All 5 host workspaces build green against canonical `phenoShared`.
- AC3: Per-host PR documents the cache-semantics parity check results.
- AC4: ADR cross-reference appears in every host PR body.

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| L1/L2 eviction order differs between variants | Medium | High | Phase 2 design step runs a parity matrix before swap; differences become canonical-side feature flags. |
| Downstream consumers depend on a private API surface | Low | Medium | Phase 1 discovery captures all `use phenotype_cache_adapter::*` sites. |
| `PhenoProc` nested copy shadows root copy | Medium | Low | Single migration commit retires both; cargo resolution check catches double-resolution. |
