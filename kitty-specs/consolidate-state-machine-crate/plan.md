# Plan: Consolidate `phenotype-state-machine` to canonical `phenoShared`

## Phased WBS

### Phase 1 — Discovery (D)
- D1: Enumerate exact copy paths for `phenotype-state-machine` across all KooshaPari org repos.
- D2: Capture LOC, last-commit, declared version per copy.
- D3: Diff each variant's public API + transition-guard contract surface.
- D4: List every internal `Cargo.toml` declaring `phenotype-state-machine = { path = ... }` plus `use phenotype_state_machine::*` import sites.
- D5: **Special:** for `ResilienceKit`, enumerate any domain-specific symbol absent from canonical (retry hooks, circuit-breaker integration, backoff state types).

**Exit criteria:** drift matrix complete; consumer-edge map complete; `ResilienceKit` divergence inventory in hand.

### Phase 2 — Design (D)
- D6: Lock canonical baseline = `phenoShared@HEAD`.
- D7: Per-host classification (`subset` / `superset` / `divergent`).
- D8: For `ResilienceKit` divergent symbols: design a canonical-side feature flag (`feature = ["resilience-hooks"]`) that lifts those symbols into canonical without forking.
- D9: Per-host dep strategy.
- D10: Per-host migration ticket.

**Exit criteria:** decisions recorded in `tasks.md`; lift-vs-flag plan explicit for `ResilienceKit`.

### Phase 3 — Build (B)
- B1: Land the canonical-side feature flag on `phenoShared` first (gates `ResilienceKit` swap).
- B2: Per host (parallel for non-`ResilienceKit`): replace `crates/phenotype-state-machine/` with canonical reference.
- B3: Add `DEPRECATED.md` + `// DEPRECATED` banner before directory removal.
- B4: Update host workspace `Cargo.toml` and consumer crates' `Cargo.toml`.
- B5: For `ResilienceKit`: swap with `features = ["resilience-hooks"]` enabled.

**Exit criteria:** single migration commit per host; canonical feature flag landed.

### Phase 4 — Test/Validate (T)
- T1: `cargo build --workspace` per host.
- T2: `cargo test --workspace` per host.
- T3: `cargo metadata --workspace` single-resolution check.
- T4: Transition-guard parity tests using each host's existing fixtures.
- T5: ADR `find` AC returns `0`.

**Exit criteria:** all gates green; AC1–AC4 satisfied.

### Phase 5 — Deploy/Handoff (H)
- H1: Open 1 PR per host (`pheno`, `PhenoProc`, `ResilienceKit`, `HexaKit`, `PhenoKits/HexaKit`) — non-`ResilienceKit` PRs parallel-mergeable; `ResilienceKit` after canonical feature-flag PR.
- H2: Audit-doc supersede pointer (coordinate via WP-CES-08 — only one sibling needs to land it).
- H3: AgilePlus WP `done`.
- H4: 24-hour post-merge probe.

**Exit criteria:** all host PRs merged; ADR ACs satisfied.

## Dependency DAG

| Phase | Task ID | Description | Depends On |
|-------|---------|-------------|------------|
| D | D1 | Enumerate copy paths | — |
| D | D2 | LOC + timestamp + version | D1 |
| D | D3 | API + guard-contract diff | D1 |
| D | D4 | Consumer-edge map | D1 |
| D | D5 | ResilienceKit divergence inventory | D3 |
| D | D6 | Lock canonical baseline | D2, D3 |
| D | D7 | Per-host classification | D3, D6 |
| D | D8 | ResilienceKit feature-flag plan | D5, D7 |
| D | D9 | Per-host dep strategy | D4 |
| D | D10 | Per-host migration ticket | D7, D8, D9 |
| B | B1 | Land canonical feature flag | D8 |
| B | B2 | Replace crate copy (non-ResilienceKit hosts, parallel) | D10 |
| B | B3 | DEPRECATED.md + banner | D10 |
| B | B4 | Workspace + consumer Cargo.toml swap | B3 |
| B | B5 | ResilienceKit swap with feature flag | B1, B4 |
| T | T1 | `cargo build` per host | B4, B5 |
| T | T2 | `cargo test` per host | T1 |
| T | T3 | Single-resolution check | T1 |
| T | T4 | Transition-guard parity | T2 |
| T | T5 | ADR `find` AC | B2, B5 (all hosts) |
| H | H1 | Open per-host PR | T1, T2, T3, T4 |
| H | H2 | Audit-doc supersede | H1 (any) |
| H | H3 | AgilePlus WP `done` | H1 (all) |
| H | H4 | Post-merge probe | H3 |

## Agent Time Estimates

| Phase | Tool calls | Parallel subagents | Wall clock |
|-------|-----------|---------------------|------------|
| Discovery (incl. ResilienceKit inventory) | 8–12 | 1 | 3–4 min |
| Design (incl. feature-flag plan) | 5–8 | 1 | 2–3 min |
| Build (incl. canonical feature flag) | 12–18 | up to 5 | 6–8 min |
| Test/Validate | 10–14 | up to 5 | 4–6 min |
| Deploy/Handoff | 6–10 | 1–2 | 3–4 min |
| **Total** | **41–62** | **up to 5 concurrent** | **~18–25 min** |

## Parallelism Notes

- The canonical feature-flag PR (B1) **must merge before** `ResilienceKit` host swap (B5).
- All non-`ResilienceKit` host migrations are fully parallel.
- Discovery's `ResilienceKit`-specific divergence inventory (D5) is on the critical path — dispatch first.

## Cross-Project Reuse Opportunities

- The `feature = ["resilience-hooks"]` pattern, once landed on `phenoShared`, becomes the precedent for any future "domain-specific lift" cases (e.g. `AuthKit`'s policy-engine divergence in the sibling WP).
- Reuse the harness scripts from sibling WPs.
