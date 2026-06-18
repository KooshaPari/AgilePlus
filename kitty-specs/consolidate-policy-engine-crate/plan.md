# Plan: Consolidate `phenotype-policy-engine` to canonical `phenoShared`

## Phased WBS

### Phase 1 — Discovery (D)
- D1: Enumerate exact copy paths for `phenotype-policy-engine` across all KooshaPari org repos.
- D2: Capture LOC, last-commit, version per copy.
- D3: Diff each variant's public API + TOML schema + decision-combination strategy.
- D4: Enumerate consumer-edge map (`path =` refs and `use phenotype_policy_engine::*` sites).
- D5: **Special:** for `AuthKit`, enumerate every authz-specific symbol absent from canonical (role hierarchy, scope-bounded predicates, subject matchers, etc.).

**Exit criteria:** drift matrix; consumer-edge map; `AuthKit` divergence inventory in hand.

### Phase 2 — Design (D)
- D6: Lock canonical baseline = `phenoShared@HEAD`.
- D7: Per-host classification (`subset` / `superset` / `divergent`).
- D8: For `AuthKit` divergent symbols: design a canonical-side feature flag (`feature = ["auth-extensions"]`) lifting those symbols into canonical without forking.
- D9: TOML schema reconciliation strategy: any divergent schema becomes either (a) a canonical-side schema-version field with backward-compat parsing, or (b) explicit consumer-side migration.
- D10: Per-host dep strategy + per-host migration ticket.

**Exit criteria:** decisions recorded in `tasks.md`; lift-vs-flag plan explicit for `AuthKit`; TOML compatibility plan explicit.

### Phase 3 — Build (B)
- B1: Land canonical-side `feature = ["auth-extensions"]` on `phenoShared` (gates `AuthKit` swap).
- B2: Land canonical-side TOML schema-version handling if needed.
- B3: Per host (parallel for non-`AuthKit`): replace `crates/phenotype-policy-engine/` with canonical reference.
- B4: Add `DEPRECATED.md` + `// DEPRECATED` banner before directory removal.
- B5: Update host workspace `Cargo.toml` and consumer crates' `Cargo.toml`.
- B6: For `AuthKit`: swap with `features = ["auth-extensions"]` enabled.

**Exit criteria:** single migration commit per host; canonical feature flag landed.

### Phase 4 — Test/Validate (T)
- T1: `cargo build --workspace` per host.
- T2: `cargo test --workspace` per host.
- T3: `cargo metadata --workspace` single-resolution check.
- T4: Policy-evaluation parity: load every consumer's existing TOML fixtures and compare decision outputs against canonical.
- T5: ADR `find` AC returns `0`.

**Exit criteria:** all gates green; AC1–AC5 satisfied.

### Phase 5 — Deploy/Handoff (H)
- H1: Open 1 PR per host (`pheno`, `PhenoProc`, `HexaKit`, `PhenoKits/HexaKit`, `AuthKit`) — non-`AuthKit` PRs parallel-mergeable; `AuthKit` after canonical feature-flag PR.
- H2: Audit-doc supersede pointer (coordinate via WP-CES-08).
- H3: AgilePlus WP `done`.
- H4: 24-hour post-merge probe.

**Exit criteria:** all host PRs merged; ADR ACs satisfied.

## Dependency DAG

| Phase | Task ID | Description | Depends On |
|-------|---------|-------------|------------|
| D | D1 | Enumerate copy paths | — |
| D | D2 | LOC + timestamp + version | D1 |
| D | D3 | API + TOML + decision-strategy diff | D1 |
| D | D4 | Consumer-edge map | D1 |
| D | D5 | AuthKit divergence inventory | D3 |
| D | D6 | Lock canonical baseline | D2, D3 |
| D | D7 | Per-host classification | D3, D6 |
| D | D8 | AuthKit feature-flag plan | D5, D7 |
| D | D9 | TOML schema reconciliation | D3, D7 |
| D | D10 | Per-host migration ticket | D7, D8, D9 |
| B | B1 | Land canonical auth-extensions feature | D8 |
| B | B2 | Land canonical TOML schema-version handling | D9 |
| B | B3 | Replace crate copy (non-AuthKit, parallel) | D10 |
| B | B4 | DEPRECATED.md + banner | D10 |
| B | B5 | Workspace + consumer Cargo.toml swap | B4 |
| B | B6 | AuthKit swap with feature flag | B1, B5 |
| T | T1 | `cargo build` per host | B5, B6 |
| T | T2 | `cargo test` per host | T1 |
| T | T3 | Single-resolution check | T1 |
| T | T4 | Policy-evaluation parity | T2 |
| T | T5 | ADR `find` AC | B3, B6 (all hosts) |
| H | H1 | Open per-host PR | T1, T2, T3, T4 |
| H | H2 | Audit-doc supersede | H1 (any) |
| H | H3 | AgilePlus WP `done` | H1 (all) |
| H | H4 | Post-merge probe | H3 |

## Agent Time Estimates

| Phase | Tool calls | Parallel subagents | Wall clock |
|-------|-----------|---------------------|------------|
| Discovery (incl. AuthKit inventory) | 8–14 | 1 | 3–5 min |
| Design (incl. feature-flag + TOML plan) | 6–10 | 1 | 2–4 min |
| Build (incl. canonical feature flag + TOML schema) | 14–22 | up to 5 | 7–10 min |
| Test/Validate | 10–14 | up to 5 | 4–6 min |
| Deploy/Handoff | 6–10 | 1–2 | 3–4 min |
| **Total** | **44–70** | **up to 5 concurrent** | **~19–29 min** |

## Parallelism Notes

- Canonical feature-flag + TOML schema PRs (B1, B2) **must merge before** `AuthKit` host swap (B6).
- All non-`AuthKit` host migrations are fully parallel.
- Discovery's `AuthKit`-specific divergence inventory (D5) is on the critical path.

## Cross-Project Reuse Opportunities

- `feature = ["auth-extensions"]` reuses the `feature = ["resilience-hooks"]` precedent from sibling `state-machine` WP.
- Reuse harness scripts (drift-matrix capture, per-host migration ticket templates, ADR `find` AC check) from sibling WPs.
- The TOML schema-version pattern, once landed, becomes the precedent for any future config-bearing crate consolidation.
