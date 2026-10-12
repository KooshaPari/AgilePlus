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

## Promotion boundary closed in source — 2026-10-08

The merge itself is an external VCS side effect and is not transactionally
coupled to SQLite. The remaining promotion work is therefore a resumable saga,
not an attempt to pretend Git+DB can be one ACID transaction.

The implemented saga persists planned promotion, immutable prepared merge identities,
compare-and-swap publication, per-step confirmation, atomic Shipped/audit/event
receipt finalization and post-commit cleanup status. Eight real Git/file SQLite
recovery witnesses and 24 CLI ship witnesses passed. Receipt replay resumes
without blindly remerging after a lost confirmation or late SQL failure.
See [current recovery evidence](PROMOTION_RECOVERY_2026-10-08.md).

The active operator dashboard also passed a real Chromium/Caddy/Axum/file SQLite
acceptance-and-restart journey in CI. Public production still displays the older
dashboard; no production deployment or desktop/tailnet execution is claimed.

## Immediate native gate

The exact current candidate still requires authoritative native execution of:

- canonical atomic acceptance tests;
- real-SQLite HTTP acceptance tests;
- gRPC acceptance tests;
- interrupted Attempt replacement/restart tests;
- promotion preflight/drift tests.

Test existence is not test execution.

## Current release blockers — 2026-10-08

Scoped Vercel deployment inspection returns 403 and no native desktop/tailnet
execution surface has been established. Linux desktop dependency advisories and
unpublished registry dependencies remain release blockers. Full workspace tests
are locally blocked on missing GLib development prerequisites. Focused changed
Rust package Clippy passed; full frontend typecheck, nine operator component
tests, build and accessibility lint passed. This source progress is separate
from remotely usable release progress.
