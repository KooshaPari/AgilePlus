# Tasks: Consolidate `phenotype-state-machine` to canonical `phenoShared`

Each WP is independently dispatchable. Hosts are parallel-mergeable except `ResilienceKit`, which gates on the canonical feature-flag PR.

## WP-CSM-01 — Discovery & drift matrix (incl. ResilienceKit inventory)

- **Scope (read):** all KooshaPari org repos containing `crates/phenotype-state-machine/`.
- **Scope (write):** `kitty-specs/consolidate-state-machine-crate/research.md`.
- **Acceptance criteria:**
  - Drift matrix lists LOC, last-commit, version, public-API delta, transition-guard contract per copy.
  - Consumer-edge map enumerates `path =` refs and `use phenotype_state_machine::*` sites.
  - `ResilienceKit` divergence inventory: every symbol absent from canonical is enumerated with a proposed lift target (canonical feature flag vs. removal).
  - Canonical baseline SHA recorded.
- **Depends on:** none.
- **Estimate:** 8–12 tool calls / 3–4 min.

## WP-CSM-02 — Per-host migration ticket authoring + canonical feature-flag plan

- **Scope (read):** WP-CSM-01 outputs.
- **Scope (write):** per-host migration-ticket subsections in `research.md` + canonical `feature = ["resilience-hooks"]` design note.
- **Acceptance criteria:**
  - Per-host classification recorded.
  - Lift-vs-remove decisions for each `ResilienceKit`-divergent symbol.
  - Per-host dep strategy chosen.
- **Depends on:** WP-CSM-01.
- **Estimate:** 5–8 tool calls / 2–3 min.

## WP-CSM-03 — Land canonical feature flag on `phenoShared`

- **Scope (write):**
  - `phenoShared/crates/phenotype-state-machine/Cargo.toml` gets a `[features]` table with `resilience-hooks = []`.
  - The lifted symbols (per WP-CSM-02 design) land behind `#[cfg(feature = "resilience-hooks")]`.
  - PR body cites the ADR + this WP.
- **Acceptance criteria:**
  - `phenoShared` workspace `cargo build --all-features` and `cargo test --all-features` pass.
  - `cargo build` (no features) still passes (default-off).
  - PR merged before WP-CSM-08 dispatches.
- **Depends on:** WP-CSM-02.
- **Estimate:** 6–10 tool calls / 3–4 min.

## WP-CSM-04 — Migrate host `pheno`

- **Scope (write):** DEPRECATED.md + banner; `pheno/Cargo.toml` workspace edits; consumer crate `Cargo.toml` edits.
- **Acceptance criteria:** `cargo build --workspace`, `cargo test --workspace` pass; cargo metadata single-resolution; PR cites ADR.
- **Depends on:** WP-CSM-02.
- **Parallel with:** WP-CSM-05, WP-CSM-06, WP-CSM-07.
- **Estimate:** 4–6 tool calls / 2–3 min.

## WP-CSM-05 — Migrate host `PhenoProc` (root + nested)

- **Scope:** both copies retired in single migration commit.
- **Acceptance criteria:** identical to WP-CSM-04 plus single resolution across root+nested.
- **Depends on:** WP-CSM-02.
- **Parallel with:** WP-CSM-04, WP-CSM-06, WP-CSM-07.
- **Estimate:** 5–8 tool calls / 3–4 min.

## WP-CSM-06 — Migrate host `HexaKit/crates`

- **Scope:** mirror of WP-CSM-04 against `HexaKit/crates`.
- **Acceptance criteria:** identical to WP-CSM-04.
- **Depends on:** WP-CSM-02.
- **Parallel with:** WP-CSM-04, WP-CSM-05, WP-CSM-07.
- **Estimate:** 4–6 tool calls / 2–3 min.

## WP-CSM-07 — Migrate host `PhenoKits/HexaKit`

- **Scope:** mirror of WP-CSM-04 against `PhenoKits/HexaKit`.
- **Acceptance criteria:** identical to WP-CSM-04.
- **Depends on:** WP-CSM-02.
- **Parallel with:** WP-CSM-04, WP-CSM-05, WP-CSM-06.
- **Estimate:** 4–6 tool calls / 2–3 min.

## WP-CSM-08 — Migrate host `ResilienceKit/rust` (special: features-enabled swap)

- **Scope (write):**
  - DEPRECATED.md + banner in `ResilienceKit/rust/crates/phenotype-state-machine/`.
  - `ResilienceKit/rust/Cargo.toml` workspace edits with `features = ["resilience-hooks"]` on the canonical dep.
  - Consumer crates' `Cargo.toml` edits.
- **Acceptance criteria:**
  - `cargo build --workspace` and `cargo test --workspace` pass.
  - cargo metadata single-resolution.
  - All previously domain-specific behaviors are reachable via the canonical feature flag.
  - PR body cites ADR + WP-CSM-03.
- **Depends on:** WP-CSM-03 (canonical feature flag landed).
- **Estimate:** 5–8 tool calls / 3–4 min.

## WP-CSM-09 — AgilePlus WP closeout

- **Scope (write):**
  - Audit-doc supersede pointer (coordinate via WP-CES-08; sibling cross-link only).
  - AgilePlus WP status `done`.
- **Acceptance criteria:**
  - AgilePlus dashboard `done`.
  - 24-hour post-merge probe shows no host re-introduction.
- **Depends on:** WP-CSM-04 ∧ WP-CSM-05 ∧ WP-CSM-06 ∧ WP-CSM-07 ∧ WP-CSM-08.
- **Estimate:** 3–5 tool calls / 1–2 min.

## Aggregate

- **WP count:** 9.
- **Critical path:** WP-CSM-01 → WP-CSM-02 → WP-CSM-03 → WP-CSM-08 → WP-CSM-09 (with WP-CSM-04..07 in parallel after WP-CSM-02).
- **Wall clock with full parallelism:** ~18–25 min.
