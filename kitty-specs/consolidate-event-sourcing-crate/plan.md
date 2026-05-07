# Plan: Consolidate `phenotype-event-sourcing` to canonical `phenoShared`

## Phased WBS

### Phase 1 — Discovery (D)
- D1: Enumerate exact copy paths across all KooshaPari org repos via `gh search code`.
- D2: Capture LOC + last-commit timestamp + `Cargo.toml` `version` for each copy.
- D3: Diff each variant's public API against canonical (function signatures, public types, feature flags).
- D4: List every internal Cargo.toml that declares `phenotype-event-sourcing = { path = ... }`.

**Exit criteria:** drift matrix complete; consumer-edge map complete; canonical baseline locked.

### Phase 2 — Design (D)
- D5: Pick canonical API as `phenoShared`'s current `main` (per ADR).
- D6: For each host: classify variant as `subset`, `superset`, or `divergent` of canonical.
- D7: For `superset` variants, decide per-symbol: lift into canonical (preferred) vs. host-side adapter shim.
- D8: Decide host strategy: `path` to a sibling clone, `git = "..."` pin, or workspace dep via shared registry.
- D9: Author per-host migration ticket with file-scoped changes (no code in plan).

**Exit criteria:** decision recorded in `tasks.md` per host; lift-vs-shim choices explicit.

### Phase 3 — Build (B)
- B1: For each host (parallel across hosts): replace `crates/phenotype-event-sourcing/` with canonical reference.
- B2: Add `DEPRECATED.md` and `// DEPRECATED` banner before removing the directory.
- B3: Update host workspace `Cargo.toml` `members` and `[workspace.dependencies]`.
- B4: Update every consuming crate's `Cargo.toml` to point at the canonical dep ref.

**Exit criteria:** each host workspace's git tree shows a single migration commit per WP unit; sibling crates updated.

### Phase 4 — Test/Validate (T)
- T1: `cargo build --workspace` per host — must pass.
- T2: `cargo test --workspace` per host — must pass.
- T3: `cargo metadata --workspace | jq '[.packages[] | select(.name=="phenotype-event-sourcing")] | length'` returns `1`.
- T4: Replay event-store hash-chain fixtures from each host against canonical — output bytes must match (or migration step is documented).
- T5: Run the ADR's `find` AC command — must return `0`.

**Exit criteria:** all gates green per host; AC1–AC5 satisfied.

### Phase 5 — Deploy/Handoff (H)
- H1: Open 1 PR per host workspace (`pheno`, `PhenoProc`, `DataKit`, `PhenoKits`, `hwLedger`) — all parallel-mergeable.
- H2: Add the "superseded" pointer to `cross-project-reuse-audit-2026-04-25.md` in the `phenoShared` PR.
- H3: Update AgilePlus WP status to `done` once all host PRs merge.
- H4: Schedule a 24-hour "post-merge probe" subagent to confirm no host workspace re-introduces a local copy.

**Exit criteria:** all host PRs merged; ADR AC1–AC5 fully satisfied; WP closed.

## Dependency DAG

| Phase | Task ID | Description | Depends On |
|-------|---------|-------------|------------|
| D | D1 | Enumerate copy paths | — |
| D | D2 | LOC + timestamp + version capture | D1 |
| D | D3 | API surface diff | D1 |
| D | D4 | Consumer-edge map | D1 |
| D | D5 | Lock canonical baseline | D2, D3 |
| D | D6 | Per-host classification | D3, D5 |
| D | D7 | Lift-vs-shim decisions | D6 |
| D | D8 | Per-host dep strategy | D4 |
| D | D9 | Per-host migration ticket | D6, D7, D8 |
| B | B1 | Replace crate copy (per host, parallel) | D9 |
| B | B2 | DEPRECATED.md + banner | D9 |
| B | B3 | Workspace `Cargo.toml` swap | B2 |
| B | B4 | Consumer Cargo.toml swap | B3 |
| T | T1 | `cargo build` per host | B4 |
| T | T2 | `cargo test` per host | T1 |
| T | T3 | `cargo metadata` single-resolution check | T1 |
| T | T4 | Hash-chain fixture replay | T2 |
| T | T5 | ADR `find` AC | B1 (all hosts) |
| H | H1 | Open per-host PR | T1, T2, T3, T4 |
| H | H2 | Audit-doc supersede pointer | H1 (any) |
| H | H3 | AgilePlus WP status `done` | H1 (all) |
| H | H4 | Post-merge probe subagent | H3 |

## Agent Time Estimates

| Phase | Tool calls | Parallel subagents | Wall clock |
|-------|-----------|---------------------|------------|
| Discovery (D1–D4) | 6–10 | 1 | 2–3 min |
| Design (D5–D9) | 4–6 | 1 | 1–2 min |
| Build (B1–B4) | 8–14 | up to 5 (one per host) | 4–6 min |
| Test/Validate (T1–T5) | 8–12 | up to 5 | 3–5 min |
| Deploy/Handoff (H1–H4) | 6–10 | 1–2 | 3–4 min |
| **Total** | **32–52** | **up to 5 concurrent** | **~13–20 min** |

## Parallelism Notes

- Phase 1 & 2 are sequential per the DAG.
- Phase 3 onward parallelizes across the 5 host workspaces (one dispatched subagent per host).
- The `phenoShared` audit-doc supersede pointer (H2) is independent and can dispatch as soon as any host PR opens.

## Cross-Project Reuse Opportunities

- This WP itself **is** the reuse-protocol execution: it deletes 5 redundant in-tree copies and consolidates onto a single shared module.
- Sibling WPs (`cache-adapter`, `state-machine`, `policy-engine`) follow the identical playbook — implementers should share scripts/checklists across the 4 WPs.
