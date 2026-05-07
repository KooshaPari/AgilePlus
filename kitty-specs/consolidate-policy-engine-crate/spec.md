---
spec_id: consolidate-policy-engine-crate
status: specified
created: 2026-04-25
last_audit: 2026-04-25
owners: phenotype-org / shared-platform
adr: docs/governance/shared-crates-canonical-home-adr-2026-04.md
crate: phenotype-policy-engine
canonical_home: github.com/KooshaPari/phenoShared/crates/phenotype-policy-engine
---

# Consolidate `phenotype-policy-engine` to `phenoShared`

## Meta

- **ID**: consolidate-policy-engine-crate
- **Title**: Migrate all redundant `phenotype-policy-engine` copies to canonical `phenoShared` home
- **Scope**: Crate-level (cross-repo); 6 host workspaces; 0 external Cargo consumers
- **Priority**: P1 — High (`AuthKit`'s authz semantics MUST be invariant; multi-copy drift is a security risk)
- **Depends On**: `shared-crates-canonical-home-adr-2026-04` (Accepted, PhenoKits#31, merged 2026-04-25)
- **Sibling WPs (parallel-mergeable)**: `consolidate-event-sourcing-crate`, `consolidate-cache-adapter-crate`, `consolidate-state-machine-crate`

## Context

Per the canonical-home ADR, `phenotype-policy-engine` (rule-based policy evaluation with TOML config) is duplicated across **6 host workspaces**:

| Host workspace | Notes |
|----------------|-------|
| `phenoShared` (canonical) | Same-day evolution; ADR-elected canonical; ships sibling `phenotype-policy-engine-py` for Python consumers |
| `pheno` | snapshot, 2026-03-31 |
| `PhenoProc` (root copy) | 2026-04-03 |
| `PhenoProc` (nested `crates/phenotype-shared/crates/`) | 2026-04-03 |
| `HexaKit/crates` | snapshot, 2026-03-31 |
| `PhenoKits/HexaKit` | snapshot, 2026-03-31 |
| `AuthKit/rust` | snapshot — special: auth-specific extensions likely |

External (cross-workspace) consumers: **0**. The ADR migration matrix flags `AuthKit` explicitly: "may have auth-specific extensions — capture as feature flag in canonical, do not fork." This mirrors the `ResilienceKit` pattern in the sibling `state-machine` WP.

## Problem Statement

1. **Six divergent policy evaluators** mean authz decisions are not invariant across consumers — a CVE patched in one variant remains exploitable in five.
2. **`AuthKit` likely augments the rule grammar or adds auth-specific predicates** (e.g. role hierarchy, scope-bounded subject matching) that MUST be lifted into canonical as a feature flag, not retained as a fork.
3. **TOML schema drift** between copies — a policy config that parses cleanly in one consumer may silently fail-open or fail-closed in another.
4. **No SemVer signal** — all variants share the same version line.

## Goals

- Adopt `phenoShared/crates/phenotype-policy-engine` as the single source of truth.
- Replace 5 redundant in-tree copies (counting both `PhenoProc` copies) with workspace dep references.
- For `AuthKit`: capture any auth-specific extensions (role hierarchy, scope-bounded predicates, etc.) as a canonical-side feature flag (e.g. `feature = ["auth-extensions"]`), not a fork.
- Mark each deprecated copy with `DEPRECATED.md` + a top-of-file `// DEPRECATED` banner.
- Land 1 PR per host workspace; non-`AuthKit` PRs are parallel-mergeable.

## Non-Goals

- No code in spec, plan, or task documents.
- No new policy DSL features unrelated to `AuthKit`-divergence reconciliation.
- No SemVer bump simultaneously with consolidation.
- No reconciliation of the sibling `phenotype-policy-engine-py` — that is a separate Python-side WP.

## Functional Requirements

| FR ID | Requirement |
|-------|-------------|
| FR-CPE-001 | Each non-canonical host workspace MUST replace its `crates/phenotype-policy-engine/` member with a workspace dep targeting `phenoShared`. |
| FR-CPE-002 | Each deprecated copy MUST carry a `DEPRECATED.md` per ADR template. |
| FR-CPE-003 | Each deprecated copy's `src/lib.rs` MUST carry a `// DEPRECATED — see DEPRECATED.md` banner during transition. |
| FR-CPE-004 | Host workspace `cargo build --workspace` and `cargo test --workspace` MUST pass after the swap. |
| FR-CPE-005 | Any auth-specific extension in `AuthKit`'s policy engine MUST be lifted into canonical as an opt-in feature flag — never retained as a fork. |
| FR-CPE-006 | `cargo metadata --workspace` from any host MUST resolve exactly one `phenotype-policy-engine` entry. |
| FR-CPE-007 | Policy-evaluation parity tests (TOML config parse + decision outputs for each consumer's existing rule sets) MUST pass on canonical. |

## Acceptance Criteria

- AC1: `find repos -path '*/crates/phenotype-policy-engine' -not -path '*phenoShared*' -not -path '*target*' -not -path '*.archive*' -not -path '*-wtrees*'` returns **0** results once host PRs merge.
- AC2: All 6 host workspaces build green against canonical `phenoShared`.
- AC3: `AuthKit`'s auth-specific behaviors are reachable via a canonical-side feature flag (or explicitly retired as dead code).
- AC4: Per-host PR documents the policy-evaluation parity check results.
- AC5: ADR cross-reference appears in every host PR body.

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `AuthKit` policy engine has authz-critical predicates absent from canonical | High | Critical | Phase 2 enumerates every `AuthKit`-only symbol; lift into canonical under `feature = ["auth-extensions"]` before host swap. |
| TOML schema differs silently — same config parses to different rule sets | Medium | High | Phase 4 parity tests load every consumer's existing policy fixtures and compare decision outputs. |
| Decision outputs diverge under edge cases (deny-overrides vs. permit-overrides) | Medium | Critical | Capture decision-combination strategy as canonical config, not host fork. |
| `AuthKit` is unmaintained snapshot | Low | Medium | If unmaintained, mark host's copy as deprecated and pin without test-gating; flag for archive. |
