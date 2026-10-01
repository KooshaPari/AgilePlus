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
