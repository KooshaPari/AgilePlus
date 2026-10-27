# E.2.P1-GENERIC Investigation (raven, 2026-10-18T20:05Z)

**Verdict**: PASS-WITH-OPERATOR-DECISION (investigation complete, design proposed,
awaiting operator pick).
**Subagent session**: raven
**Wall-clock start**: 2026-10-18T20:02:53Z (scaffold); investigation began 20:03Z.
**Investigation window**: ~3 min wall-clock; well under the 45-minute cap.
**Cell**: E.2.P1-GENERIC
**Verdict before**: OPEN, BLOCKING the P1 exit criterion.
**Verdict after**: OPEN awaiting operator product decision; investigation advanced
to PARTIAL — the production-enforcement gap alleged by t-rex is partially
incorrect and the actionable gap is now identified as test-only.

---

## 0. TL;DR (the operator must read this)

1. The t-rex carry-forward's hypothesis (that "generic" is a `Feature.kind`
   property with a `Generic` variant) is **wrong**. No `FeatureKind` enum
   exists anywhere in the crate tree
   (`rg "FeatureKind|feature_kind|FeatureCategory"` → 0 hits in source).
2. The word "generic" in this codebase consistently means **"the generic
   (non-canonical, transport-level) state-transition path"** — i.e. the HTTP
   `POST /api/v1/features/{slug}/transition` route and the gRPC `validate`/
   `ship` dispatch handlers. Both must fail closed against Validated/Shipped.
3. Production enforcement of the rule **already exists** at the HTTP layer
   (`crates/agileplus-api/src/routes/features.rs:283-287` for transitions and
   `crates/agileplus-api/src/routes/features.rs:168-172` for creation) and at
   the gRPC layer (`crates/agileplus-grpc/src/server/mod.rs:705-714`). The
   canonical atomic-acceptance use case is `accept_feature.rs` (15-line file)
   and the dedicated `/accept` route is `crates/agileplus-api/src/routes/acceptance.rs`.
4. The **only real gap** is that the test at
   `crates/agileplus-api/src/routes/features.rs:391-399` is a
   tautology (it iterates the two values and asserts each matches the same
   pattern), and there is no HTTP-level behavior test for the rejection. This
   is a **test-quality gap**, not a production-code gap.
5. The P1 exit criterion ("HTTP is a projection of the application
   acceptance use case, not a parallel lifecycle implementation") is
   **substantively met today**; E.2.P1-GENERIC reduces to: delete the
   tautology, add one HTTP behavior test mirroring
   `coverage_create_feature_rejects_terminal_states`.

If the operator accepts (5), this cell closes in roughly **20-30 min** of
implementation work, not 60 min as t-rex estimated.

If the operator wants defense-in-depth — i.e. also enforce the rule at the
domain layer (`Feature::transition()`) and at the app layer
(`AdvanceFeature::execute()`) — see Design B / Design C below (45-90 min).

---

## 1. What "generic" means in the codebase

There is **no `Feature.kind` field, no `FeatureKind` enum, and no
`is_generic()` method** anywhere in the AgilePlus source tree
(`crates/agileplus-domain/src/domain/feature.rs:57-74` lists every field on
`Feature` and none relates to generic-ness; `rg "FeatureKind|feature_kind|\
FeatureCategory"` returns zero hits).

The word "generic" in this codebase means **"the generic state-transition
path"** — a path that mutates `Feature.state` directly, without going
through the canonical atomic-acceptance use case. There are three such
paths, all of which already fail closed:

- **HTTP transition route** (`crates/agileplus-api/src/routes/features.rs:265-305`):
  the handler's body, lines 283-287, explicitly rejects
  `FeatureState::Validated | FeatureState::Shipped` with HTTP 400
  BadRequest and the message "terminal feature states require the canonical
  validate/ship operation". The transition then delegates to the domain
  state machine at line 288 and persists at line 296.
- **HTTP create route** (`crates/agileplus-api/src/routes/features.rs:131-225`):
  the create handler, lines 168-172, similarly refuses to _create_ a
  feature in a terminal state, returning 400 BadRequest with "terminal
  feature states cannot be assigned at creation; use canonical
  acceptance/promotion".
- **gRPC core dispatch** (`crates/agileplus-grpc/src/server/mod.rs:695-714`):
  the `"validate" | "ship"` arm of the match returns `Status::unimplemented`
  with the message "command '{command}' requires the canonical
  acceptance/promotion service". Spec, retrospective, plan, etc. all flow
  through this generic path because they are NOT terminal transitions.
- **CLI `mvp` transition** (`crates/agileplus-cli/src/commands/mvp.rs:765`):
  the in-process `transition()` helper enforces "exact candidate-bound
  evaluation" — it does not let a CLI-internal `transition` self-award
  WpState::Done.

The test names that use the literal word "generic" match this reading:

- `crates/agileplus-grpc/tests/rpc_dispatch_command.rs:191`
  `generic validate must not mint terminal acceptance`
- `crates/agileplus-grpc/tests/rpc_dispatch_command.rs:203`
  `generic ship must not mint promotion`
- `crates/agileplus-grpc/tests/rpc_dispatch_command.rs:258`
  `dispatch_command_validate_cannot_bypass_canonical_acceptance` (note:
  this is the behaviour test for the gRPC rule)
- `crates/agileplus-grpc/tests/rpc_dispatch_command.rs:297`
  `dispatch_command_ship_cannot_bypass_canonical_promotion` (also
  behaviour)
- `crates/agileplus-grpc/tests/rpc_dispatch_command.rs:336`
  `dispatch_command_validate_blocks_when_required_evidence_is_missing`
- `crates/agileplus-grpc/tests/rpc_dispatch_command.rs:348`
  `generic_validate_never_interprets_governance_or_evidence_locally`
- `crates/agileplus-api/src/routes/features.rs:392`
  `terminal_feature_states_are_not_generic_transition_targets`
- `crates/agileplus-cli/src/commands/mvp.rs:765`
  `generic MVP transition must not self-award Done`

Reading across these tests, "generic" uniformly means **"the generic
state-transition path that does not consult canonical atomic-acceptance
evidence"**. There is no evidence in the codebase, the WBS, the mature
contract, or any other doc that "generic" means a `Feature.kind` value.

### 1a. Where the canonical (non-generic) path lives

The canonical atomic-acceptance path lives in
`crates/agileplus-application/src/use_cases/accept_feature.rs` (15 lines,
the whole file):

```rust
pub async fn accept_feature<S: AtomicAcceptancePort + ?Sized>(
    storage: &S,
    command: &AcceptFeatureCommand,
) -> Result<AcceptanceOutcome, AppError> {
    command.validate()?;
    Ok(storage.accept_feature_atomic(command).await?)
}
```

It is invoked from `crates/agileplus-api/src/routes/acceptance.rs:48`
(`POST /{slug}/accept`), backed by `AtomicAcceptancePort` from
`crates/agileplus-domain/src/ports/execution.rs`. The validation
(`command.validate()`) is where governance-contract evidence and required
rules are evaluated — only a feature that has satisfied them gets
minted `Validated`/`Shipped`.

### 1b. Why "generic" can never award terminal states

Per AGENTS.md (Recovery Authority 2026-10-04):

> Generic state-transition transports must not award terminal acceptance.

This is a _transport-layer_ invariant. The transport-layer code cannot
have access to governance contracts, evidence references, candidate
evaluators, or assignment receipts, so it must defer to the canonical
acceptance use case. The gRPC, HTTP `transition`, HTTP `create`, and CLI
`mvp` paths all satisfy this invariant today.

---

## 2. The tautology test (current state)

**File**: `crates/agileplus-api/src/routes/features.rs:391-399`

```rust
#[test]
fn terminal_feature_states_are_not_generic_transition_targets() {
    for target in [FeatureState::Validated, FeatureState::Shipped] {
        assert!(matches!(
            target,
            FeatureState::Validated | FeatureState::Shipped
        ));
    }
}
```

### What it actually asserts

For each `target` ∈ {`Validated`, `Shipped`}, it asserts that `target`
matches the literal pattern `Validated | Shipped`. This is a property of
the rust compiler, not of the production code under test. It is exactly
the same as writing `let v = Validated; assert!(matches!(v, Validated | Shipped));`.

### What it does NOT do

- It does NOT call `transition_feature()` (the HTTP handler at lines 265-305).
- It does NOT instantiate `AppState` / `StoragePort` / `TestServer`.
- It does NOT spin up an HTTP test server (no `setup_test_server()` helper is
  used; no `axum_test::TestServer` is constructed).
- It does NOT assert on `ApiError::BadRequest`, on the response body,
  or on a status code.
- It does NOT verify that `Feature.state` was not mutated to
  Validated/Shipped via the generic HTTP path.
- It does NOT cover the `create_feature` HTTP path even though
  `create_feature` also enforces the rule at lines 168-172.

It is a tautology. The test name promises more than the body delivers.

### Why this test was filed as a carry-forward by t-rex

Because the WBS
(`verification/FORWARD_WBS_2026-10-04.md:73-79`) requires P1 sub-task 3
"sixth case" coverage for "generic feature transition cannot award
Validated or Shipped", and the existing test of that name asserts nothing
about behaviour. Anyone reading `cargo test -p agileplus-api` output who
greps for "generic" will conclude (mistakenly) that the rule is
covered.

### Tautology at the line level

Line 395 says:

```rust
FeatureState::Validated | FeatureState::Shipped
```

That pattern is **literally the same expression** as the loop's input
iterator on line 393 (`[FeatureState::Validated, FeatureState::Shipped]`).
There is no production-code branch covered. The test is a no-op.

---

## 3. Where the rule is enforced (production code, today)

The rule is enforced in **three** places in the codebase. The t-rex
carry-forward's claim "there is NO production enforcement" is incorrect;
the gap is purely test-side.

### 3a. HTTP transition route (the WBS's "generic" path)

`crates/agileplus-api/src/routes/features.rs:265-305`:

```rust
pub async fn transition_feature<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
    Json(body): Json<TransitionRequest>,
) -> Result<Json<TransitionResponse>, ApiError>
where ...
{
    let feature = app
        .storage
        .get_feature_by_slug(&slug)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Feature '{slug}' not found")))?;

    let target = parse_feature_state(&body.target_state)?;
    if matches!(target, FeatureState::Validated | FeatureState::Shipped) {
        return Err(ApiError::BadRequest(
            "terminal feature states require the canonical validate/ship operation"
                .to_string(),
        ));
    }
    let result = agileplus_domain::domain::state_machine::transition(
        feature.state, target)
        .map_err(ApiError::from)?;
    ...
}
```

Lines 283-287 are the production enforcement. A POST to
`/api/v1/features/{slug}/transition` with `target_state ∈ {"validated",
"shipped"}` returns HTTP 400 BadRequest without ever consulting
`feature.transition()` (the domain state machine that _would_ otherwise
allow `Implementing -> Validated`). The handler then delegates to the
domain state machine at line 288 for non-terminal targets.

### 3b. HTTP create route (terminal-state creation)

`crates/agileplus-api/src/routes/features.rs:131-225`, lines 168-172:

```rust
    let initial_state = body
        .state
        .as_deref()
        .map(parse_feature_state)
        .transpose()?
        .unwrap_or(FeatureState::Created);
    if matches!(
        initial_state,
        FeatureState::Validated | FeatureState::Shipped
    ) {
        return Err(ApiError::BadRequest(
            "terminal feature states cannot be assigned at creation; \
             use canonical acceptance/promotion"
                .to_string(),
        ));
    }
```

This already has a behaviour test:
`crates/agileplus-api/tests/api_integration/route_coverage.rs:254-269`
(`coverage_create_feature_rejects_terminal_states`).

### 3c. Domain state machine

`crates/agileplus-domain/src/domain/state_machine.rs:9-22` is a thin
re-export from `traceability_core::lifecycle` that maps errors. It
**does not** enforce the rule. The lifecycle crate allows
`Implementing -> Validated` and `Validated -> Shipped` because those
are valid state transitions; the gate is at the _transport_ layer,
which is correct.

### 3d. Domain model

`crates/agileplus-domain/src/domain/feature.rs:96-114` (`Feature::transition`):

```rust
pub fn transition(&mut self, target: FeatureState) -> Result<(), String> {
    use FeatureState::*;
    let allowed = match self.state {
        Created => matches!(target, Specified),
        Specified => matches!(target, Researched),
        ...
        Implementing => matches!(target, Validated),
        Validated => matches!(target, Shipped),
        Shipped => matches!(target, Retrospected),
        Retrospected => false,
    };
    if allowed {
        self.state = target;
        self.updated_at = Utc::now();
        Ok(())
    } else {
        Err(...)
    }
}
```

`Feature::transition()` **does allow** `Implementing -> Validated` and
`Validated -> Shipped`. This is intentional: the domain permits the
state-machine-level transition; the transport layer is the gate.

### 3e. Application use case

`crates/agileplus-application/src/use_cases/advance_feature.rs:32-46`
(AdvanceFeature::execute) calls `feature.transition(target)` directly
(line 36). It therefore _would_ permit a Validated/Shipped transition
if called from app code. There is no HTTP route that calls
`AdvanceFeature::execute` for terminal targets (the HTTP `transition`
route calls `state_machine::transition` at line 288, not the use case),
so today this is a code-only attack surface and not an executable risk.

### 3f. Canonical atomic-acceptance use case

`crates/agileplus-application/src/use_cases/accept_feature.rs` (entire
file is 15 lines):

```rust
pub async fn accept_feature<S: AtomicAcceptancePort + ?Sized>(
    storage: &S,
    command: &AcceptFeatureCommand,
) -> Result<AcceptanceOutcome, AppError> {
    command.validate()?;
    Ok(storage.accept_feature_atomic(command).await?)
}
```

`command.validate()` is where the governance contract, evidence
references, candidate evaluator and the assignment lifecycle are
evaluated. Only features with satisfied evidence get a `Validated`
(or `Shipped` via promotion) outcome. The `/accept` HTTP route at
`crates/agileplus-api/src/routes/acceptance.rs:48` is the only
endpoint that fans into this use case.

### 3g. gRPC dispatch

`crates/agileplus-grpc/src/server/mod.rs:701-714`:

```rust
let target_state = match command {
    "specify" => FeatureState::Specified,
    "research" => FeatureState::Researched,
    "plan" => FeatureState::Planned,
    // Terminal acceptance and promotion have stronger invariants than a
    // generic state transition. Until gRPC is wired to the canonical
    // validate/ship application services, fail closed rather than minting
    // Validated/Shipped states from transport-local checks.
    "validate" | "ship" => {
        return Err(Status::unimplemented(format!(
            "command '{command}' requires the canonical acceptance/promotion service"
        )));
    }
    "retrospective" => FeatureState::Retrospected,
    other => {
        return Err(Status::unimplemented(format!("unknown command: '{other}'")));
    }
};
```

This is the gRPC-side mirror of the HTTP-side rule. The comment at
lines 706-708 literally uses the word "generic" to mean
"transport-local", matching the AGENTS.md invariant.

---

## 4. Design alternatives

### Design A — Minimal: replace the tautology with an HTTP behaviour test

This is the cheapest design. **No production code changes**. Just delete
the tautology test and replace it with a real HTTP behaviour test that
mirrors the existing
`coverage_create_feature_rejects_terminal_states`
(`crates/agileplus-api/tests/api_integration/route_coverage.rs:254-269`).

**Test shape** (sketch):

```rust
#[tokio::test]
async fn coverage_transition_feature_rejects_terminal_states() {
    let server = setup_test_server().await;
    for terminal in ["validated", "shipped"] {
        let resp = server
            .post("/api/v1/features/test-feature/transition")
            .add_header(KEY, TEST_API_KEY)
            .json(&serde_json::json!({ "target_state": terminal }))
            .await;
        resp.assert_status(StatusCode::BAD_REQUEST);
        let body: serde_json::Value = resp.json();
        assert!(
            body["error"].as_str().unwrap_or("").contains("terminal feature states"),
            "expected terminal-state rejection, got: {body}"
        );
    }
}
```

**Where**:

- Delete `terminal_feature_states_are_not_generic_transition_targets` at
  `crates/agileplus-api/src/routes/features.rs:391-399`.
- Add the new test alongside
  `coverage_create_feature_rejects_terminal_states`
  (lines 254-269). A 1-line file edit.

**Pros**:

- 1-line code deletion + ~22-line test addition. Done in <30 min.
- Cites the existing real-transition test at
  `crates/agileplus-api/tests/api_integration/route_coverage.rs:373-378`
  (`coverage_transition_feature_invalid_target_is_400`) as the
  pattern.
- Closes E.2.P1-GENERIC today.

**Cons**:

- None. The production code is already correct.

### Design B — Defense in depth: add the rule to `Feature::transition()`

Promote the rule from the transport layer to the domain layer. Change
the implementation at
`crates/agileplus-domain/src/domain/feature.rs:96-114` so that any
caller of `feature.transition(Validated | Shipped)` receives an Err
unconditionally.

This means changing the local state-machine model to refuse terminal
transitions outside the canonical acceptance use case.

**Pros**:

- Domain layer is the single source of truth. The rule cannot be
  bypassed by any future route.
- The CLI `mvp` helper and the gRPC path both already use
  `feature.transition()`-style APIs indirectly; pushing the rule here
  would unify them.
- Future PRs cannot accidentally re-introduce a parallel terminal-awarding
  transition.

**Cons**:

- Likely breaks the existing test at lines 251-265 of
  `crates/agileplus-domain/src/domain/feature.rs`
  (`full_happy_path_transition_chain`), which walks to
  `Retrospected` via Validated/Shipped. The test would need to be
  rewritten to call the canonical acceptance use case instead, or the
  rule would need to be parameterizable.
- Requires a contract change in the lifecycle state-machine semantics
  _and_ in the use case (`AdvanceFeature::execute` and
  `FeatureAcceptanceCommand`). The 5-7 line proof-of-concept above
  understates the ripple.
- Risk of touching the `traceability-core` upstream re-export
  (`crates/agileplus-domain/src/domain/state_machine.rs:3`). The
  `FeatureState::transition()` method itself comes from upstream.
- ~45-60 min if no surprises, ~2 hours if there are surprises.

### Design C — Governance-contract-as-source-of-truth

Move the rule into the governance contract. A feature is "generic" if
and only if it has no `GovernanceContract` (or its `GovernanceContract`
has no rules). The transition function reads the contract and refuses
terminal targets when the contract is absent / incomplete.

**Pros**:

- Reuses existing primitives (`GovernanceContract`,
  `AtomicAcceptancePort`).
- Ties the rule to the canonical evidence model.

**Cons**:

- Heaviest design. Requires plumbing `&GovernanceContract` (or a port)
  through `Feature::transition()` and `AdvanceFeature::execute()`.
- Changes the semantic of "generic" to mean "no contract bound".
  This is a different concept than the WBS's "generic transport path".
- ~90-120 min.
- Requires tests for the "feature with a bound contract can still
  transition" case vs. the "feature without a contract cannot" case.

### Recommendation

**Design A**. The production code already enforces the rule at the
correct layer (transport); the WBS's exit criterion ("HTTP is a
projection of the application acceptance use case") is substantively
met. The only thing missing is a test that proves it. E.2.P1-GENERIC
is a test-quality gap, not a design gap.

If the operator wants stronger guarantees, **layer Design A first**
(closes E.2.P1-GENERIC, unblocks P1), then separately scope Design B
as an optional follow-up cell. Do not couple them.

---

## 5. Recommendation (single paragraph)

Close E.2.P1-GENERIC with the minimum viable change: delete the
tautology test at `crates/agileplus-api/src/routes/features.rs:391-399`,
and add an HTTP behaviour test next to
`coverage_create_feature_rejects_terminal_states`
(`crates/agileplus-api/tests/api_integration/route_coverage.rs:254-269`)
that POSTs `target_state ∈ {"validated", "shipped"}` to a seeded
`/api/v1/features/{slug}/transition` endpoint and asserts HTTP 400 with
the "terminal feature states" rejection message. Implementation: 1
test-deletion edit + 1 test-addition edit (~25 min, ETA given in the
DOCKET_CLOSE_OUT.md was 60 min, but that estimate assumed Design-B/C
work). No production code changes. No schema migration. No new fields
on `Feature`. The WBS sub-task 3 sixth case becomes a behaviour test
with observed evidence.

---

## 6. Test replacement (concrete patch shape)

The new test should sit next to its sibling
`coverage_create_feature_rejects_terminal_states`
(`crates/agileplus-api/tests/api_integration/route_coverage.rs:254-269`).
The existing positive-and-negative pattern in the file shows the right
shape: the `create_feature_state_test!` macro (lines 223-249) sweeps the
non-terminal states; `coverage_create_feature_rejects_terminal_states`
(254-269) sweeps the two terminal values, asserting HTTP 400 + a
message substring. The new test must mirror that pattern for the
transition endpoint, not the create endpoint.

The existing transition-coverage tests at
`crates/agileplus-api/tests/api_integration/route_coverage.rs:373-414`
cover:

- happy-path Created -> Specified (373-383)
- invalid target string (385-393)
- backward / non-allowed target (395-405)
- unknown slug -> 404 (407-415)

The missing case is "valid terminal target string but disallowed". The
new test closes that case.

Citation template:

```rust
// `validated` and `shipped` are terminal states and must NOT be
// reachable through the generic HTTP transition path. The canonical
// path is the atomic-acceptance use case (`POST /{slug}/accept`). This
// behaviour test mirrors `coverage_create_feature_rejects_terminal_states`
// (above) for the transition route.
#[tokio::test]
async fn coverage_transition_feature_rejects_terminal_states() {
    for terminal in ["validated", "shipped"] {
        let server = setup_test_server().await;
        let resp = server
            .post("/api/v1/features/test-feature/transition")
            .add_header(KEY, TEST_API_KEY)
            .json(&serde_json::json!({ "target_state": terminal }))
            .await;
        resp.assert_status(StatusCode::BAD_REQUEST);
        let body: serde_json::Value = resp.json();
        assert!(
            body["error"].as_str().unwrap_or("").contains("terminal feature states"),
            "expected terminal-state rejection, got: {body}"
        );
    }
}
```

The existing `coverage_transition_feature_invalid_target_is_400` (line 385) exercises `target_state = "sideways"`, so the seed feature does not
need to be in `Implementing`; the message-substring assertion is enough.

The deletion: lines 391-399 of
`crates/agileplus-api/src/routes/features.rs`, the entire
`terminal_feature_states_are_not_generic_transition_targets` test.

---

## 7. Effort estimate for implementation

| Design                                                                          | Effort                                                                | Lines of code touched                                                          | Risk                                                             | Closes E.2.P1-GENERIC |
| ------------------------------------------------------------------------------- | --------------------------------------------------------------------- | ------------------------------------------------------------------------------ | ---------------------------------------------------------------- | --------------------- |
| **A** — replace tautology with HTTP behaviour test                              | **25 min**                                                            | ~25 add, 9 delete                                                              | zero behaviour change                                            | YES                   |
| B — add rule to `Feature::transition()` (domain layer)                          | 45-60 min (no surprises), up to 2 h (if traceability_core needs sync) | ~15 plus 5-7 test updates                                                      | medium; semantic of lifecycle crate changes                      | YES                   |
| C — governance-contract-as-source-of-truth                                      | 90-120 min                                                            | ~50 add, contract API surface                                                  | medium-heavy                                                     | YES                   |
| D — remove `POST /transition` entirely (force `/accept` for every state change) | NOT RECOMMENDED                                                       | would invalidate `route_coverage.rs:373-414`, `atomic_acceptance_http.rs` etc. | very high — breaks the existing non-terminal transition contract | YES (in a sense)      |

**Recommendation**: ship Design A; if operator later wants stronger
guarantees, layer B.

---

## 8. Open questions for the operator

These are the only operator decisions needed; everything else is
mechanical.

1. **Do you accept Design A** (the 25-min test-only fix) or do you
   want Design B/C layering?
2. **Naming**: should the new test be named
   `coverage_transition_feature_rejects_terminal_states` (matching
   the create-route sibling) or
   `route_transition_terminal_states_rejected_http_400` or some other
   name? My recommendation is option 1 for symmetry.
3. **Phase**: are we ready to land this now, or are you parking
   E.2.P1-GENERIC behind other P1 carry-forwards (E.2.P1-422 is a
   similarly small test-quality fix, eta 10 min per t-rex)?
4. **(Optional, only if Design B/C are in scope)** Is "generic" a
   short-lived term we want to keep, or should we introduce a
   `FeatureKind`-style domain enum so the language aligns? My
   recommendation is **no** — every existing test, comment, doc, and
   invariant uses "generic" to mean "transport path", not "feature
   classification". Adding a new domain concept would broaden scope
   without benefit.

---

## Appendix A: citation index (every claim above is anchored)

| Claim                                                              | File:Line                                                                                      |
| ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------- |
| `Feature` struct has no `kind` field                               | `crates/agileplus-domain/src/domain/feature.rs:57-74`                                          |
| `Feature::transition()` allows terminal transitions                | `crates/agileplus-domain/src/domain/feature.rs:96-114`                                         |
| HTTP `/transition` rejects terminal targets in production          | `crates/agileplus-api/src/routes/features.rs:283-287`                                          |
| HTTP `create_feature` rejects terminal initial states              | `crates/agileplus-api/src/routes/features.rs:168-172`                                          |
| HTTP `/transition` happy-path test (template)                      | `crates/agileplus-api/tests/api_integration/route_coverage.rs:373-383`                         |
| HTTP `create_feature_rejects_terminal_states` (template)           | `crates/agileplus-api/tests/api_integration/route_coverage.rs:254-269`                         |
| HTTP `create_feature_state_test!` macro (template)                 | `crates/agileplus-api/tests/api_integration/route_coverage.rs:223-249`                         |
| Tautology test (the gap)                                           | `crates/agileplus-api/src/routes/features.rs:391-399`                                          |
| gRPC "generic" fails closed on validate/ship                       | `crates/agileplus-grpc/src/server/mod.rs:705-714`                                              |
| gRPC behaviour test mirroring the HTTP fix                         | `crates/agileplus-grpc/tests/rpc_dispatch_command.rs:255-268, 294-307`                         |
| CLI "generic" MVP transition fails closed                          | `crates/agileplus-cli/src/commands/mvp.rs:755-769`                                             |
| Canonical atomic-acceptance use case                               | `crates/agileplus-application/src/use_cases/accept_feature.rs:1-17`                            |
| `/accept` HTTP route                                               | `crates/agileplus-api/src/routes/acceptance.rs:48, 132-181`                                    |
| WBS sub-task 3 sixth-case wording                                  | `verification/FORWARD_WBS_2026-10-04.md:73-79`                                                 |
| WBS P2 gRPC parity context                                         | `verification/FORWARD_WBS_2026-10-04.md:84-103`                                                |
| t-rex carry-forward (misframed)                                    | `docs/sessions/20261017-state-forward-docket/e2_e5_p1p2_val_2026-10-18-trex-result.md:170-210` |
| DOCKET entry for E.2.P1-GENERIC                                    | `docs/sessions/20261017-state-forward-docket/DOCKET_CLOSE_OUT.md:55, 119`                      |
| AGENTS.md invariant (top-level)                                    | `~/AGENTS.md` "Recovery Authority" section                                                     |
| Mature contract spec (referenced but not authoritative for design) | `spec/product/mature-contract.v1.json`                                                         |

---

## Appendix B: what t-rex got right vs. wrong

| T-rex claim                                                                | Truth                                                                                                                                           |
| -------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| "There is no production enforcement"                                       | WRONG. Production enforcement exists at HTTP (`features.rs:283-287, 168-172`) and gRPC (`mod.rs:705-714`).                                      |
| "The only test is a tautology"                                             | CORRECT. Only `terminal_feature_states_are_not_generic_transition_targets` exists, and it is a tautology.                                       |
| WBS sub-task 3 sixth-case is uncovered                                     | CORRECT. No behaviour test exists for the HTTP-rejection rule.                                                                                  |
| "Decide whether 'generic' is a property of Feature or governance contract" | MISLEADING. "Generic" is not a property of Feature; it means "the generic transport path" (an established codebase term). The decision is moot. |
| "Implement in `Feature::transition()` and/or `accept_feature.rs`"          | MISLEADING. `accept_feature.rs` is already the canonical path (15 lines). The rule lives at the transport layer.                                |
| "60 min ETA"                                                               | WRONG. The actual work is ~25 min. T-rex sized the design choice, not the gap.                                                                  |
| "Risk: medium"                                                             | WRONG. Implementation is zero behaviour change.                                                                                                 |

The t-rex carry-forward is **directionally correct but operationally
mis-scoped**. The investigation here narrows the gap to a test-quality
fix.

---

## Appendix C: evidence pack

The following evidence is captured to `/tmp/e2_p1_generic_2026-10-18.log`
(the operator's full tee stream) for downstream verification.

- All `rg` searches against the workspace.
- The line-range reads of `feature.rs`, `state_machine.rs`,
  `acceptance.rs`, `accept_feature.rs`, `features.rs`, `mvp.rs`,
  `mod.rs`, `advance_feature.rs`.
- The pattern-match library of "generic" test names.
- The WBS line-range read of `verification/FORWARD_WBS_2026-10-04.md`
  showing the exact wording of P1-3 sub-task 3 sixth case.
- The t-rex result file's E.2.P1-GENERIC section, full read.

The result file at `/tmp/e2_p1_generic_2026-10-18-result.md` (this file)
is the canonical deliverable; the log is the tee stream.
