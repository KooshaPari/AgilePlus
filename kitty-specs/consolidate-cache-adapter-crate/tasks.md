# Tasks: Consolidate `phenotype-cache-adapter` to canonical `phenoShared`

Each WP is independently dispatchable. Hosts are parallel-mergeable.

## WP-CCA-01 — Discovery & drift matrix

- **Scope (read):** all KooshaPari org repos containing `crates/phenotype-cache-adapter/`.
- **Scope (write):** `kitty-specs/consolidate-cache-adapter-crate/research.md`.
- **Acceptance criteria:**
  - Drift matrix lists LOC, last-commit, version, public-API delta, and cache-semantics constants (TTL default, capacity default, L1/L2 split) per copy.
  - Consumer-edge map enumerates internal Cargo.toml `path =` refs and `use phenotype_cache_adapter::*` import sites.
  - Canonical baseline = `phenoShared@HEAD` SHA recorded.
- **Depends on:** none.
- **Estimate:** 6–10 tool calls / 2–3 min.

## WP-CCA-02 — Per-host migration ticket authoring

- **Scope (read):** WP-CCA-01 outputs.
- **Scope (write):** per-host migration-ticket subsections in `research.md`.
- **Acceptance criteria:**
  - Per-host classification recorded.
  - Cache-semantics divergence either (a) lifted into canonical as a feature flag, or (b) accepted as canonical truth with host-side test updates — never a host fork.
  - Per-host dep strategy chosen.
- **Depends on:** WP-CCA-01.
- **Estimate:** 4–6 tool calls / 1–2 min.

## WP-CCA-03 — Migrate host `pheno`

- **Scope (write):**
  - DEPRECATED.md + banner in `pheno/crates/phenotype-cache-adapter/`.
  - `pheno/Cargo.toml` workspace edits.
  - Consumer crates' `Cargo.toml` edits.
- **Acceptance criteria:**
  - `cargo build --workspace` and `cargo test --workspace` pass.
  - `cargo metadata` resolves exactly one `phenotype-cache-adapter`.
  - Cache-semantics parity test results captured in PR body.
  - PR body cites the ADR.
- **Depends on:** WP-CCA-02.
- **Parallel with:** WP-CCA-04, WP-CCA-05, WP-CCA-06.
- **Estimate:** 4–6 tool calls / 2–3 min.

## WP-CCA-04 — Migrate host `PhenoProc` (both root and nested copies)

- **Scope (write):**
  - DEPRECATED.md + banner in both `PhenoProc/crates/phenotype-cache-adapter/` and `PhenoProc/crates/phenotype-shared/crates/phenotype-cache-adapter/`.
  - `PhenoProc` workspace `Cargo.toml` updates.
  - Nested `phenotype-shared/Cargo.toml` retired or repointed.
- **Acceptance criteria:** identical to WP-CCA-03 plus single resolution across root+nested.
- **Depends on:** WP-CCA-02.
- **Parallel with:** WP-CCA-03, WP-CCA-05, WP-CCA-06.
- **Estimate:** 5–8 tool calls / 3–4 min.

## WP-CCA-05 — Migrate host `DataKit/rust`

- **Scope:** mirror of WP-CCA-03 against `DataKit/rust`.
- **Acceptance criteria:** identical to WP-CCA-03.
- **Depends on:** WP-CCA-02.
- **Parallel with:** WP-CCA-03, WP-CCA-04, WP-CCA-06.
- **Estimate:** 4–6 tool calls / 2–3 min.

## WP-CCA-06 — Migrate host `PhenoKits/HexaKit`

- **Scope:** mirror of WP-CCA-03 against `PhenoKits/HexaKit`.
- **Acceptance criteria:** identical to WP-CCA-03.
- **Depends on:** WP-CCA-02.
- **Parallel with:** WP-CCA-03, WP-CCA-04, WP-CCA-05.
- **Estimate:** 4–6 tool calls / 2–3 min.

## WP-CCA-07 — Cache-semantics parity verification

- **Scope (write):** parity test suite landed on canonical `phenoShared` covering TTL expiry ordering, eviction policy, capacity bound — running each host's existing behavioral fixtures against the canonical surface.
- **Acceptance criteria:**
  - Parity matrix in WP-CCA-01's `research.md` updated with green/red per host.
  - Any red cell either resolved by lifting host behavior into canonical (preferred) or accepted with explicit consumer-side update — never a fork.
- **Depends on:** WP-CCA-01.
- **Parallel with:** WP-CCA-03..06 in the design phase; gates merge of those WPs.
- **Estimate:** 6–10 tool calls / 3–4 min.

## WP-CCA-08 — AgilePlus WP closeout + audit-doc pointer

- **Scope (write):**
  - One-line trailing pointer added to `cross-project-reuse-audit-2026-04-25.md` (only the first sibling WP to merge needs to do this — coordinate via WP-CES-08).
  - AgilePlus WP status updated `done`.
- **Acceptance criteria:**
  - AgilePlus dashboard reflects `done`.
  - 24-hour post-merge probe shows no host re-introduction.
- **Depends on:** WP-CCA-03 ∧ WP-CCA-04 ∧ WP-CCA-05 ∧ WP-CCA-06 ∧ WP-CCA-07.
- **Estimate:** 3–5 tool calls / 1–2 min.

## Aggregate

- **WP count:** 8.
- **Critical path:** WP-CCA-01 → WP-CCA-02 → (4 parallel host migrations + WP-CCA-07 parity) → WP-CCA-08.
- **Wall clock with full parallelism:** ~13–20 min.
