# Work Packages: phenotype-infrakit Stabilization — Consolidate 19 Infrastructure Crates

**Inputs:** [`spec.md`](./spec.md), [`plan.md`](./plan.md).
**Prerequisites:** read access to `repos/phenotype-infrakit/`, `repos/phenotype-shared/`, `repos/Tokn/`, `repos/Authvault/`, `repos/PolicyStack/`, plus the 14 other crate roots listed in `spec.md`. Rust toolchain (stable + MSRV from WP-002a). Python 3.x toolchain. Node + npm. Zig toolchain (only if WP-004 chooses publish).
**Scope:** `repos/phenotype-infrakit/` workspace (Rust) + the four non-Rust crates. No new crates.

> **Planner-agent rule:** this file is a spec, not an implementation. Tasks reference file paths and acceptance criteria. No code, no concrete diffs.

---

## WP-000: Preflight — drain backlog, free disk

- **State:** planned
- **Sequence:** 0
- **Phase:** P0
- **File scope:** `repos/phenotype-infrakit/` (entire repo); GitHub PRs `#544–#563`; worktrees `cache-adapter-impl`, `phenotype-crypto-complete-v2`; ~20 stale branches.
- **Acceptance:**
  - Open PR count for `phenotype-infrakit` is 0 (or each remaining PR has a documented reason for staying open, linked to a tracking issue).
  - Both noted worktrees rebased onto `main` and either merged or closed with rationale.
  - Stale branches (no associated PR, no commits in last 30 days) deleted on `origin`.
  - `df -h /System/Volumes/Data` shows ≥10 Gi free post-`cargo clean` (per Phenotype disk-budget policy).
- **Dependencies:** none (entry WP).
- **Subtasks:**
  - [ ] T000-1 Re-verify counts vs `spec.md` audit memo; record live numbers in `docs/audit/2026-Q2/preflight.md`.
  - [ ] T000-2 Drain `#544`, `#553`, `#554`, `#557`, `#558`, `#559`, `#560`, `#561`, `#562`, `#563` in topo order (workspace stabilization PRs first).
  - [ ] T000-3 Resolve `cache-adapter-impl` (detached HEAD): rebase or close.
  - [ ] T000-4 Merge `phenotype-crypto-complete-v2` if ready, else close with rationale.
  - [ ] T000-5 Audit ~20 stale branches; delete or convert each to a tracked PR.
  - [ ] T000-6 `cargo clean` and confirm ≥10 Gi free.
- **Risks:** PR rebase conflicts → process PRs in dependency order; `cargo clean` during active builds → check `ps aux | grep cargo` first (per memory `feedback_pruner_atime_limitation`).

---

## WP-001: Audit all 19 crates

- **State:** planned
- **Sequence:** 1
- **Phase:** P0
- **File scope:** all 19 crate roots listed in `spec.md` "Repositories Affected".
- **Acceptance:**
  - Per-crate audit row containing: API surface count (pub items), test coverage %, doc quality grade, dep count, maturity (`production-ready` / `needs-stabilization` / `stub-or-deprecate`).
  - Inter-crate dependency graph rendered to `docs/audit/2026-Q2/depgraph.md` (Mermaid).
  - Duplicated-functionality matrix lists overlap clusters (config / errors / ports / policy / etc.).
  - Language-fit memo for `phenotype-logging-zig` and `phenotype-middleware-py`.
- **Dependencies:** WP-000.
- **Subtasks:**
  - [ ] T001-1 Verify each of 19 crates clones and builds (or document failure).
  - [ ] T001-2 Measure pub API surface (`cargo public-api` for Rust; AST tooling for Python/TS).
  - [ ] T001-3 Coverage measurement (cargo tarpaulin / pytest-cov / nyc).
  - [ ] T001-4 Doc-quality grading via README presence + rustdoc coverage report.
  - [ ] T001-5 Build internal + external dependency graph.
  - [ ] T001-6 Identify overlap clusters; produce matrix.
  - [ ] T001-7 Categorize each crate; write recommendation per crate.
- **Risks:** crate that does not build → record but don't block audit. Coverage tool variance → cross-check tarpaulin vs llvm-cov.

---

## WP-002a: Consolidation design ADR

- **State:** planned
- **Sequence:** 2a
- **Phase:** P1
- **File scope:** `phenotype-infrakit/docs/adr/0NN-consolidation-design.md`.
- **Acceptance:**
  - ADR records decision: single workspace vs 3-split (`core` / `runtime` / `tools`).
  - Merge map enumerates every consolidation: `phenotype-shared → phenotype-contracts`; `PolicyStack → phenotype-policy-engine`; `phenotype-error-{core,errors,macros} → phenotype-error-core`; `phenotype-{ports-canonical,port-traits,async-traits} → phenotype-contracts`.
  - Error convention pinned: `thiserror` + `#[from]`; pub types `Debug + Clone` where practical.
  - MSRV chosen and recorded.
  - `[workspace.dependencies]` policy: workspace-level pin for any dep used by ≥2 crates.
- **Dependencies:** WP-001.
- **Subtasks:**
  - [ ] T002a-1 Compare workspace-shape options against WP-001 dep graph (build-time, churn, downstream blast radius).
  - [ ] T002a-2 Enumerate merge map cell-by-cell with deprecation aliases where shapes differ.
  - [ ] T002a-3 Decide MSRV (rustc version + rationale).
  - [ ] T002a-4 Write ADR; ship as PR; merge.
- **Risks:** premature 3-split → default to single workspace unless WP-001 evidence forces split.

---

## WP-002b: Publish-name reservation + metadata template

- **State:** planned
- **Sequence:** 2b
- **Phase:** P1
- **File scope:** `phenotype-infrakit/docs/audit/2026-Q2/publish-names.md`; per-crate `Cargo.toml` `[package]` metadata.
- **Acceptance:**
  - Availability checked on crates.io / PyPI / npm for every target name.
  - Reservation strategy decided (claim now vs claim at publish time).
  - License (`Apache-2.0` org default), `repository`, `documentation`, `keywords`, `categories` template applied per crate.
- **Dependencies:** WP-001.
- **Subtasks:**
  - [ ] T002b-1 Query each registry for name availability.
  - [ ] T002b-2 Pick fallback prefix scheme on collision (e.g. `phenotype-<name>`).
  - [ ] T002b-3 Apply metadata template stub to each crate manifest.
- **Risks:** name squatted → fallback prefix; document in ADR.

---

## WP-003: Workspace consolidation (Rust)

- **State:** planned
- **Sequence:** 3
- **Phase:** P2
- **File scope:** root `Cargo.toml`; every Rust crate's `Cargo.toml` and `src/` tree under `phenotype-infrakit/`.
- **Acceptance:**
  - Layout matches WP-002a ADR.
  - All Rust crates from spec are workspace members or formally deprecated per ADR.
  - Duplicate crates merged (no two crates expose the same canonical type).
  - `cargo build --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check` all green.
  - No new shell scripts introduced (per Phenotype scripting hierarchy).
- **Dependencies:** WP-002a.
- **Subtasks:**
  - [ ] T003-1 Apply layout: move/rename crates per ADR.
  - [ ] T003-2 Migrate `phenotype-config` (with tests + docs intact).
  - [ ] T003-3 Migrate `phenotype-gauge`, `phenotype-nexus`, `phenotype-forge`.
  - [ ] T003-4 Migrate `phenotype-cipher`, `Authvault`, `Tokn`, `Zerokit`.
  - [ ] T003-5 Merge `phenotype-shared → phenotype-contracts`; reconcile types; keep deprecation re-exports until WP-009.
  - [ ] T003-6 Merge `PolicyStack → phenotype-policy-engine`.
  - [ ] T003-7 Collapse `phenotype-error-{core,errors,macros}` and `phenotype-{ports-canonical,port-traits,async-traits}` per ADR.
  - [ ] T003-8 Migrate `Quillr`, `Httpora`, `Apisync`, `phenotype-cli-core`.
  - [ ] T003-9 Apply error-handling convention crate-by-crate.
  - [ ] T003-10 Run + fix `cargo build --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`.
- **Risks:** type reconciliation breaking changes → use `#[deprecated]` re-exports until WP-009 lands downstream bumps.

---

## WP-004: Non-Rust crate handling (Python / TypeScript / Zig)

- **State:** planned
- **Sequence:** 4
- **Phase:** P2
- **File scope:** `phenotype-middleware-py/` (`pyproject.toml`, `src/`); `phenotype-auth-ts/` (`package.json`, `src/`); `phenotype-logging-zig/`.
- **Acceptance:**
  - Python: `ruff check` clean, `pytest` green, `pyproject.toml` ready for PyPI publish.
  - TypeScript: `tsc --noEmit`, lint, test green; `package.json` ready for npm publish.
  - Zig: WP-002a ADR decision honored — either Zig package manifest ready for publish or crate marked deprecated with migration path.
- **Dependencies:** WP-002a.
- **Subtasks:**
  - [ ] T004-1 Python packaging clean-up + lint/test pass.
  - [ ] T004-2 TypeScript build/lint/test pass + `package.json` metadata.
  - [ ] T004-3 Apply WP-002a Zig decision; document outcome.
- **Risks:** Zig ecosystem fit poor → ADR allows deprecation; consolidation, not removal of behavior, since callers (if any) get a migration note.

---

## WP-005: API stabilization, tests, docs, semver

- **State:** planned
- **Sequence:** 5
- **Phase:** P3
- **File scope:** every crate's public API; `tests/`; `CHANGELOG.md` per crate.
- **Acceptance:**
  - All public types implement `Debug` + `Clone` (or rationale recorded).
  - No stray `impl Trait` in pub APIs unless documented.
  - Per-crate CHANGELOG seeded at `0.1.0`.
  - Rustdoc with at least one example per public item.
  - `cargo doc --workspace --no-deps` clean.
  - Every test references an FR per `repos/CLAUDE.md` "Testing & Specification Traceability".
- **Dependencies:** WP-003, WP-004.
- **Subtasks:**
  - [ ] T005-1 Public-API audit (`cargo public-api`) → fix gaps.
  - [ ] T005-2 Write missing tests for `phenotype-config`.
  - [ ] T005-3 Write missing tests for `phenotype-cipher`, `Authvault`, `Tokn` (target ≥90 % coverage).
  - [ ] T005-4 Write missing tests for `phenotype-gauge`, `phenotype-nexus`, `phenotype-forge`.
  - [ ] T005-5 Write missing tests for `Quillr`, `Httpora`, `Apisync`, `phenotype-cli-core`.
  - [ ] T005-6 Write rustdoc with examples for every pub item.
  - [ ] T005-7 Seed `CHANGELOG.md` at `0.1.0` for each crate; tag FR references.
- **Risks:** crypto crate coverage gaps → prioritize first; record any unreachable branch with `#[cfg(not(tarpaulin))]` only with tracking issue.

---

## WP-006: Cross-crate integration tests + coverage report

- **State:** planned
- **Sequence:** 6
- **Phase:** P3
- **File scope:** `phenotype-infrakit/tests/` integration suites; `docs/audit/2026-Q2/coverage.md`.
- **Acceptance:**
  - Cross-crate integration tests cover the duplicate-functionality clusters identified in WP-001 (config + error + ports + policy interplay).
  - Tarpaulin/llvm-cov report shows overall ≥80 %, crypto/auth ≥90 %.
  - Report archived under `docs/audit/2026-Q2/coverage.md`.
- **Dependencies:** WP-005.
- **Subtasks:**
  - [ ] T006-1 Author integration suites per cluster.
  - [ ] T006-2 Run coverage; archive report; gate met.
- **Risks:** flaky tests → quarantine with tracking issue, never `#[ignore]` silently.

---

## WP-007: Publish to crates.io / PyPI / npm

- **State:** planned
- **Sequence:** 7
- **Phase:** P4
- **File scope:** `.github/workflows/publish-*.yml`; per-registry tokens in CI secrets; verification scripts (Rust binaries per scripting policy).
- **Acceptance:**
  - Tag-driven publish workflow under `.github/workflows/`; triggers on `v*` tags.
  - Every Rust target crate published to crates.io.
  - `phenotype-middleware-py` published to PyPI.
  - `phenotype-auth-ts` published to npm.
  - Each published version installable from a clean environment.
- **Dependencies:** WP-006, WP-002b.
- **Subtasks:**
  - [ ] T007-1 Configure registry tokens (crates.io / PyPI / npm) in GitHub Secrets.
  - [ ] T007-2 Author publish workflow(s).
  - [ ] T007-3 Publish workspace crates in topological order (no inter-crate cycle by construction).
  - [ ] T007-4 Publish Python + TypeScript packages.
  - [ ] T007-5 Smoke install in clean container per package.
- **Risks:** publish ordering on inter-dependent crates → derive order from `cargo metadata`. Token leakage → scoped tokens, never echo. CI billing — see global GitHub Actions Billing Policy; rely on Linux runners only.

---

## WP-008: Cross-crate dep audit + dedup

- **State:** planned
- **Sequence:** 8
- **Phase:** P4
- **File scope:** root `Cargo.toml` `[workspace.dependencies]`; per-crate manifests; `docs/audit/2026-Q2/security.md`; CI MSRV check.
- **Acceptance:**
  - Zero duplicate deps (anything used ≥2× sits in `[workspace.dependencies]`).
  - `cargo audit`, `pip-audit`, `npm audit` reports archived; vulnerabilities either fixed or accepted with tracking link.
  - Zero dependency cycles.
  - MSRV CI job runs on every PR.
  - Dependency-update policy doc landed.
- **Dependencies:** WP-007.
- **Subtasks:**
  - [ ] T008-1 Audit dep tree; consolidate duplicates.
  - [ ] T008-2 `cargo audit` / `pip-audit` / `npm audit`; archive reports.
  - [ ] T008-3 Verify acyclic graph (`cargo metadata` post-process).
  - [ ] T008-4 Add MSRV CI job.
  - [ ] T008-5 Author dependency-update policy doc.
- **Risks:** transitive vuln without upstream fix → record acceptance per global "Suppression/Ignore Rules" (rule + concrete reason + tracking link).

---

## WP-009: Downstream consumer update + smoke

- **State:** planned
- **Sequence:** 9
- **Phase:** P4
- **File scope:** consumer manifests in `repos/AgilePlus`, `repos/phenotype-infra`, `repos/bifrost-extensions`, `repos/Tokn` consumers, and any other crate identified in WP-001 dep graph.
- **Acceptance:**
  - Each downstream consumer bumped to published versions; deprecation re-exports from WP-003 removed.
  - Smoke `cargo build` (or equivalent) green per consumer.
  - Forward-only migration recorded in each consumer's CHANGELOG.
- **Dependencies:** WP-008.
- **Subtasks:**
  - [ ] T009-1 Enumerate consumers from WP-001 graph.
  - [ ] T009-2 Bump each manifest; run `cargo build`/`pytest`/`pnpm build` smoke.
  - [ ] T009-3 Drop deprecation re-exports left in WP-003.
  - [ ] T009-4 Record forward-only migration in consumer CHANGELOGs.
- **Risks:** consumer build break → fix in same PR; never partial-bump (forward-only protocol).

---

## Dependency & Execution Summary

```
WP-000 ──► WP-001 ──► WP-002a ──► WP-003 ──┐
                   └► WP-002b              ├─► WP-005 ──► WP-006 ──► WP-007 ──► WP-008 ──► WP-009
                      WP-002a ──► WP-004 ──┘                              ▲
                      WP-002b ─────────────────────────────────────────── ┘
```

**Parallelization windows:**
- WP-002a / WP-002b run together after WP-001.
- WP-003 / WP-004 run together after WP-002a.
- Within WP-003, T003-2…T003-8 fan out across 3 subagent batches grouped by crate cluster.
- Within WP-005, T005-2…T005-6 fan out per crate.

**MVP slice:** WP-000 + WP-001 + WP-002a + WP-003 yields a clean, building, deduplicated workspace; that is a defensible interim state if downstream pressure requires pausing before WP-005+.

**Forbidden:** human checkpoints, "schedule a meeting", "assign owner", "review with team", week-based estimates. All effort lives in `plan.md` "Effort (agent-time)" table.
