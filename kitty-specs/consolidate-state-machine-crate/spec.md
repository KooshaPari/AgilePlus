---
spec_id: consolidate-state-machine-crate
status: specified
created: 2026-04-25
last_audit: 2026-04-25
owners: phenotype-org / shared-platform
adr: docs/governance/shared-crates-canonical-home-adr-2026-04.md
crate: phenotype-state-machine
canonical_home: github.com/KooshaPari/phenoShared/crates/phenotype-state-machine
---

# Consolidate `phenotype-state-machine` to `phenoShared`

## Meta

- **ID**: consolidate-state-machine-crate
- **Title**: Migrate all redundant `phenotype-state-machine` copies to canonical `phenoShared` home
- **Scope**: Crate-level (cross-repo); 6 host workspaces; 0 external Cargo consumers
- **Priority**: P1 — High (FSM transition guards underpin retry/circuit-breaker semantics in `ResilienceKit`)
- **Depends On**: `shared-crates-canonical-home-adr-2026-04` (Accepted, PhenoKits#31, merged 2026-04-25)
- **Sibling WPs (parallel-mergeable)**: `consolidate-event-sourcing-crate`, `consolidate-cache-adapter-crate`, `consolidate-policy-engine-crate`

## Context

Per the canonical-home ADR, `phenotype-state-machine` (generic FSM with transition guards) is duplicated across **6 host workspaces**:

| Host workspace | Notes |
|----------------|-------|
| `phenoShared` (canonical) | Same-day evolution; ADR-elected canonical |
| `pheno` | snapshot, 2026-03-31 |
| `PhenoProc` (root copy) | 2026-04-03 |
| `PhenoProc` (nested `crates/phenotype-shared/crates/`) | 2026-04-03 |
| `ResilienceKit/rust` | snapshot — special: domain-specific divergence possible |
| `HexaKit/crates` | snapshot, 2026-03-31 |
| `PhenoKits/HexaKit` | snapshot, 2026-03-31 |

External (cross-workspace) consumers: **0**. The ADR migration matrix flags `ResilienceKit` as the highest-risk host: "snapshot; verify no domain-specific divergence" — `ResilienceKit` may have augmented the FSM with retry-policy or circuit-breaker hooks that are absent from canonical.

## Problem Statement

1. **Six divergent FSM cores** mean transition-guard semantics are not invariant across consumers — a bug fixed in one variant silently re-occurs elsewhere.
2. **`ResilienceKit` may have lifted FSM into a domain-specific variant**, which (per the ADR) MUST be brought back as a feature flag in canonical, not retained as a fork.
3. **`HexaKit/crates` and `PhenoKits/HexaKit` ship parallel state-machine copies** — likely identical, but each must be retired explicitly.
4. **No SemVer signal** — all variants share the same version line.

## Goals

- Adopt `phenoShared/crates/phenotype-state-machine` as the single source of truth.
- Replace 5 redundant in-tree copies (counting both `PhenoProc` copies) with workspace dep references.
- For `ResilienceKit`: capture any domain-specific divergence as a canonical-side feature flag (e.g. `feature = ["resilience-hooks"]`), not a fork.
- Mark each deprecated copy with `DEPRECATED.md` + a top-of-file `// DEPRECATED` banner.
- Land 1 PR per host workspace (parallel-mergeable across non-`ResilienceKit` hosts; `ResilienceKit` is sequential after canonical lift if needed).

## Non-Goals

- No code in spec, plan, or task documents.
- No SemVer bump simultaneously with consolidation.
- No retro-fitting of new FSM features unrelated to drift reconciliation.

## Functional Requirements

| FR ID | Requirement |
|-------|-------------|
| FR-CSM-001 | Each non-canonical host workspace MUST replace its `crates/phenotype-state-machine/` member with a workspace dep targeting `phenoShared`. |
| FR-CSM-002 | Each deprecated copy MUST carry a `DEPRECATED.md` per ADR template. |
| FR-CSM-003 | Each deprecated copy's `src/lib.rs` MUST carry a `// DEPRECATED — see DEPRECATED.md` banner during transition. |
| FR-CSM-004 | Host workspace `cargo build --workspace` and `cargo test --workspace` MUST pass after the swap. |
| FR-CSM-005 | Any divergence in `ResilienceKit`'s FSM (e.g. retry hooks, circuit-breaker integration) MUST be lifted into canonical as an opt-in feature flag — never retained as a fork. |
| FR-CSM-006 | `cargo metadata --workspace` from any host MUST resolve exactly one `phenotype-state-machine` entry. |
| FR-CSM-007 | FSM transition-guard parity tests MUST pass on canonical for every consumer's existing fixtures. |

## Acceptance Criteria

- AC1: `find repos -path '*/crates/phenotype-state-machine' -not -path '*phenoShared*' -not -path '*target*' -not -path '*.archive*' -not -path '*-wtrees*'` returns **0** results once host PRs merge.
- AC2: All 6 host workspaces build green against canonical `phenoShared`.
- AC3: `ResilienceKit`'s domain-specific behaviors are reachable via a canonical-side feature flag (or explicitly removed as dead code with a `ResilienceKit`-side commit referencing this WP).
- AC4: ADR cross-reference appears in every host PR body.

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `ResilienceKit` FSM has retry hooks absent from canonical | High | High | Phase 2 design step enumerates the delta; lift into canonical under `feature = ["resilience-hooks"]` before host swap. |
| `HexaKit/crates` and `PhenoKits/HexaKit` already drift between each other | Medium | Medium | Treat as two distinct host migrations; cargo metadata check catches any leftover. |
| Transition-guard semantics differ silently | Medium | High | Phase 4 parity tests using each host's existing fixture suite. |
| `ResilienceKit` host is a snapshot repo with no active maintainer | Low | Medium | If unmaintained, mark host's copy as deprecated and pin to canonical SHA without further test gating. |
