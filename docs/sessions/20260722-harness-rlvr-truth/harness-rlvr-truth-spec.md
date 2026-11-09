# Specification: Harness RLVR Truth
**Slug**: harness-rlvr-truth | **Date**: 2026-07-22 | **State**: specified

## Problem Statement
Ablation and comparison surfaces must report honest RLVR (reward / verifier) truth —
finite composite and layer scores from the harness — rather than placeholder or
adversarial-stub patterns that look like perfect intent with zero judge/hallucination
signal. Prior work landed partial plumbing:

- pheno-harness #33: emit `rlvr_*` on ablation cells
- phenotype-omlx #46: cockpit Cell passthrough for `rlvr_*`
- PR-C (in flight): `stock_vs_ours` quality stubs rewrite

This feature closes the truth gap end-to-end: dry ablation cells carry finite
`rlvr_composite` / `rlvr_l0..l3`, Go Cell round-trips preserve those fields,
`stock_vs_ours` refuses the adversarial stub signature, and cockpit
`resolveRlvr` reports `source: harness` when harness fields are present.

## Target Users
Harness / eval engineers running ablation and stock-vs-ours comparisons;
cockpit operators reading RLVR panels; agents validating cell contracts.

## Functional Requirements
- **FR-1**: Dry ablation runs SHALL emit cells with finite numeric
  `rlvr_composite` and `rlvr_l0`, `rlvr_l1`, `rlvr_l2`, `rlvr_l3` (no NaN/Inf/null
  for completed dry cells).
- **FR-2**: Go Cell serialize/deserialize round-trip SHALL preserve all `rlvr_*`
  fields present on the input cell (no silent drop or type coercion loss).
- **FR-3**: `stock_vs_ours` quality path SHALL NOT emit the adversarial stub
  signature `intent ≈ 1` + `judge = 0` + `hallu = 0` for adversarial fixtures
  (after PR-C stub rewrite lands or as part of this feature).
- **FR-4**: Cockpit `resolveRlvr` SHALL report `source: "harness"` when cell
  `rlvr_*` fields are present (preferring harness over synthetic/fallback).
- **FR-5**: Contract docs / CELL assignment metrics SHALL document the required
  `rlvr_*` field set and finiteness expectations for dry ablation cells.

## Non-Functional Requirements
- Changes stay scoped to pheno-harness ablation/comparison contracts and
  phenotype-omlx bench-cockpit RLVR resolve path; no unrelated cockpit UI churn.
- Tests MUST reference FR IDs (tag, marker, or docstring) per Phenotype QA policy.
- Fail loudly on missing required RLVR fields for dry cells — no silent
  zero-fill that masquerades as truth.

## Constraints & Dependencies
- Depends on merged: pheno-harness #33 (`rlvr_*` emission), phenotype-omlx #46
  (Cell passthrough).
- Coordinates with in-flight PR-C (`stock_vs_ours` quality stubs rewrite); do not
  regress that rewrite.
- Related AgilePlus feature: `dual-harness-benchmark-optimize` (broader harness
  bench); this feature is narrower — RLVR truth fidelity only.
- Local quality gates only (GitHub Actions billing constraint); verify with
  harness/omlx unit tests before merge.

## Acceptance Criteria
- Dry ablation cells have finite `rlvr_composite` and `rlvr_l0..l3`.
- Go Cell round-trip preserves `rlvr_*`.
- `stock_vs_ours` does not emit intent≈1 + judge=0 + hallu=0 for adversarial fixtures.
- Cockpit `resolveRlvr` reports `source: harness` when fields present.
- `agileplus list` shows feature `harness-rlvr-truth` in state `specified` (or later).
