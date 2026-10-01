# AgilePlus Runtime Evidence Plan

**Status:** execution-ready plan; no execution result is implied  
**Date:** 2026-10-01

## Evidence identity

Every runtime witness binds:
- AP requirement/case IDs;
- exact repository candidate;
- Development/SpecRevision/WP/Assignment/Attempt/Candidate/Evaluation identities as applicable;
- evaluator/tool version;
- environment;
- raw evidence;
- explicit non-vacuous result.

## Track A1 — evaluator convergence

Execute differential tests between legacy CLI semantics and shared evaluator for:
- matching required evidence;
- wrong evidence type;
- one-of-two requirement evidence missing;
- empty contract;
- missing active policy;
- threshold policy;
- manual approval;
- custom evaluator unavailable.

Do not delete/switch the legacy evaluator until equivalence or explicitly accepted semantic differences are evidenced.

## Track A2 — durable execution identities

Prove:
- SpecRevision survives restart;
- Assignment is immutable/superseded explicitly;
- Attempt A remains after replacement Attempt B;
- claims expire/reap and do not transfer unrelated authority;
- exact candidate is Git/tree/artifact identity, not generic job id when exact identity is available;
- Evaluation binds exact candidate/evaluator/spec/criteria.

## Track A3 — acceptance boundary

Execute adversarial cases:
- zero criteria;
- review+CI green while criterion fails;
- policy pass while correctness fails;
- correctness pass while policy denies;
- candidate changes after evaluation;
- worker self-reports success;
- merge conflict/error;
- skipped/diagnostic flags cannot promote.

AcceptedWork requires correctness grade + governance permission.

## Track A4 — audit integrity

Verify:
- historical audit-hash-v1 fixture;
- documented non-covered-field mutation behavior;
- v2 canonical serialization;
- every v2 covered-field mutation breaks verification;
- explicit v1→v2 chain transition;
- archive/redaction semantics.

Never rewrite historical v1 hashes.

## Track A5 — worker replacement

End-to-end witness:
1. accepted Development/SpecRevision/WP/Assignment;
2. Attempt A obtains claim/worktree and produces partial result;
3. A expires/dies;
4. Attempt B receives new authority;
5. A history remains;
6. B produces exact Candidate;
7. independent Evaluation grades candidate;
8. governance permits/denies;
9. accepted completion recorded only when both pass.

## Track A6 — interface parity

For supported CLI/HTTP/MCP surfaces, query/mutate the same fixture and compare:
- canonical IDs;
- revisions;
- evaluation status;
- authority/error semantics;
- no greener state on one transport.

## Track A7 — Tracera federation

With and without Tracera available:
- export exact work/evaluation receipt;
- preserve standalone AgilePlus truth;
- Tracera independently admits/rejects applicability;
- Tracera rejection does not rewrite AgilePlus AcceptedWork;
- AgilePlus Done/AcceptedWork does not directly set Tracera Satisfied.

## Track A8 — persistence/recovery/profile failures

Execute restart/migration/backup/config-precedence/plugin-failure cases from the semantic verification catalogue.

Optional Plane/NATS/Neo4j/MinIO/P2P failure must not redefine core local work truth.

## CI debt classification

Keep three classes separate:
1. branch-introduced semantic/build regression;
2. pre-existing repository/workflow debt;
3. optional-profile/infrastructure unavailable.

A red workflow is not automatically a branch defect; a green unrelated workflow is not acceptance evidence.

## Promotion rule

PR #1090 remains draft until the agreed runtime witness subset passes with exact evidence. Implementation may not weaken criteria in the same change to manufacture acceptance.
