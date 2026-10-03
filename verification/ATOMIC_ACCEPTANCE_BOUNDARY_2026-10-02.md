# Atomic acceptance boundary — 2026-10-02

Predecessor: `ba8e6820e92a6af43ed9b6fe6276acb17e3865f4`.
Scope: AgilePlus terminal work acceptance. PR #1090 stays draft.

## Defects addressed

The previous `accept_feature` accepted caller-created `GovernanceReceipt` booleans,
read candidate records before writing, and issued separate WP/Feature/audit writes.
The CLI emitted its state-transition event afterward. A storage error could leave
Done/Validated without all required records, and a caller could supply its own pass.

## New boundary

`AtomicAcceptancePort` is separate from generic `StoragePort`. The SQLite adapter
starts an IMMEDIATE transaction before reading current feature state, governance,
feature-scoped evidence, policies/metrics, assignments, criterion receipts and
attempts. It uses the shared pure governance and candidate validators, not a weaker
SQL or transport-specific evaluator. Every work package is preflighted before the
first mutation, and the transaction remains held throughout the writes.

The transaction commits together:

- Review-to-Done work-package changes;
- feature Implementing-to-Validated;
- per-WP and feature audit entries, preserving audit-hash-v1 semantics;
- the hash-chained transition event, containing candidate/evaluation and audit IDs;
- a durable, immutable request/receipt row.

There is no async suspension, network call, nested adapter lock or CLI artifact
write inside the transaction. An error propagates before commit and rolls back.
Repeated identical request IDs return the original historical receipt; reuse of
an ID for a different command conflicts. A replay is not a new applicability check.

Empty governance placeholders and unrecognized evidence-type suffixes now fail
closed. They are not empty-success or wildcard evidence specifications.

## Callers

CLI `validate` retains diagnostic reporting and flags, but passes no successful
boolean to the mutation path. The atomic adapter reevaluates governance itself.
Diagnostic flags cannot call terminal acceptance. Report/artifact delivery occurs
after the transaction and its failure cannot split canonical state from audit/event.

Authenticated `POST /api/v1/features/{slug}/accept` accepts a request ID and optional
expected governance version. The body cannot supply a pass, authoritative flag,
or actor override. It is deliberately distinct from governance-only `/validate`.
The production HTTP bootstrap binds acceptance to its existing SQLite instance.
Embedders without this capability return 501, not success.

The actor is the authenticated transport's capability label, not yet individual
multi-user attribution. Existing API-key authorization remains the auth boundary.
Generic HTTP/gRPC terminal state setters must stay fail-closed. gRPC's canonical
acceptance wiring is not claimed by this patch; shipping remains a separate workflow.

## Verification design

`agileplus-sqlite/tests/atomic_acceptance.rs`: 10 native tests, including injected
failure at five write boundaries, complete rollback, retry, idempotency conflicts,
current evidence/candidate/assignment drift, malformed governance, receipt
immutability, file reopen and concurrent same-request behavior.

`agileplus-api/tests/atomic_acceptance_http.rs`: 5 mounted HTTP tests covering actual
SQLite success/replay, auth rejection, forbidden caller overrides, storage failure
redaction/rollback and unsupported-embedding 501.

The 15 existing migration regression cases are retained in a focused tests module.
The 026 rollback test now locates its migration instead of assuming which later
migration happens to be newest. Existing migration SQL bodies are unchanged.

Native results are pending until the exact candidate's workflow executes.
`Recovery Acceptance Witnesses` runs tests independently of the formatting step,
uses only a standard public runner, has read-only permissions, and uploads no
billable artifacts or uses any paid/private-runner fallback.

## Limits and forward gates

- Recorded Git candidate reference equality is checked here. Real Git-object and
  source-ref resolution remains the shipping adapter's responsibility.
- Evaluator identity separation is not a cryptographic trust/attestation system.
- Existing v1 audit coverage is unchanged; this is not audit-hash-v2 implementation.
- External Git merges cannot be made atomic by this SQLite acceptance transaction.
  Promotion needs its own recoverable external-side-effect protocol.
- CLI/HTTP/gRPC whole-product parity, live worker replacement, Tracera federation,
  user-facing recovery and external validation are still distinct gates.
