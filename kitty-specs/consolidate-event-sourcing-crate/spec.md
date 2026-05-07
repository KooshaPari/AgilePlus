---
spec_id: consolidate-event-sourcing-crate
status: specified
created: 2026-04-25
last_audit: 2026-04-25
owners: phenotype-org / shared-platform
adr: docs/governance/shared-crates-canonical-home-adr-2026-04.md
crate: phenotype-event-sourcing
canonical_home: github.com/KooshaPari/phenoShared/crates/phenotype-event-sourcing
---

# Consolidate `phenotype-event-sourcing` to `phenoShared`

## Meta

- **ID**: consolidate-event-sourcing-crate
- **Title**: Migrate all redundant `phenotype-event-sourcing` copies to canonical `phenoShared` home
- **Scope**: Crate-level (cross-repo); 6 host workspaces; 0 external Cargo consumers
- **Priority**: P1 — High (unblocks API drift remediation across 6 repos)
- **Depends On**: `shared-crates-canonical-home-adr-2026-04` (Accepted, PhenoKits#31, merged 2026-04-25)
- **Sibling WPs (parallel-mergeable)**: `consolidate-cache-adapter-crate`, `consolidate-state-machine-crate`, `consolidate-policy-engine-crate`

## Context

The 2026-04-25 cross-project reuse audit identified `phenotype-event-sourcing` as physically duplicated across **6 host workspaces** with measurable API drift:

| Host workspace | LOC of `src/lib.rs` | Last touched |
|----------------|---------------------|--------------|
| `phenoShared` (canonical, per ADR) | 42 | 2026-04-25 (today) |
| `pheno` | 26 | 2026-03-31 |
| `PhenoProc` (root copy) | 57 | 2026-04-03 |
| `PhenoProc` (nested `crates/phenotype-shared/crates/`) | 57 | 2026-04-03 |
| `DataKit/rust` | 57 | 2026-04-05 |
| `PhenoKits/HexaKit` | 26 | 2026-03-31 |
| `hwLedger/vendor` | (vendored snapshot) | snapshot |

External (cross-workspace) consumers: **0**. All current dependency edges are local `path = "crates/..."` references inside their host workspaces.

## Problem Statement

1. **Three-way API drift** (26 / 42 / 57 LOC variants) means a fix to one variant does not propagate; security/correctness regressions are silently re-introduced.
2. **Six host workspaces** each compile their own copy, increasing build time and producing distinct on-disk schemas (event-store hash chains may diverge).
3. **No SemVer signal** to consumers — all variants declare `version = "0.2.0"` so cargo cannot distinguish them.
4. **`phenoShared` is the only home with same-day evolution commits**, yet five other homes still ship divergent copies.

## Goals

- Adopt `phenoShared/crates/phenotype-event-sourcing` as the **single source of truth** per the ADR.
- Replace all 5 redundant in-tree copies + 1 vendored copy with workspace dep references to the canonical crate.
- Mark each deprecated copy with `DEPRECATED.md` + a top-of-file `// DEPRECATED` banner before physical removal.
- Land 1 PR per host workspace (parallel-mergeable since hosts are independent Cargo workspaces).
- Reconcile API drift: pick the canonical baseline as whatever `phenoShared` ships at task-start (per ADR §"Out of scope"), and document any consumer-side adapter needed in `tasks.md`.

## Non-Goals

- No new feature work on the crate API.
- No re-purposing of the dormant `KooshaPari/phenotype-shared` repo (separate org-hygiene WP).
- No code in this spec, plan, or task documents (Planner-Agents-No-Code rule).
- No simultaneous SemVer bump — the canonical crate retains its current version line.

## Functional Requirements

| FR ID | Requirement |
|-------|-------------|
| FR-CES-001 | Each non-canonical host workspace MUST replace its `crates/phenotype-event-sourcing/` member with a workspace dep targeting `phenoShared`. |
| FR-CES-002 | Each deprecated copy MUST carry a `DEPRECATED.md` (per ADR §"Deprecation plan" template) before physical deletion. |
| FR-CES-003 | Each deprecated copy's `src/lib.rs` MUST carry a `// DEPRECATED — see DEPRECATED.md` banner during the transition window. |
| FR-CES-004 | Host workspace `cargo build --workspace` and `cargo test --workspace` MUST pass after the swap. |
| FR-CES-005 | `cargo metadata --workspace` from any host MUST resolve exactly one `phenotype-event-sourcing` entry across the dep graph. |
| FR-CES-006 | The audit doc `cross-project-reuse-audit-2026-04-25.md` MUST receive a "superseded for canonical-home decision" pointer to the ADR. |

## Acceptance Criteria

- AC1: `find repos -path '*/crates/phenotype-event-sourcing' -not -path '*phenoShared*' -not -path '*target*' -not -path '*.archive*' -not -path '*-wtrees*'` returns **0** results once all host PRs merge.
- AC2: All 6 host workspaces (`pheno`, `PhenoProc`, `DataKit`, `PhenoKits/HexaKit`, `hwLedger`) build green against canonical `phenoShared`.
- AC3: Each host PR includes a `DEPRECATED.md` for its removed copy in the diff (during transition) or a clean removal commit referencing the ADR.
- AC4: API-drift reconciliation note (which variant's surface "won" and any host-side shim) is captured in `tasks.md` and per-host PR description.
- AC5: ADR cross-reference appears in every host PR body.

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| 57-LOC variants depend on API surface absent from 42-LOC canonical | Medium | High | Phase 2 design step enumerates the API delta and either lifts missing surface into canonical or wraps in host-side adapter. |
| Hash-chain on-disk schema diverges between variants | Low | High | Phase 4 validation compares replay of fixture events; mismatch triggers explicit migration step. |
| Host workspace has unrelated red CI; merge blocked | Medium | Medium | Use HOOKS_SKIP=1 on pre-existing failures; document in PR. |
| `hwLedger/vendor` snapshot is intentionally pinned | Low | Medium | If vendored to satisfy compliance, swap to git-pinned canonical commit instead of removal. |
