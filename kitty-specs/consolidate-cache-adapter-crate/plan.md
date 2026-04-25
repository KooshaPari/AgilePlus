# Plan: Consolidate `phenotype-cache-adapter` to canonical `phenoShared`

## Phased WBS

### Phase 1 — Discovery (D)
- D1: Enumerate exact copy paths for `phenotype-cache-adapter` across all KooshaPari org repos.
- D2: Capture LOC, last-commit timestamp, declared `Cargo.toml` `version` per copy.
- D3: Diff each variant's public API + cache-semantics constants (TTL default, capacity default, L1/L2 split policy).
- D4: List every internal `Cargo.toml` declaring `phenotype-cache-adapter = { path = ... }` plus all `use phenotype_cache_adapter::*` import sites.

**Exit criteria:** drift matrix complete; consumer-edge map complete; cache-semantics parity matrix in hand.

### Phase 2 — Design (D)
- D5: Lock canonical baseline = `phenoShared@HEAD` at task-start (record SHA).
- D6: Per-host classification (`subset` / `superset` / `divergent`).
- D7: Cache-semantics decision: any divergent eviction or TTL behavior becomes a canonical-side feature flag, never a host fork.
- D8: Per-host dep-strategy decision (`path` to sibling clone, `git = "..."` pin, or shared registry).
- D9: Author per-host migration ticket.

**Exit criteria:** decisions recorded in `tasks.md`; no host left as silent divergent.

### Phase 3 — Build (B)
- B1: Per host (parallel): replace `crates/phenotype-cache-adapter/` with canonical reference.
- B2: Add `DEPRECATED.md` and `// DEPRECATED` banner before directory removal.
- B3: Update host workspace `Cargo.toml` `members` and `[workspace.dependencies]`.
- B4: Update consumer crate `Cargo.toml`s.

**Exit criteria:** single migration commit per host; consumer crates updated.

### Phase 4 — Test/Validate (T)
- T1: `cargo build --workspace` per host.
- T2: `cargo test --workspace` per host.
- T3: `cargo metadata --workspace | jq '[.packages[] | select(.name=="phenotype-cache-adapter")] | length'` returns `1`.
- T4: Cache-semantics parity tests (TTL, eviction order, capacity bound) — run on canonical against per-host fixture suites.
- T5: ADR `find` AC returns `0`.

**Exit criteria:** all gates green; AC1–AC4 satisfied.

### Phase 5 — Deploy/Handoff (H)
- H1: Open 1 PR per host (`pheno`, `PhenoProc`, `DataKit`, `PhenoKits/HexaKit`) — parallel-mergeable.
- H2: Add audit-doc supersede pointer (shared with sibling WPs; only one WP needs to do this).
- H3: Update AgilePlus WP status `done`.
- H4: Schedule 24-hour post-merge probe subagent.

**Exit criteria:** all host PRs merged; ADR ACs satisfied; WP closed.

## Dependency DAG

| Phase | Task ID | Description | Depends On |
|-------|---------|-------------|------------|
| D | D1 | Enumerate copy paths | — |
| D | D2 | LOC + timestamp + version | D1 |
| D | D3 | API + semantics diff | D1 |
| D | D4 | Consumer-edge map | D1 |
| D | D5 | Lock canonical baseline | D2, D3 |
| D | D6 | Per-host classification | D3, D5 |
| D | D7 | Semantics-parity flagging | D6 |
| D | D8 | Per-host dep strategy | D4 |
| D | D9 | Per-host migration ticket | D6, D7, D8 |
| B | B1 | Replace crate copy (per host, parallel) | D9 |
| B | B2 | DEPRECATED.md + banner | D9 |
| B | B3 | Workspace `Cargo.toml` swap | B2 |
| B | B4 | Consumer Cargo.toml swap | B3 |
| T | T1 | `cargo build` per host | B4 |
| T | T2 | `cargo test` per host | T1 |
| T | T3 | Single-resolution check | T1 |
| T | T4 | Cache-semantics parity | T2 |
| T | T5 | ADR `find` AC | B1 (all hosts) |
| H | H1 | Open per-host PR | T1, T2, T3, T4 |
| H | H2 | Audit-doc supersede pointer | H1 (any) |
| H | H3 | AgilePlus WP `done` | H1 (all) |
| H | H4 | Post-merge probe | H3 |

## Agent Time Estimates

| Phase | Tool calls | Parallel subagents | Wall clock |
|-------|-----------|---------------------|------------|
| Discovery | 6–10 | 1 | 2–3 min |
| Design | 4–6 | 1 | 1–2 min |
| Build | 8–12 | up to 4 | 4–6 min |
| Test/Validate | 8–12 | up to 4 | 3–5 min |
| Deploy/Handoff | 6–8 | 1–2 | 3–4 min |
| **Total** | **32–48** | **up to 4 concurrent** | **~13–20 min** |

## Parallelism Notes

- Phase 1 and Phase 2 are sequential.
- Phase 3 onward parallelizes across the 4 host workspaces (one subagent per host, counting `PhenoProc`'s root + nested as a single host scope).
- Cache-semantics parity (T4) can dispatch as a single shared subagent rather than per-host.

## Cross-Project Reuse Opportunities

- Reuse the harness scripts authored for the sibling event-sourcing WP (drift matrix + per-host migration ticket templates).
- The cache-semantics parity test suite, once authored on canonical, is reusable for any future cache adapter work.
