# Tasks: Consolidate `phenotype-policy-engine` to canonical `phenoShared`

Each WP is independently dispatchable. Hosts are parallel-mergeable except `AuthKit`, which gates on the canonical feature-flag PR.

## WP-CPE-01 — Discovery & drift matrix (incl. AuthKit inventory)

- **Scope (read):** all KooshaPari org repos containing `crates/phenotype-policy-engine/`.
- **Scope (write):** `kitty-specs/consolidate-policy-engine-crate/research.md`.
- **Acceptance criteria:**
  - Drift matrix lists LOC, last-commit, version, public-API delta, TOML schema, decision-combination strategy per copy.
  - Consumer-edge map enumerates `path =` refs and `use phenotype_policy_engine::*` sites.
  - `AuthKit` divergence inventory: every authz-specific symbol absent from canonical is enumerated with a proposed lift target.
  - Canonical baseline SHA recorded.
- **Depends on:** none.
- **Estimate:** 8–14 tool calls / 3–5 min.
**File Scope:**
- Read: [all KooshaPari org repos containing `crates/phenotype-policy-engine/`, none]
- Write: [`kitty-specs/consolidate-policy-engine-crate/research.md`]

## WP-CPE-02 — Per-host migration ticket + canonical feature-flag + TOML reconciliation plan

- **Scope (read):** WP-CPE-01 outputs.
- **Scope (write):** per-host migration-ticket subsections + canonical `feature = ["auth-extensions"]` design note + TOML schema-version reconciliation note in `research.md`.
- **Acceptance criteria:**
  - Per-host classification recorded.
  - Lift-vs-remove decisions for each `AuthKit`-divergent symbol.
  - TOML compatibility plan recorded (canonical-side schema-version handling vs. consumer-side migration).
  - Per-host dep strategy chosen.
- **Depends on:** WP-CPE-01.
- **Estimate:** 6–10 tool calls / 2–4 min.
**File Scope:**
- Read: [WP-CPE-01 outputs, WP-CPE-01]
- Write: [research.md]

## WP-CPE-03 — Land canonical `auth-extensions` feature flag on `phenoShared`

- **Scope (write):**
  - `phenoShared/crates/phenotype-policy-engine/Cargo.toml` gets a `[features]` table with `auth-extensions = []`.
  - The lifted symbols (per WP-CPE-02) land behind `#[cfg(feature = "auth-extensions")]`.
  - PR body cites the ADR + this WP.
- **Acceptance criteria:**
  - `phenoShared` workspace `cargo build --all-features` and `cargo test --all-features` pass.
  - `cargo build` (no features) still passes (default-off).
  - PR merged before WP-CPE-08 dispatches.
- **Depends on:** WP-CPE-02.
- **Estimate:** 6–10 tool calls / 3–4 min.
**File Scope:**
- Read: [all KooshaPari org repos containing `crates/phenotype-policy-engine/`, phenoShared/, WP-CPE-02]
- Write: [phenoShared/crates/phenotype-policy-engine/Cargo.toml, phenoShared]

## WP-CPE-04 — Land canonical TOML schema-version handling on `phenoShared`

- **Scope (write):** if WP-CPE-02's TOML reconciliation plan calls for canonical-side schema versioning, land it on canonical with backward-compat parse paths.
- **Acceptance criteria:**
  - Each consumer's existing TOML fixture parses cleanly under canonical.
  - Decision outputs for each fixture match the consumer's previous behavior.
- **Depends on:** WP-CPE-02.
- **Estimate:** 5–8 tool calls / 2–3 min.
**File Scope:**
- Read: [all KooshaPari org repos containing `crates/phenotype-policy-engine/`, phenoShared/, WP-CPE-02]
- Write: [phenoShared]

## WP-CPE-05 — Migrate host `pheno`

- **Scope (write):** DEPRECATED.md + banner; `pheno/Cargo.toml` workspace edits; consumer crate `Cargo.toml` edits.
- **Acceptance criteria:** `cargo build --workspace`, `cargo test --workspace` pass; cargo metadata single-resolution; policy-evaluation parity green; PR cites ADR.
- **Depends on:** WP-CPE-02, WP-CPE-04.
- **Parallel with:** WP-CPE-06, WP-CPE-07, WP-CPE-08.
- **Estimate:** 4–6 tool calls / 2–3 min.
**File Scope:**
- Read: [pheno/Cargo.toml, Cargo.toml, WP-CPE-02, WP-CPE-04]
- Write: [DEPRECATED.md + banner; `pheno/Cargo.toml` workspace edits; consumer crate `Cargo.toml` edits, pheno]

## WP-CPE-06 — Migrate host `PhenoProc` (root + nested)

- **Scope:** both copies retired in single migration commit.
- **Acceptance criteria:** identical to WP-CPE-05 plus single resolution across root+nested.
- **Depends on:** WP-CPE-02, WP-CPE-04.
- **Parallel with:** WP-CPE-05, WP-CPE-07, WP-CPE-08.
- **Estimate:** 5–8 tool calls / 3–4 min.
**File Scope:**
- Read: [all KooshaPari org repos containing `crates/phenotype-policy-engine/`, phenoShared/, WP-CPE-02, WP-CPE-04]
- Write: [kitty-specs/consolidate-policy-engine-crate/research.md]

## WP-CPE-07 — Migrate hosts `HexaKit/crates` and `PhenoKits/HexaKit`

- **Scope:** mirror of WP-CPE-05 against both `HexaKit/crates` and `PhenoKits/HexaKit`. Two PRs (one per host) — bundled into one WP for ticket economy.
- **Acceptance criteria:** identical to WP-CPE-05; both hosts independent PRs.
- **Depends on:** WP-CPE-02, WP-CPE-04.
- **Parallel with:** WP-CPE-05, WP-CPE-06, WP-CPE-08.
- **Estimate:** 6–10 tool calls / 3–5 min.
**File Scope:**
- Read: [all KooshaPari org repos containing `crates/phenotype-policy-engine/`, phenoShared/, WP-CPE-02, WP-CPE-04]
- Write: [HexaKit/crates, PhenoKits/HexaKit]

## WP-CPE-08 — Migrate host `AuthKit/rust` (special: features-enabled swap)

- **Scope (write):**
  - DEPRECATED.md + banner in `AuthKit/rust/crates/phenotype-policy-engine/`.
  - `AuthKit/rust/Cargo.toml` workspace edits with `features = ["auth-extensions"]` on the canonical dep.
  - Consumer crates' `Cargo.toml` edits.
- **Acceptance criteria:**
  - `cargo build --workspace` and `cargo test --workspace` pass.
  - cargo metadata single-resolution.
  - All previously authz-specific behaviors are reachable via the canonical feature flag.
  - Policy-evaluation parity tests pass on every existing AuthKit policy fixture.
  - PR body cites ADR + WP-CPE-03.
- **Depends on:** WP-CPE-03, WP-CPE-04.
- **Estimate:** 5–8 tool calls / 3–4 min.
**File Scope:**
- Read: [Cargo.toml, WP-CPE-03, WP-CPE-04]
- Write: [DEPRECATED.md + banner in `AuthKit/rust/crates/phenotype-policy-engine/`, AuthKit/rust/Cargo.toml, AuthKit/rust]

## WP-CPE-09 — AgilePlus WP closeout

- **Scope (write):**
  - Audit-doc supersede pointer (coordinate via WP-CES-08; sibling cross-link only).
  - AgilePlus WP status `done`.
- **Acceptance criteria:**
  - AgilePlus dashboard `done`.
  - 24-hour post-merge probe shows no host re-introduction.
- **Depends on:** WP-CPE-05 ∧ WP-CPE-06 ∧ WP-CPE-07 ∧ WP-CPE-08.
- **Estimate:** 3–5 tool calls / 1–2 min.
**File Scope:**
- Read: [all KooshaPari org repos containing `crates/phenotype-policy-engine/`, phenoShared/, WP-CPE-05 ∧ WP-CPE-06 ∧ WP-CPE-07 ∧ WP-CPE-08]
- Write: [kitty-specs/consolidate-policy-engine-crate/research.md]

## Aggregate

- **WP count:** 9.
- **Critical path:** WP-CPE-01 → WP-CPE-02 → (WP-CPE-03 ∧ WP-CPE-04) → WP-CPE-08 → WP-CPE-09 (with WP-CPE-05..07 in parallel after WP-CPE-04).
- **Wall clock with full parallelism:** ~19–29 min.
