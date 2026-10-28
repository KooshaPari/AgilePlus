# E.2 / E.5 Validation (P1+P2) — t-rex, 2026-10-18T19:36Z

**Verdict**: PARTIAL
**Subagent session**: t-rex (`session_snake_1792352065409_e571d49ceb42da4e`)
**Wall-clock**: ~95 s (start 19:34:31Z, end 19:36:07Z)
**Cheap closes filed**: 2

## Setup

- Working directory: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus`
- Branch: `docs/state-forward-docket-20261017`
- Pre-existing untracked file (not from this run): `docs/sessions/20261017-state-forward-docket/dependabot_bumps_2026-10-18-stingray.log`
- Build: `cargo build --workspace --all-targets` → exit 0, 13.19 s (target/ warm cache)
- No source mutations made during validation.

## Source-of-truth mapping

`verification/FORWARD_WBS_2026-10-04.md` lines 69–109 define P1+P2. The P1+P2 sub-tasks map to the existing test suites as follows:

### P1 (HTTP acceptance)

P1 sub-task 1 (OpenAPI contract) — verified by `crates/agileplus-api/src/routes/acceptance.rs` (production code) and the test runner below. Production code was not modified by this validation; OpenAPI documentation artifact presence is out of scope for a validation cheap_close.

P1 sub-task 2 (idempotency / request_id semantics) — verified by tests below.

P1 sub-task 3 (the 6 provable cases):
| WBS case | Test name (file: `crates/agileplus-api/tests/atomic_acceptance_http.rs`) | Verdict |
|---|---|---|
| missing feature → 404 | `missing_feature_returns_404_without_acceptance_side_effects` (l. 317) | PASS |
| stale governance version → 409 | `stale_governance_expectation_cannot_accept_over_http` (l. 330) | PASS |
| bad correctness/governance → 422 | `client_cannot_supply_a_pass_or_actor_override` (l. 353) | **PASS but weak assertion** (see cheap close E.2.P1-422) |
| storage failure → sanitized 500 | `persistence_failure_over_http_leaves_no_terminal_state_or_receipt` (l. 376) | PASS |
| exact replay returns original receipt | `authenticated_http_acceptance_commits_and_replays_one_receipt` (l. 260) | PASS |
| generic feature transition cannot award Validated or Shipped | `terminal_feature_states_are_not_generic_transition_targets` (file: `crates/agileplus-api/src/routes/features.rs:392`) | **NO PRODUCTION ENFORCEMENT** (see cheap close E.2.P1-GENERIC) |

P1 sub-task 4 (receipt identity retrievable after restart) — `mounted_receipt_survives_database_reopen_without_regrading` (l. 218). PASS.

P1 sub-task 5 (restart-backed DB integration witness) — same test as P1-4. PASS.

### P2 (gRPC validation parity)

P2 canonical path rules map to tests in `crates/agileplus-grpc/tests/atomic_acceptance_rpc.rs`:

| WBS rule                                 | Test name                                                                                                                                                           | Verdict |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------- |
| RPC → resolve feature                    | `grpc_commits_and_replays_canonical_acceptance` (setup resolves feature)                                                                                            | PASS    |
| durable request identity                 | `grpc_commits_and_replays_canonical_acceptance` (replay returns same receipt) + `grpc_same_request_id_with_changed_command_conflicts_instead_of_regrading` (l. 269) | PASS    |
| AcceptFeatureCommand / accept_feature    | `grpc_commits_and_replays_canonical_acceptance` (l. 134)                                                                                                            | PASS    |
| AtomicAcceptancePort / AcceptanceOutcome | `grpc_commits_and_replays_canonical_acceptance`                                                                                                                     | PASS    |
| no gRPC-local governance evaluator       | `grpc_requires_auth_and_rejects_grading_overrides_without_mutation` (l. 169)                                                                                        | PASS    |
| no gRPC-local candidate evaluator        | `grpc_requires_auth_and_rejects_grading_overrides_without_mutation`                                                                                                 | PASS    |
| no direct state mutation                 | `grpc_rejects_stale_governance_version_without_mutation` (l. 246) + `grpc_storage_failure_rolls_back_and_redacts_details` (l. 219)                                  | PASS    |
| request identity stable / idempotent     | `grpc_same_request_id_with_changed_command_conflicts_instead_of_regrading`                                                                                          | PASS    |
| version the API if proto can't represent | not directly tested; design rule, not a test surface                                                                                                                | N/A     |

---

## P1 — HTTP acceptance

**Test file**: `crates/agileplus-api/tests/atomic_acceptance_http.rs`
**Runner**: `cargo test -p agileplus-api --test atomic_acceptance_http --no-fail-fast`
**Exit code**: 0
**Results**: 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25 s

Verbatim tail of the run:

```
running 10 tests
test unauthorized_http_cannot_reach_acceptance ... ok
test receipt_lookup_redacts_storage_failures ... ok
test missing_feature_returns_404_without_acceptance_side_effects ... ok
test unsupported_embedder_returns_501_not_an_optimistic_success ... ok
test client_cannot_supply_a_pass_or_actor_override ... ok
test receipt_lookup_requires_auth_and_never_awards_acceptance ... ok
test stale_governance_expectation_cannot_accept_over_http ... ok
test persistence_failure_over_http_leaves_no_terminal_state_or_receipt ... ok
test authenticated_http_acceptance_commits_and_replays_one_receipt ... ok
test mounted_receipt_survives_database_reopen_without_regrading ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
```

| #   | Test name                                                           | P1 sub-task                                   | Verdict                                |
| --- | ------------------------------------------------------------------- | --------------------------------------------- | -------------------------------------- |
| 1   | `unauthorized_http_cannot_reach_acceptance`                         | P1-2 auth pre-condition                       | PASS                                   |
| 2   | `receipt_lookup_redacts_storage_failures`                           | P1-3 storage-failure sanitization (read path) | PASS                                   |
| 3   | `missing_feature_returns_404_without_acceptance_side_effects`       | P1-3 missing feature → 404                    | PASS                                   |
| 4   | `unsupported_embedder_returns_501_not_an_optimistic_success`        | P1-2 negative path / config gate              | PASS                                   |
| 5   | `client_cannot_supply_a_pass_or_actor_override`                     | P1-3 bad governance/correctness → 422         | PASS (assertion weak — see E.2.P1-422) |
| 6   | `receipt_lookup_requires_auth_and_never_awards_acceptance`          | P1-2 receipt lookup is read-only              | PASS                                   |
| 7   | `stale_governance_expectation_cannot_accept_over_http`              | P1-3 stale governance → 409                   | PASS                                   |
| 8   | `persistence_failure_over_http_leaves_no_terminal_state_or_receipt` | P1-3 storage failure → sanitized 500          | PASS                                   |
| 9   | `authenticated_http_acceptance_commits_and_replays_one_receipt`     | P1-2 idempotency + P1-3 exact replay          | PASS                                   |
| 10  | `mounted_receipt_survives_database_reopen_without_regrading`        | P1-4 / P1-5 restart persistence               | PASS                                   |

---

## P2 — gRPC validation

**Test file**: `crates/agileplus-grpc/tests/atomic_acceptance_rpc.rs`
**Runner**: `cargo test -p agileplus-grpc --test atomic_acceptance_rpc --no-fail-fast`
**Exit code**: 0
**Results**: 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13 s

Verbatim tail of the run:

```
running 6 tests
test grpc_requires_auth_and_rejects_grading_overrides_without_mutation ... ok
test grpc_rejects_foreign_scope ... ok
test grpc_rejects_stale_governance_version_without_mutation ... ok
test grpc_storage_failure_rolls_back_and_redacts_details ... ok
test grpc_commits_and_replays_canonical_acceptance ... ok
test grpc_same_request_id_with_changed_command_conflicts_instead_of_regrading ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

| #   | Test name                                                                  | P2 rule                                                                                                               | Verdict |
| --- | -------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | ------- |
| 1   | `grpc_commits_and_replays_canonical_acceptance`                            | resolve feature / durable identity / AcceptFeatureCommand / accept_feature / AtomicAcceptancePort / AcceptanceOutcome | PASS    |
| 2   | `grpc_requires_auth_and_rejects_grading_overrides_without_mutation`        | no gRPC-local evaluator (governance + candidate) + no direct mutation                                                 | PASS    |
| 3   | `grpc_rejects_foreign_scope`                                               | canonical-repo-root scope enforcement (no mutation)                                                                   | PASS    |
| 4   | `grpc_storage_failure_rolls_back_and_redacts_details`                      | no direct mutation + storage-failure sanitization                                                                     | PASS    |
| 5   | `grpc_rejects_stale_governance_version_without_mutation`                   | no gRPC-local governance + no mutation                                                                                | PASS    |
| 6   | `grpc_same_request_id_with_changed_command_conflicts_instead_of_regrading` | stable request identity (same id → same receipt, changed command → conflict, never regrades)                          | PASS    |

---

## Edge cases (FORWARD_WBS P1-3 explicit list)

| Edge case                                                    | Test                                                                                                             | HTTP/code                                                                                      | Verdict                                                                           |
| ------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| missing feature → 404                                        | `missing_feature_returns_404_without_acceptance_side_effects`                                                    | `StatusCode::NOT_FOUND` (l. 323)                                                               | PASS                                                                              |
| stale governance → 409                                       | `stale_governance_expectation_cannot_accept_over_http`                                                           | `StatusCode::CONFLICT` (l. 338)                                                                | PASS                                                                              |
| bad correctness/governance → 422                             | `client_cannot_supply_a_pass_or_actor_override`                                                                  | `is_client_error()` (l. 370)                                                                   | PASS but assertion is `is_client_error()`, not `StatusCode::UNPROCESSABLE_ENTITY` |
| storage failure → sanitized 500                              | `persistence_failure_over_http_leaves_no_terminal_state_or_receipt`                                              | `StatusCode::INTERNAL_SERVER_ERROR` (l. 384); body redacted ("private database detail" absent) | PASS                                                                              |
| exact replay returns original receipt                        | `authenticated_http_acceptance_commits_and_replays_one_receipt`                                                  | `replayed=true`, `first["receipt"] == replay["receipt"]` (l. 286)                              | PASS                                                                              |
| generic feature transition cannot award Validated or Shipped | `terminal_feature_states_are_not_generic_transition_targets` (`crates/agileplus-api/src/routes/features.rs:392`) | tautology only (asserts variants exist)                                                        | **NO PRODUCTION ENFORCEMENT**                                                     |

Production-code evidence for the 422 mapping exists at `crates/agileplus-api/src/routes/acceptance.rs:152-155`:

```
DomainError::Validation(message) => (
    StatusCode::UNPROCESSABLE_ENTITY,
    Json(json!({"error":"acceptance_rejected", "message":message})),
),
```

So 422 IS wired. The test simply does not pin it.

Production-code evidence for "generic cannot award Validated or Shipped" does NOT exist:

- `crates/agileplus-domain/src/domain/feature.rs:57-74` — `Feature` struct has no `kind`, `type`, or `category` field
- Workspace-wide grep for `feature_kind`, `FeatureKind`, `is_generic`, `feature_category` over `crates/`: zero hits
- The only test (`terminal_feature_states_are_not_generic_transition_targets`) is a tautology: it iterates `[FeatureState::Validated, FeatureState::Shipped]` and asserts `matches!(target, FeatureState::Validated | FeatureState::Shipped)`, which always succeeds.

---

## New cheap closes filed

### E.2.P1-422 — Pin the 422 status in `client_cannot_supply_a_pass_or_actor_override`

- **Cell**: E.2 / E.5
- **Move**: tighten the assertion in `crates/agileplus-api/tests/atomic_acceptance_http.rs:370` from `response.status_code().is_client_error()` to `assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY)` for the `passed: true`, `authoritative: true`, and `actor: "owner"` payloads.
- **Why**: production code at `acceptance.rs:152-155` already maps `DomainError::Validation` to HTTP 422; the test passes for any 4xx, so a future regression that returns 400 or 409 would still satisfy the assertion and silently weaken the contract.
- **eta_min**: 10
- **Evidence**:
    ```
    $ rg -n "is_client_error|UNPROCESSABLE_ENTITY" crates/agileplus-api/tests/atomic_acceptance_http.rs
    370:        assert!(response.status_code().is_client_error());
    ```
- **Risk**: zero behavior change; assertion tightening only.

### E.2.P1-GENERIC — Implement the "generic feature transition cannot award Validated or Shipped" rule

- **Cell**: E.2 / E.5
- **Move**:
    1. Decide whether "generic" is a property of the **Feature** (new `kind` field with `Generic`/`Specific`) or of the **governance contract** (e.g. missing acceptance contract ⇒ transitions to Validated/Shipped are rejected). Either is defensible; the WBS does not specify.
    2. Implement the rule in `crates/agileplus-domain/src/domain/feature.rs::Feature::transition()` (lines 95–115) and/or the atomic acceptance port at `crates/agileplus-application/src/use_cases/accept_feature.rs` (currently 17 lines — currently only documents that "callers cannot supply a boolean governance pass or separately write terminal state").
    3. Replace the tautology at `crates/agileplus-api/src/routes/features.rs:392` with a behavior test that attempts `transition(FeatureState::Implementing, FeatureState::Validated)` on a feature whose governance contract is absent (or whose `kind == Generic`) and asserts `Err(...)`.
- **Why**: the WBS sub-task 3 sixth case is the only P1-3 case without production enforcement and without a real test. Today, any feature (even one without governance) can be transitioned to `Validated`/`Shipped` via the feature-level state machine.
- **eta_min**: 60 (rule design + 2 acceptance-port changes + 2 tests)
- **Evidence**:
    ```
    $ rg -n "feature_kind|FeatureKind|is_generic|feature_category" crates/ --include='*.rs'
    (no matches)
    $ sed -n '57,74p' crates/agileplus-domain/src/domain/feature.rs
    pub struct Feature {
        pub id: i64,
        pub slug: String,
        pub friendly_name: String,
        pub state: FeatureState,
        pub spec_hash: [u8; 32],
        pub target_branch: String,
        pub plane_issue_id: Option<String>,
        pub plane_state_id: Option<String>,
        pub labels: Vec<String>,
        pub module_id: Option<i64>,
        pub project_id: Option<i64>,
        pub created_at_commit: Option<String>,
        pub last_modified_commit: Option<String>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }
    ```
    ```
    $ sed -n '391,399p' crates/agileplus-api/src/routes/features.rs
    fn terminal_feature_states_are_not_generic_transition_targets() {
        for target in [FeatureState::Validated, FeatureState::Shipped] {
            assert!(matches!(
                target,
                FeatureState::Validated | FeatureState::Shipped
            ));
        }
    }
    ```
- **Risk**: medium — requires product decision (where does "generic" live?). Likely blocks the P1 exit criterion until resolved.

---

## Blockers

- **E.2.P1-GENERIC** is the only blocking issue. Without an enforcement decision + production change + behavior test, the P1 exit criterion ("HTTP is a projection of the application acceptance use case, not a parallel lifecycle implementation") is partially unmet for the "generic cannot award Validated or Shipped" sub-case. All other P1+P2 sub-tasks have current evidence and pass.
- E.2.P1-422 is a test-quality fix; not blocking, but cheap to land.

## Operator action required

1. Decide the product semantics of "generic feature" — E.2.P1-GENERIC, eta 60 m.
2. Land the assertion tightening — E.2.P1-422, eta 10 m.
3. Re-run `cargo test -p agileplus-api --test atomic_acceptance_http --no-fail-fast` and `cargo test -p agileplus-grpc --test atomic_acceptance_rpc --no-fail-fast` after each fix to confirm green.

## Operator-safe verification log

- `tail -F /tmp/e2_e5_p1p2_val_2026-10-18.log` for the full tee stream.
- P1 raw run: `/tmp/e2_e5_p1p2_p1_test.out`
- P2 raw run: `/tmp/e2_e5_p1p2_p2_test.out`
- Build: `cargo build --workspace --all-targets` → exit 0 in 13.19 s.
- Working tree clean (no source files added, no patches, no commits); only pre-existing untracked file `docs/sessions/20261017-state-forward-docket/dependabot_bumps_2026-10-18-stingray.log` from a prior session remains.
