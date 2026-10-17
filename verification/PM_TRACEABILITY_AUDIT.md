# Traceability-core / PM audit

Base: origin/main 00397a59 (KooshaPari/AgilePlus). Local main was clean and three commits ahead; preserved. Work is isolated on pm/work-item-audit-schema.

Feature, WorkPackage, Cycle, Module live in agileplus-domain (`domain/feature.rs`, `domain/work_package/mod.rs`, `domain/cycle/entity.rs`, `domain/module.rs`). traceability-core supplies `requirement.rs`, `artifact.rs` (Test kind/ref), `governance.rs` (Evidence), `tracelink.rs`, `matrix.rs`, and `lifecycle.rs`. No dedicated Test or product Release entity found. ReleaseRequest/ReleaseArgs release claims; ReleaseChannel is governance metadata.

CLI verbs are registered in agileplus-cli/src/main.rs. REST registrations are in agileplus-api/src/router/compose.rs and routes/*.rs. API nests use CredentialStore API-key validation (Bearer, X-API-Key, query key); HTML module/cycle pages are public. Router uses Tower HTTP TraceLayer. Custom middleware/otel.rs exists but is not attached by compose.rs; API binary uses NoOpObservability.

| Journey | Tag | Evidence |
|---|---|---|
| J1 intake/triage/board/cycle | PARTIAL | Source supports persistent queue and cycle transitions; triage only prints and queue lacks a status-transition verb. CLI runtime blocked. |
| J2 spec proposal->archive | PARTIAL | Source supports specify/refinement and governed ship; no first-class proposal/archive CLI verbs. Runtime blocked. |
| J3 requirement->test->evidence->release | PARTIAL | Requirement/link/evidence primitives and ship exist; no dedicated Test/Release entity or release CLI verb. Full runtime chain unverified. |

These tags are provisional source findings. Golden-fixture CLI execution was blocked by the CLI build. No journey is claimed implemented end to end.

Full workspace test build exceeded 15 minutes and failed on Windows disk exhaustion before tests executed. Clean baseline/core retries and targeted new-code check/tests were blocked by Cargo locks and disk pressure. Rust counts: 0 executed / 0 passed / 0 assertion failures, with build failure and blocked attempts recorded separately. Supplementary Python sqlite3 migration smoke passed 6 assertions. Whole-workspace formatting hit Windows command-length error 206; changed files formatted directly, and git diff --check passed.

The complete evolving operator result is C:/Users/koosh/workers/agilenum-RESULT.md. Source/runtime/deployment status remain separate; no deployment or installable release is claimed.

See PM_SCHEMA_FIRST.md for new routes, schema, hash-chain format, and targeted verification commands.
