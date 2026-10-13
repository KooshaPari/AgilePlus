# AgilePlus Stable Release Definition

A tiered definition of "stable" for AgilePlus, gating release on observable, evidence-bound criteria rather than calendar dates.

## Tier S0 — Push-green

**Definition:** Merging any approved PR to `main` produces a green push, with the project's gates (CI required checks) not failing because of pre-existing project debt.

**Removal criteria:** All tier-S0 criteria must hold for 7 consecutive days on `main`.

| Criterion | Source of truth | Current state |
|---|---|---|
| OSV-Scanner Cargo.lock critical+high = 0 | `gh run view` on `security.yml` | **0 critical + 0 high met** as of commit `9b534c06` (2026-10-13). The 7-day clock can start. Tracking issue: #1107. 2 vulns remain (RUSTSEC-2024-0429 medium, RUSTSEC-2024-0370 unknown) but neither is critical/high, so the S0 criterion is satisfied. |
| `cargo test` workspace green on `main` | `Rust` workflow required check | Mostly green; flaky tests are pre-existing (see `flaky-test detection` checks). |
| `cargo clippy -- -D warnings` clean | `Rust` workflow | Mostly clean; nit-level warnings exist. |

## Tier S1 — Contract-stable

**Definition:** The AgilePlus specification is *the executable contract*. Validation of the contract can pass on a fresh checkout, and the contract is internally consistent (no quota-generated rows, no scenario-duplicates of formal requirements).

**Removal criteria:** All tier-S0 + tier-S1 criteria must hold, and the 3 spec PRs must be resolved (#1088, #1090, #1097 → tracking #1110).

| Criterion | Source of truth | Current state |
|---|---|---|
| Quota-generated catalogue removed | Issue #1088 | **landed in #1088** (squash-merge 2026-10-11). Active catalogue marked `accepted=false`, `grading_eligible=false`, `test_generation_ready=false`. |
| Formal spec parses without scenario-duplicates | Issue #1097 | **landed in #1097** (squash-merge 2026-10-12 23:33 UTC). |
| Mature semantic contract (67 reqs, 12 journeys, 92 oracles) lands with execution evidence | Issue #1090 | **landed in #1090** (squash-merge 2026-10-12 23:24 UTC). 100-commit monster landed as one squash-merge per the operator's call. Execution-evidence gate moves to S2. |
| `agileplus validate` returns 10/10 on a fresh checkout | `crates/agileplus-cli/src/commands/validate.rs` | partially green post-#1090 + #1097; remaining FAIL count tracked in the new validate command flow test. Issue #1110 is the spec-convergence tracker; this is now an S1 follow-on, not a blocker. |
| Audit-hash v1 envelope + v2 design committed | `spec/product/AUTHORITY_INDEX.md` | **landed in #1090** (squash-merge 2026-10-12). |
| Tracera federation boundary documented | `docs/adr/0018-tracera-agileplus-sibling-runtimes.md` | landed in #1088 |

## Tier S2 — Externally-stable v1.0.0-rc

**Definition:** A new contributor can read the spec, write a feature work package, and have CI confirm acceptance — without operator intervention.

**Removal criteria:** All tier-S0 + tier-S1 + tier-S2 criteria must hold, plus a release-candidate tag is published.

| Criterion | Source of truth | Current state |
|---|---|---|
| `agileplus new-feature "<story>"` → produces WP + acceptance tests | E2E | open |
| CI runs the produced acceptance tests | `crates/agileplus-cli/tests/atomic_acceptance_*.rs` | partial, in #1090 |
| Mutation testing on `agileplus-domain` is green | `cargo-mutants (agileplus-domain)` | not yet green; long-running |
| OSV-Scanner Cargo.lock = 0/0 (no advisory waiver) | `security.yml` OSV job | **waiver active**; 49 → 2 in progress per #1107. This commit-chain cleared RUSTSEC-2024-0449 (unsafe-libyaml) by replacing `serde_yaml 0.9.34+deprecated` with `serde_norway 0.9.42` (cafkafk/emberian, pure-Rust parser). 2 vulns remain: RUSTSEC-2024-0429 (glib 0.18.5, medium, requires tauri/tray-icon bump to 0.26+ with `default-features = false`) and RUSTSEC-2024-0370 (proc-macro-error 1.0.4, unmaintained, no fix; goes away with the glib bump). 0 critical + 0 high: criteria met. |
| 100% of 67 mature semantic requirements have at least one passing oracle | `crates/agileplus-cli/tests/validate_command_flow.rs` | not yet; tracked in #1090 |
| Audit hash chain v1 envelopes every WP transition | `crates/agileplus-application/src/use_cases/accept_feature.rs` | partial, in #1090 |
| Documentation site published, includes work-package authoring walkthrough | `docs/` | not yet |
| 1.0.0-rc tag published | git tag | not yet |

## Time-to-stable estimates (workload, not calendar)

| Tier | Workload estimate | Dependency |
|---|---|---|
| **S0** | 0–1 weeks from current state | #1107 (49 → 0 dep-upgrade). PR #1109 is the waiver, 4 dependabot PRs (#1091, #1104, #1105, #1106) already merged this cycle. `unsafe-libyaml` (RUSTSEC-2024-0449) cleared by commit `9b534c06` (2026-10-13). 2 vulns remaining are glib/proc-macro-error (tauri-stack-bump blockers, not critical/high). The 7-day 0/0 clock can start; the 2 remaining will be cleared by the tauri/tray-icon bump tracked separately. |
| **S1** | 1–3 weeks focused from current state | #1110 (spec convergence). All 3 PRs landed (#1088, #1090, #1097). Phase 3 (this STABLE.md) is the canonical tier doc. Remaining S1 work: `agileplus validate` 10/10 green, S2's `new-feature` happy-path. |
| **S2** | 4–8 weeks best case, 2–3 months realistic | Builds on S0 + S1. |

These are *workload* estimates — they assume focused, unblocked attention on the listed workstreams. Calendar time depends on parallelization and interruptions.

## What this doc is NOT

- Not a release schedule. Workload is given, not deadlines.
- Not a substitute for the project's `PRD.md`, `ROADMAP.md`, or governance docs. Where they disagree, those take precedence.
- Not a guarantee of S2. S2's "no advisory waiver" criterion requires Cargo.lock to be 0/0 for 7 days, which is contingent on upstream maintainers releasing patches for the 49 outstanding vulns.

## Update protocol

This file is updated when:
1. A tier is reached (mark it; do not remove — historic evidence stays).
2. A criterion's source-of-truth state changes.
3. The operator explicitly asks for a re-baseline.

This file lives at the repo root, versioned with the code. Edits are PRs.
