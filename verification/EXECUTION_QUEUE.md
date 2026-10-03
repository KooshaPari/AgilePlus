# AgilePlus Runtime Execution Queue

**Status:** ordered handoff; source changes not performed in the non-code phase.

## P0 — make branch-native Rust evidence executable

1. Run exact repository formatter on changed Rust surfaces; commit formatting-only diff.
2. Re-run Rust matrix and targeted governance differential tests.
3. Confirm the added partial-evidence case rejects one-of-two CI-backed requirements.
4. Preserve legacy CLI evaluator until differential evidence justifies convergence.

## P1 — execution identity witness

Run durable SpecRevision→Assignment→Attempt→Candidate→Evaluation path, including restart.

Then worker replacement:
- Attempt A expires/fails;
- Attempt B receives new claim;
- A remains durable;
- B produces exact Git candidate;
- Evaluation binds exact candidate.

## P2 — acceptance boundary

Execute zero-criteria, self-grade, review+CI-but-failed-criterion, policy-vs-correctness, stale candidate and merge-error adversarial cases.

## P3 — interface parity

Compare supported CLI/HTTP/MCP canonical IDs/status/error semantics on one fixture.

## P4 — Tracera federation

Run both available and unavailable peer cases; prove AcceptedWork != Product Satisfied.

## P5 — audit v2

Implement only after canonical serialization/covered-field fixture is locked; preserve v1 hashes.

## Repository debt track — do not confuse with branch regression

- ~180 Python Ruff hygiene errors;
- Cargo Deny unmaintained/advisory/bans/license findings;
- gitleaks workflow invocation failure;
- performance/coverage gate failures;
- optional/legacy Go coverage failure.

Security/dependency debt is real release debt even when pre-existing. It must not be waived merely because it predates this PR.
