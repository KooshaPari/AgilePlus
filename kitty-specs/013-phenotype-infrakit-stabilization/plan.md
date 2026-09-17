# Plan: phenotype-infrakit Stabilization — Consolidate 19 Infrastructure Crates

> **Inputs:** [`spec.md`](./spec.md), [`tasks.md`](./tasks.md), audit memo `spec.md` §"Audit Update — 2026-04-02".
> **Target repo:** `repos/phenotype-infrakit/` (origin: `github.com/KooshaPari/phenotype-infrakit`).
> **Out of scope:** new crates, language migrations, behavioral rewrites, deprecations beyond what consolidation forces.

---

## Phase Map

| Phase | Goal | Deliverables | Exit Gate |
|-------|------|--------------|-----------|
| P0 Discovery | Pin down current state — open PRs, dirty worktrees, stale branches, broken builds. | Live audit memo (counts of PRs/worktrees/branches/disk), per-crate maturity matrix, duplicate-functionality matrix, dependency graph. | `cargo metadata --workspace` clean; PR/worktree/branch counts recorded; matrix lists each of 19 crates. |
| P1 Design | Decide consolidation targets, workspace shape (single vs 3-split), error/Cargo conventions. | ADR for workspace shape, merge map (`shared→contracts`, `PolicyStack→policy-engine`, error/ports collapses), publish-name reservation list. | ADR merged to `phenotype-infrakit/docs/adr/`; merge map referenced by every WP-003/WP-004 task. |
| P2 Build | Land the consolidation — close P0 backlog, migrate crates, merge duplicates, normalize errors/MSRV. | Single workspace builds clean; duplicates removed; preflight backlog (10 PRs + 2 worktrees + ~20 stale branches) drained. | `cargo build --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check` all green on `main`. |
| P3 Test/Validate | Stabilize APIs and prove ≥80 % coverage; cross-crate integration tests; CHANGELOG/semver. | Test reports per crate; CHANGELOGs at `0.1.0`; rustdoc landing; tarpaulin/llvm-cov report. | Coverage ≥80 % overall, ≥90 % for crypto/auth crates; `cargo doc --workspace --no-deps` clean. |
| P4 Deploy/Handoff | Publish to crates.io / PyPI / npm; CI on tag; final security audit; downstream callers updated. | Published versions; tag-driven publish workflow; `cargo audit` / `pip-audit` / `npm audit` reports; updated downstream `Cargo.toml`/`pyproject.toml`/`package.json`. | All targeted packages installable from a clean machine; downstream `cargo build` green. |

---

## Work Packages (DAG)

| Phase | WP ID | Description | Depends On |
|-------|-------|-------------|------------|
| P0 | WP-000 | Preflight: drain 10 open PRs (`#544–#563`), resolve 2 dirty worktrees (`cache-adapter-impl`, `phenotype-crypto-complete-v2`), prune ~20 stale branches, run `cargo clean` to reclaim ~1.3 GB. | — |
| P0 | WP-001 | Audit all 19 crates: API surface, test coverage, doc quality, dependency graph, duplicated-functionality matrix, language-fit assessment. | WP-000 |
| P1 | WP-002a | Consolidation design ADR: single workspace vs 3-split (`core` / `runtime` / `tools`); merge map; error-handling conventions (`thiserror` + `#[from]`); MSRV; workspace dep policy. | WP-001 |
| P1 | WP-002b | Publish-name reservation + license/metadata template; verify availability on crates.io / PyPI / npm. | WP-001 |
| P2 | WP-003 | Workspace consolidation: migrate Rust crates into the chosen layout; merge `phenotype-shared → phenotype-contracts`; merge `PolicyStack → phenotype-policy-engine`; collapse `phenotype-error-core/errors/macros` and `ports-canonical/port-traits/async-traits → phenotype-contracts`. | WP-002a |
| P2 | WP-004 | Non-Rust crate handling: `phenotype-middleware-py` packaging clean-up; `phenotype-auth-ts` build/lint pass; decide fate of `phenotype-logging-zig` (publish vs deprecate per ADR). | WP-002a |
| P3 | WP-005 | API stabilization: enforce `Debug`/`Clone` on public types, kill stray `impl Trait` in pub APIs, write missing tests, write rustdoc with examples, set `0.1.0`, add per-crate `CHANGELOG.md`. | WP-003, WP-004 |
| P3 | WP-006 | Cross-crate integration tests + tarpaulin/llvm-cov report meeting coverage gates. | WP-005 |
| P4 | WP-007 | Publication: tag-driven publish to crates.io / PyPI / npm; verify install from clean env; CI publish workflow under `.github/workflows/`. | WP-006, WP-002b |
| P4 | WP-008 | Cross-crate dependency audit + dedup: `cargo audit`, `pip-audit`, `npm audit`; consolidate to `[workspace.dependencies]`; verify zero cycles; document MSRV CI check; dependency-update policy. | WP-007 |
| P4 | WP-009 | Downstream consumer update + smoke tests: bump callers (e.g. `AgilePlus`, `phenotype-infra`, `bifrost-extensions`, `Tokn` consumers) to published versions; smoke `cargo build` per consumer. | WP-008 |

### DAG (textual)

```
WP-000 ──► WP-001 ──► WP-002a ──► WP-003 ──┐
                   └► WP-002b              ├─► WP-005 ──► WP-006 ──► WP-007 ──► WP-008 ──► WP-009
                      WP-002a ──► WP-004 ──┘                              ▲
                      WP-002b ─────────────────────────────────────────── ┘
```

**Critical path:** WP-000 → WP-001 → WP-002a → WP-003 → WP-005 → WP-006 → WP-007 → WP-008 → WP-009 (9 hops).

**Parallelization opportunities:**
- P1: WP-002a and WP-002b run concurrently (independent inputs).
- P2: WP-003 (Rust) and WP-004 (Python/TS/Zig) run concurrently after WP-002a.
- P2 within WP-003: crate-migration subtasks (`T012`–`T017` in `tasks.md`) fan out across 3–4 parallel subagent batches grouped by crate cluster.

---

## Effort (agent-time)

> Agent-time only. No human checkpoints. Mapping per global "Timescales" rule.

| WP | Tool calls (rough) | Parallel subagent batches | Wall-clock target |
|----|--------------------|---------------------------|-------------------|
| WP-000 | 15–25 | 1 batch (3 agents: PR drain / worktree resolve / branch prune) | 8–15 min |
| WP-001 | 20–30 | 1 batch (4 agents grouped by language) | 10–15 min |
| WP-002a | 6–10 | — (single planner agent, no code) | 4–6 min |
| WP-002b | 4–8 | — | 3–5 min |
| WP-003 | 30–60 | 2 batches × 3 agents (crate clusters) | 15–25 min |
| WP-004 | 10–20 | 1 batch (3 agents: py / ts / zig) | 6–10 min |
| WP-005 | 25–45 | 2 batches × 3 agents (per-crate test/doc passes) | 15–20 min |
| WP-006 | 10–18 | 1 batch (2 agents: integration tests / coverage report) | 8–12 min |
| WP-007 | 12–20 | — (sequential per registry) | 8–12 min |
| WP-008 | 8–14 | 1 batch (3 agents: cargo / pip / npm audits) | 5–8 min |
| WP-009 | 15–25 | 1 batch (N agents = downstream consumer count) | 10–15 min |
| **Total** | **155–265** | up to 5 concurrent agents at peak | **90–145 min** wall-clock end-to-end |

---

## Cross-Project Reuse Opportunities

Per `~/.claude/CLAUDE.md` Phenotype Org Cross-Project Reuse Protocol:

| Candidate | Source crates | Target shared module | Impacted repos |
|-----------|---------------|----------------------|----------------|
| Unified config loader | `phenotype-config`, `phenotype-config-core` | `phenotype-config-core` (already extracted, see MEMORY 2026-03-29) | `AgilePlus`, `phenotype-infra`, `Tokn`, `civ` |
| Canonical error types | `phenotype-error-core`, `phenotype-errors`, `phenotype-error-macros` | Single `phenotype-error-core` | All Rust consumers |
| Port traits | `phenotype-ports-canonical`, `phenotype-port-traits`, `phenotype-async-traits` | `phenotype-contracts` | All Rust consumers |
| Policy engine | `PolicyStack`, `phenotype-policy-engine` | `phenotype-policy-engine` | `Tokn`, `Authvault`, `civ` |

WP-009 enforces forward-only migration: callers updated, duplicated local impls removed.

---

## Risks & Mitigations

| Risk | Phase | Mitigation |
|------|-------|------------|
| WP-000 drain fails (PR conflicts) | P0 | Process PRs in dependency order; rebase via subagent; do not skip — required for clean baseline. |
| Workspace split (3-way) thrashes downstream | P1 | ADR explicitly weighs single-workspace baseline vs 3-split; default = single workspace unless build-time evidence justifies split. |
| Coverage gate (≥80 %) infeasible for some crates | P3 | Crypto/auth crates ≥90 %; utility crates may land at 70 % with rationale recorded in CHANGELOG. |
| crates.io / npm name collisions | P4 | WP-002b reserves names early; collisions fall back to `phenotype-` prefix. |
| `phenotype-logging-zig` lacks ecosystem fit | P2 | WP-004 explicitly decides publish vs deprecate; deprecation falls under spec scope (consolidation, not behavior loss). |
| `cargo audit` blocks on transitive vuln | P4 | Document acceptances inline with tracking issue per global "Suppression/Ignore Rules". |

---

## Acceptance (rolls up to spec.md "Success Criteria")

- All 19 crates accounted for in workspace (migrated, merged, or formally deprecated per ADR).
- `cargo build --workspace` / `cargo clippy --workspace -- -D warnings` / `cargo fmt --check` green on `main`.
- Coverage gates met (overall ≥80 %, crypto/auth ≥90 %).
- Published versions live on crates.io / PyPI / npm and installable from clean environment.
- `cargo audit` / `pip-audit` / `npm audit` reports archived under `phenotype-infrakit/docs/audit/2026-Q2/`.
- Downstream consumer manifests updated; their CI green.
- AgilePlus statuses: WP-000 … WP-009 all `done`.

---

## Handoff Pointers

- Implementer entry point: [`tasks.md`](./tasks.md) (WP-by-WP subtask breakdown).
- Spec context: [`spec.md`](./spec.md) §"Audit Update — 2026-04-02" for live counts (10 PRs, 2 worktrees, ~20 stale branches, 1.8 GB → 0.5 GB after `cargo clean`).
- Monorepo gotchas: memory file `reference_monorepo_patterns.md` (`.archive/` embedded git, never `git add -A`).
- Governance: `~/.claude/CLAUDE.md` (Planner Agents · Phased WBS · Timescales) and `repos/CLAUDE.md` (testing FR traceability — every test references an FR).
