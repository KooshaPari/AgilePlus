# Forward delta — 2026-10-04 promotion/recovery

This file records implemented deltas that supersede earlier WBS wording.

## Completed since the prior WBS

- gRPC validate now routes through the canonical atomic acceptance use case.
- gRPC acceptance requires bearer auth, project scope and durable request_id.
- gRPC rejects authority/grading override arguments.
- Stale governance and divergent request replay have adversarial witnesses.
- Restart reconciliation expires interrupted Attempts without deleting history.
- Replacement Attempts proceed under the same durable Assignment.
- Promotion exact-candidate preflight is now application-owned in
  `use_cases::promotion`.
- CLI ship consumes that canonical preflight.
- Immediate pre-merge source re-resolution is application-owned.
- Promotion drift now fails before merge/state mutation.
- Dry-run preflights exact candidates without merge/cleanup/artifact/state side effects.

## Promotion boundary still open

The merge itself is an external VCS side effect and is not transactionally
coupled to SQLite. The remaining promotion work is therefore a resumable saga,
not an attempt to pretend Git+DB can be one ACID transaction.

Next design must durably distinguish:

1. planned promotion;
2. each accepted candidate merge attempt;
3. resulting merge identity;
4. terminal Shipped persistence;
5. audit/event receipt;
6. cleanup completion.

A crash after merge but before DB finalization must be recoverable without
blindly merging the candidate again.

## Immediate native gate

The exact current candidate still requires authoritative native execution of:

- canonical atomic acceptance tests;
- real-SQLite HTTP acceptance tests;
- gRPC acceptance tests;
- interrupted Attempt replacement/restart tests;
- promotion preflight/drift tests.

Test existence is not test execution.
