# Tasks: Consolidate `phenotype-event-sourcing` to canonical `phenoShared`

Each Work Package (WP) is independently dispatchable to a subagent. Hosts are parallel-mergeable.

## WP-CES-01 — Discovery & drift matrix

- **Scope (read):** all KooshaPari org repos containing `crates/phenotype-event-sourcing/`.
- **Scope (write):** `kitty-specs/consolidate-event-sourcing-crate/research.md` (drift matrix table).
- **Acceptance criteria:**
  - Drift matrix lists LOC, last-commit, declared version, and public-API delta vs. canonical for all 6 copies.
  - Consumer-edge map enumerates every internal Cargo.toml currently importing the crate via `path =`.
  - Canonical API baseline locked to `phenoShared@HEAD` at task start (commit SHA recorded).
- **Depends on:** none.
- **Estimate:** 6–10 tool calls / 2–3 min.

## WP-CES-02 — Per-host migration ticket authoring

- **Scope (read):** WP-CES-01 outputs.
- **Scope (write):** one migration-ticket subsection in `research.md` per host workspace (`pheno`, `PhenoProc`, `DataKit`, `PhenoKits/HexaKit`, `hwLedger`).
- **Acceptance criteria:**
  - Per-host classification (`subset` / `superset` / `divergent`) recorded.
  - Per-host lift-vs-shim decisions recorded for any superset symbols.
  - Per-host dep strategy chosen (`path` to sibling clone, `git = "..."` pin, or shared registry).
- **Depends on:** WP-CES-01.
- **Estimate:** 4–6 tool calls / 1–2 min.

## WP-CES-03 — Migrate host `pheno`

- **Scope (read):** WP-CES-02 ticket for `pheno`.
- **Scope (write):**
  - Replace `pheno/crates/phenotype-event-sourcing/src/lib.rs` with `// DEPRECATED — see DEPRECATED.md` banner (transition).
  - Add `pheno/crates/phenotype-event-sourcing/DEPRECATED.md` per ADR template.
  - Edit `pheno/Cargo.toml` `members` and `[workspace.dependencies]`.
  - Edit consumer crates' `Cargo.toml` to use canonical dep ref.
- **Acceptance criteria:**
  - `cargo build --workspace` and `cargo test --workspace` pass in `pheno`.
  - `cargo metadata` resolves exactly one `phenotype-event-sourcing` entry.
  - PR body cites the canonical-home ADR.
- **Depends on:** WP-CES-02.
- **Parallel with:** WP-CES-04, WP-CES-05, WP-CES-06, WP-CES-07.
- **Estimate:** 4–6 tool calls / 2–3 min.

## WP-CES-04 — Migrate host `PhenoProc` (both root and nested copies)

- **Scope (read):** WP-CES-02 ticket for `PhenoProc`.
- **Scope (write):**
  - Both `PhenoProc/crates/phenotype-event-sourcing/` and `PhenoProc/crates/phenotype-shared/crates/phenotype-event-sourcing/` get DEPRECATED.md + banner.
  - `PhenoProc` workspace `Cargo.toml` updated.
  - Nested `phenotype-shared/Cargo.toml` either retired or repointed.
- **Acceptance criteria:** identical to WP-CES-03 plus: only one resolution remains across both root and nested workspaces.
- **Depends on:** WP-CES-02.
- **Parallel with:** WP-CES-03, WP-CES-05, WP-CES-06, WP-CES-07.
- **Estimate:** 5–8 tool calls / 3–4 min (highest complexity due to nesting).

## WP-CES-05 — Migrate host `DataKit/rust`

- **Scope:** mirror of WP-CES-03 against `DataKit/rust`.
- **Acceptance criteria:** identical to WP-CES-03.
- **Depends on:** WP-CES-02.
- **Parallel with:** WP-CES-03, WP-CES-04, WP-CES-06, WP-CES-07.
- **Estimate:** 4–6 tool calls / 2–3 min.

## WP-CES-06 — Migrate host `PhenoKits/HexaKit`

- **Scope:** mirror of WP-CES-03 against `PhenoKits/HexaKit`.
- **Notes:** 26-LOC variant — likely `subset` of canonical; expect minimal shim work.
- **Acceptance criteria:** identical to WP-CES-03.
- **Depends on:** WP-CES-02.
- **Parallel with:** WP-CES-03, WP-CES-04, WP-CES-05, WP-CES-07.
- **Estimate:** 4–6 tool calls / 2–3 min.

## WP-CES-07 — Migrate host `hwLedger/vendor`

- **Scope:** vendored snapshot replacement.
- **Notes:** if vendored for compliance reasons, swap to a `git = "..."` pin against a `phenoShared` commit SHA rather than path; record rationale in PR body.
- **Acceptance criteria:** vendor directory removed (or replaced with pin manifest); host workspace builds.
- **Depends on:** WP-CES-02.
- **Parallel with:** WP-CES-03, WP-CES-04, WP-CES-05, WP-CES-06.
- **Estimate:** 4–6 tool calls / 2–3 min.

## WP-CES-08 — Audit-doc supersede pointer + AgilePlus WP closeout

- **Scope (write):**
  - One-line trailing pointer added to `repos/docs/governance/cross-project-reuse-audit-2026-04-25.md`.
  - AgilePlus WP status updated via `agileplus status consolidate-event-sourcing-crate --wp <id> --state done` after all host PRs merge.
- **Acceptance criteria:**
  - Audit doc references the ADR.
  - AgilePlus dashboard reflects `done`.
  - Post-merge probe subagent confirms no host re-introduces a local copy within 24 hours.
- **Depends on:** WP-CES-03 ∧ WP-CES-04 ∧ WP-CES-05 ∧ WP-CES-06 ∧ WP-CES-07.
- **Estimate:** 4–6 tool calls / 2–3 min.

## Aggregate

- **WP count:** 8.
- **Critical path:** WP-CES-01 → WP-CES-02 → (5 parallel host migrations) → WP-CES-08.
- **Wall clock with full parallelism:** ~13–20 min.
