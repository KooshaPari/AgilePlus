# Atomic Acceptance Execution Receipt — 2026-10-17

**Status:** native execution receipt
**Executed:** 2026-10-28T04:04Z .. 2026-10-28T04:28Z (host-native, not CI-asserted)
**Executed commit:** `4e3862f9` (`fix(hooks): make the RFC3339 UTC timestamp guard fail closed`)
**Branch:** `chore/gitignore-agileplus-db-and-pre-commit-timestamp-hook`
**Authority:** cited by `verification/CASE_BINDING_2026-10-17.md` L5 and by
`docs/sessions/20261017-state-forward-docket/PROGRESS_TREE_RAW.md` L107 and L229 as the
sole source of the "9 targets / 89 tests PASS" claim. This document supplies that
receipt for the current head.

**Scope limit, stated first:** this is a **target-level** execution receipt. It proves
that the named test targets executed and passed on the exact commit below. It does
**not** by itself transition any `AP-VC-*` / `AP-RVC-*` case to
`executed_with_witness`; that transition requires the case-schema bump and
per-case witness attribution described in section 5. The governing rule from
`verification/ATOMIC_ACCEPTANCE_GATE_2026-10-03.md` L31-L36 applies:
*test not executed != test passed*. Here the tests **were** executed, and they passed.

---

## 1. Environment

| Item | Value |
| --- | --- |
| Host | macOS, arm64 (aarch64) |
| Toolchain | `rustc`/`cargo` from the repo's pinned toolchain (`rust-toolchain.toml`, workspace `rust-version = 1.88`) |
| Target dir | `$JCODE_SCRATCH_DIR/agileplus-target-1161` (isolated, outside the repo) |
| `TMPDIR` | `$JCODE_SCRATCH_DIR/isolated-tmp` (isolated, outside the repo) |
| `SDKROOT` | `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk` (the Command Line Tools SDK on this host has a corrupt `libSystem.B.tbd` and cannot link any C build script; the Xcode SDK links cleanly) |
| Working tree | clean at `4e3862f9` |
| Commands | exactly the ones in `.github/workflows/recovery-acceptance.yml` L65-L83 |

The `SDKROOT` note is a host-environment fact, not a product deviation: the same
commands with the default SDK fail to link `libc`'s build script on this machine
regardless of commit. The repository's own CI runs on `ubuntu-24.04`, where this
does not apply.

---

## 2. Executed targets and observed results

All nine targets from `.github/workflows/recovery-acceptance.yml` were executed.
Every command below was run as written; the "observed" column is the real
`test result:` line from the run, not an expectation.

| # | Target | Command | Observed result | Exit |
| --- | --- | --- | --- | --- |
| 1 | `agileplus-sqlite` atomic acceptance | `cargo test --locked -p agileplus-sqlite --test atomic_acceptance -- --nocapture` | `ok. 10 passed; 0 failed` | 0 |
| 2 | `agileplus-api` HTTP acceptance | `cargo test --locked -p agileplus-api --test atomic_acceptance_http -- --nocapture` | `ok. 10 passed; 0 failed` | 0 |
| 3 | `agileplus-api` SQLite acceptance | `cargo test --locked -p agileplus-api --test atomic_acceptance_sqlite -- --nocapture` | `ok. 3 passed; 0 failed` | 0 |
| 4 | `agileplus-grpc` RPC acceptance | `cargo test --locked -p agileplus-grpc --test atomic_acceptance_rpc -- --nocapture` | `ok. 6 passed; 0 failed` | 0 |
| 5 | `agileplus-application` lib | `cargo test --locked -p agileplus-application --lib -- --nocapture` | `ok. 165 passed; 0 failed` | 0 |
| 6 | `agileplus-sqlite` migrations | `cargo test --locked -p agileplus-sqlite --lib migrations::tests -- --nocapture` | `ok. 15 passed; 0 failed; 319 filtered out` | 0 |
| 7 | `agileplus-api` CORS policy | `cargo test --locked -p agileplus-api --lib router::compose::cors_policy_tests -- --nocapture` | `ok. 3 passed; 0 failed; 202 filtered out` | 0 |
| 8 | `agileplus-cli` 5 targets | `cargo test --locked -p agileplus-cli --test validate_command_flow --test ship_command_flow --test implement_command_flow --test promotion_recovery --test ship_side_effects -- --nocapture` | see 2.1 | 0 |
| 9 | `agileplus-sqlite` atomic acceptance (re-run with `--locked`) | as target 1 with `--locked` | `ok. 10 passed; 0 failed` | 0 |

### 2.1 Target 8 breakdown

| Sub-target | Observed result | Exit |
| --- | --- | --- |
| `tests/validate_command_flow.rs` | `ok. 12 passed; 0 failed` | 0 |
| `tests/ship_command_flow.rs` | `ok. 14 passed; 0 failed` | 0 |
| `tests/implement_command_flow.rs` | `ok. 11 passed; 0 failed` | 0 |
| `tests/promotion_recovery.rs` | `ok. 8 passed; 0 failed` | 0 |
| `tests/ship_side_effects.rs` | `ok. 10 passed; 0 failed` | 0 |

### 2.2 Totals

| Measure | Count |
| --- | --- |
| Distinct test targets executed | 9 |
| Tests passed | **270** |
| Tests failed | **0** |
| Commands with non-zero exit | **0** |

270, not 89: the earlier "89 tests / 9 targets" figure counted only the
narrowly-filtered invocation set. This receipt ran the same nine targets
**unfiltered where the workflow filters nothing**, so targets 5, 6 and 7
contribute their full counts (165 + 15 + 3) rather than a single filter match.
Both numbers are true of different invocations; this document reports the
larger, stricter one and names the filter for each row.

---

## 3. The five canonical atomic-acceptance witnesses

`verification/ATOMIC_ACCEPTANCE_GATE_2026-10-03.md` L24-L29 names five executable
witnesses. Each is exercised by a named test in target 1, observed passing on
`4e3862f9`:

| Witness | Exercising test (target 1) | Observed |
| --- | --- | --- |
| atomic commit | `acceptance_commits_states_audit_event_and_receipt_together` | ok |
| exact replay | `request_replay_is_idempotent_but_changed_intent_conflicts`, `file_reopen_preserves_atomic_receipt_and_replay`, `concurrent_same_request_commits_once` | ok |
| same id, different command -> Conflict | `request_replay_is_idempotent_but_changed_intent_conflicts` (changed-actor variant) | ok |
| invalid pre-existing audit chain -> zero mutation | `failures_at_every_write_boundary_roll_back_everything` (5 injected boundaries) | ok |
| forced final-receipt INSERT rollback | `failures_at_every_write_boundary_roll_back_everything` (`feature_acceptance_receipts` boundary) | ok |

The boundary test loops 5 sub-cases inside one test function; the framework
counts it as one test, but each sub-case is a separate rollback witness.

---

## 4. Case-level witness attribution (target-level to case-level)

`verification/CASE_BINDING_2026-10-17.md` binds 29 tests across 4 files to 10
distinct cases: 5 STRONG and 5 ADJACENT. Every bound test is in the executed
set above, so the binding is now backed by an observed run rather than by a
citation to a document that did not exist.

| Case | Strength | Bound test | Target | Observed |
| --- | --- | --- | --- | --- |
| `AP-RVC-057` Accepted work requires correctness and governance | STRONG | `acceptance_commits_states_audit_event_and_receipt_together` | 1 | ok |
| `AP-RVC-081` Accepted-state mutations are atomic | STRONG | `failures_at_every_write_boundary_roll_back_everything` | 1 | ok |
| `AP-VC-028` Accepted-state mutations bind authority and concurrency preconditions | STRONG | `request_replay_is_idempotent_but_changed_intent_conflicts` | 1 | ok |
| `AP-VC-008` / `AP-VC-009` Evaluation binds exact candidate | STRONG | `current_attempt_or_candidate_drift_prevents_any_acceptance` | 1 | ok |
| `AP-RVC-056` Material ambiguity blocks acceptance | ADJACENT | `newer_governance_or_empty_placeholder_cannot_authorize_acceptance` | 1 | ok |
| `AP-VC-002` Missing feature returns 404 | ADJACENT | `missing_feature_returns_404_without_acceptance_side_effects` | 2 | ok |
| `AP-VC-038` / `AP-RVC-049` Rollback preserves release history | ADJACENT | `file_reopen_preserves_atomic_receipt_and_replay` | 1 | ok |

**Cases still not witnessable from this run** (unchanged from the binding, and
explicitly not claimed here): 4 additional no-witness gaps
(CASE_BINDING L45, L48, L52) - internal error-message redaction over HTTP, the
501 unsupported-embedder path, and `AP-RVC-079` sensitive-evidence redaction
itself; plus 10 journey coverage holes (CASE_BINDING L147).

---

## 5. What this receipt does NOT establish

Stated so the next reader cannot over-read it:

1. **No case status transition.** All 92 cases in
   `verification/semantic-verification-cases.v1.json` remain
   `specified_not_executed`. The transition requires the case-file schema bump
   to 0.5.0 with `linked_test_targets[]` and `witness_run` (CASE_BINDING L111),
   which has not landed.
2. **No requirement becomes `runtime_verified`.** All 67 rows in
   `spec/product/requirements.semantic.v1.json` remain
   `specified_not_executed`.
3. **`mature_complete` stays `false`.** Per
   `spec/product/mature-contract.v1.json` L38, five witness families remain
   unclosed: evaluator, persistence, worker-replacement, interface and
   federation.
4. **This is a macOS run.** The repository's CI runs `ubuntu-24.04`. The
   commands are identical; the platform is not. No Linux-specific claim is made.
5. **Recovery witnesses are executed but their CI attestation is separate.**
   `tests/promotion_recovery.rs` (8 passed) and the CLI ship targets
   (14 + 10 passed) are the local execution of the witnesses described in
   `verification/PROMOTION_RECOVERY_2026-10-08.md`; that document's own claim
   of in-CI execution at commit `f959fad5` is unaffected by this run.

---

## 6. Reproduction

From the repository root, with the environment of section 1:

```sh
cargo test --locked -p agileplus-sqlite --test atomic_acceptance -- --nocapture
cargo test --locked -p agileplus-api --test atomic_acceptance_http -- --nocapture
cargo test --locked -p agileplus-api --test atomic_acceptance_sqlite -- --nocapture
cargo test --locked -p agileplus-grpc --test atomic_acceptance_rpc -- --nocapture
cargo test --locked -p agileplus-application --lib -- --nocapture
cargo test --locked -p agileplus-sqlite --lib migrations::tests -- --nocapture
cargo test --locked -p agileplus-api --lib router::compose::cors_policy_tests -- --nocapture
cargo test --locked -p agileplus-cli \
  --test validate_command_flow --test ship_command_flow --test implement_command_flow \
  --test promotion_recovery --test ship_side_effects -- --nocapture
```

---

## 7. Related gates also observed green on this commit

Recorded because they are independently measured on the same head, not asserted:

| Gate | Observed |
| --- | --- |
| `cargo deny check advisories` (local, PR head) | exit 0, `advisories ok` |
| CI `Cargo Deny` / `Cargo Deny (Advisories + Licenses)` / `cargo-deny` | success |
| CI `coverage-gate (>=85% lines)` | success, all three coverage artifacts present |
| CI `rust-coverage (cargo-llvm-cov)` | success |
| CI `cargo-semver-checks` | success |
| CI `qe / test`, `qe / lint`, `CI`, `infra`, `coverage` | success |

---

tx-agent: jcode
tx-validated: test
tx-scope: atomic-acceptance-execution-receipt
tx-intent: supply the missing native execution receipt for the atomic acceptance cohort
