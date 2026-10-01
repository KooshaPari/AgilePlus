# AgilePlus CI Evidence — 2026-10-01

**Candidate observed:** `f23ee60c794e0f9cefea4b7a8314b0381d94360a`  
**Evidence class:** native GitHub CI observation; not full product acceptance.

## Observed checks

### Positive

- `ci / test`: **SUCCESS**
- TS/JS: **SUCCESS**
- Dependency Review: **SUCCESS**
- SonarCloud quality gate: **SUCCESS**
- Semgrep: **SUCCESS**
- OSV PR scanner: **SUCCESS** (no new alerts in PR diff)

These are supporting evidence. They do not close the runtime evidence plan by themselves.

### Negative / classified

#### Rust job — branch-local formatting failure

The Rust matrix fails `cargo fmt --check` on branch-modified execution/governance surfaces, including:
- API governance route;
- CLI implement path;
- CLI governance differential tests;
- ship command tests;
- Evaluation/Attempt domain records;
- governance evaluator;
- execution port;
- SQLite execution repository.

No semantic compiler/test failure is established by this job because formatting fails first.

**Classification:** branch-local implementation hygiene defect. It is mechanically repairable, but source modification is outside the current non-code-only phase.

The green aggregate `ci / test` therefore must not be interpreted as proof that the Rust matrix is fully green; the jobs cover different gates/scopes.

#### Python job — repository-wide hygiene debt

Ruff reports approximately **180 errors**, dominated by:
- unsorted/unformatted imports;
- unused imports;
- unused `noqa`;
- formatting rewrites.

The log spans broad pre-existing Python surfaces unrelated to the mature semantic branch work.

**Classification:** repository-wide/pre-existing hygiene debt unless a changed-file diff proves a branch-introduced subset.

#### Security job — workflow/tool invocation failure

Gitleaks fails scanning Git with:
`failed to scan Git repository: stderr is not empty`

The workflow invokes a first-parent/log-range command and exits unexpectedly.

**Classification:** CI/workflow infrastructure debt, not evidence of a leaked secret.

Separate security scanners on this candidate report success/no new PR alerts.

#### Cargo Deny — dependency-policy debt

Observed:
- multiple unmaintained crates;
- advisory findings including rustls/webpki-related vulnerabilities;
- bans/licenses failures.

**Classification:** real repository dependency/security debt. It is not waived by the semantic contract and must be addressed before an applicable release/security gate can pass. It is not evidence that the mature semantic specification is wrong.

#### Performance/coverage jobs

Performance and coverage jobs fail/skip independently of the semantic contract.

**Classification:** runtime quality/evidence debt requiring profile-specific investigation.

## Important positive boundary

The aggregate `ci / test` check is green on this candidate. This is the first native evidence in this pass that the branch is not generically test-broken.

It does **not** yet prove:
- evaluator differential semantics;
- worker replacement;
- exact-candidate acceptance;
- interface parity;
- federation;
- audit v2.

Those remain explicit tracks in `RUNTIME_EVIDENCE_PLAN.md`.

## Promotion state

**Not promotable from this evidence.**

Reason:
- required runtime witnesses remain;
- security/dependency policy debt exists;
- lint aggregate is red;
- performance/coverage evidence incomplete.

No semantic criterion is weakened because CI is red.


## Exact-candidate acceptance recovery checkpoint

The runtime recovery branch has advanced through a bounded acceptance-hardening slice:

- SpecRevision replay is idempotent only for identical immutable content.
- changed execution basis creates a parent-linked SpecRevision and explicitly supersedes the prior Assignment;
- migration 028 enforces at most one active Assignment per WorkPackage;
- replacement Attempts remain durable rather than overwriting prior execution history;
- review approval is non-terminal and records an Inconclusive exact-candidate Evaluation;
- unresolved jobs cannot masquerade as exact candidates;
- `validate` requires both governance success and a Satisfied Evaluation bound to a Completed Attempt's exact `git:` candidate for every WorkPackage;
- `ship` rejects zero-work promotion, rechecks source drift, and merges the immutable evaluated commit SHA rather than a mutable branch;
- promotion receipts include the exact accepted candidates;
- automated governance policies without a concrete evaluator fail closed.

Exact rustfmt normalization for this source state was produced by:

- `c52325ae429a619287053ed040c0dcfdc9b889b0` — `style(rust): apply cargo fmt`

The bot-authored commit's workflows completed as `action_required` without executing jobs. This user-authored evidence checkpoint exists to obtain native CI on the same formatted runtime semantics plus this note.

These changes still do **not** provide a first-class independent candidate evaluator capable of legitimately producing a new Satisfied Evaluation. That remains an explicit runtime gap rather than being filled by review approval or governance evidence.
