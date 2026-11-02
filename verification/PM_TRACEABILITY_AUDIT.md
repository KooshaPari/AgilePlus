# Traceability-core / PM audit

Audit base: origin/main 4bd2971b in KooshaPari/AgilePlus. Initial main faa8b301 was ahead 3 / behind 2 with two untracked JOURNEY logs; preserved. Existing branch pm/work-item-audit-schema started from default-head ancestor 00397a59; current origin/main was merged normally. No history rewrite or cleanup performed.

## Domain and transport inventory

Feature, WorkPackage, Cycle and Module live in agileplus-domain/src/domain/{feature.rs,work_package/mod.rs,cycle/entity.rs,module.rs}. WorkPackage transitions are in work_package/mod.rs; CycleState is in cycle/state.rs. traceability-core supplies requirement.rs (Requirement/RequirementStatus), artifact.rs (generic Test kind/ref), governance.rs (Evidence), tracelink.rs, matrix.rs and lifecycle.rs (FeatureState). No dedicated Test or product Release aggregate was found. ReleaseRequest/ReleaseArgs release claims; ReleaseChannel is governance metadata. AgilePlus AcceptedWork remains distinct from Tracera satisfaction.

CLI registration is agileplus-cli/src/main.rs. cargo run -p agileplus-cli --offline -- --help succeeded; recursive help executed 74 root/subcommand help calls. Product verbs: cycle/list/evidence/specify/queue/module/research/plan/implement/evaluate/validate/ship/retrospective/triage/rubric/dag/okf/mvp/dashboard/cockpit.

REST registration is agileplus-api/src/router/compose.rs and routes/*.rs. Protected nests cover features, work-packages, governance/acceptance/audit, modules, cycles, branches, worktrees, events/stream, projects, epics, stories, users and new work-items. CredentialStore API-key validation accepts Bearer, X-API-Key and query keys. Public health/info and HTML module/cycle pages bypass that nest. TraceLayer and configured CORS are attached; custom middleware/otel.rs is not attached, and main.rs uses NoOpObservability.

## Golden fixtures

Executed against a throwaway JOURNEY-fixture Git repository under this worktree, with no remote. Project slug uses lowercase journey-project as required by validation; all titles/artifacts use JOURNEY prefixes. Syntax mistakes were corrected before evaluating capability.

| Journey | Tag | Command and outcome |
|---|---|---|
| J1 intake/triage/board/cycle | PARTIAL | queue add JOURNEY-intake succeeds; triage JOURNEY-intake prints success but queue list still reports new. cycle create/add and transitions Active/Review succeed; Shipped refuses an unvalidated feature. |
| J2 spec proposal->archive | PARTIAL | specify --feature JOURNEY-feature --from-file JOURNEY-spec.md creates the artifact and Specified state. proposal/archive --help are unrecognized; ship --dry-run rejects Specified because Validated is required. |
| J3 requirement->test->evidence->release | PARTIAL | mvp project/epic/story create succeeds with FR-JOURNEY-001; mvp wp create --story 1 fails a foreign-key constraint. requirement/test/release --help are unrecognized; evidence attach fails without a feature WP. Full chain not demonstrated. |

## Validation

- Full workspace test build failed before test execution: agileplus-agent-service requires missing protoc. Earlier shared-target attempts were interrupted after cache/linker stalls. No workspace pass claimed.
- Core fallback (domain, SQLite, traceability-core): 1843 passed, 17 failed, 1 ignored. Thirteen failures concern Windows home/config/credential lookup; four exposed migration expectations affected by this change and were corrected.
- Final complete SQLite rerun after those corrections: 616 passed, 0 failed.
- cargo check for domain/SQLite/API passed.
- New domain hash-field witness passed in the core run; targeted SQLite work-item witnesses: 5 passed; real file-SQLite authenticated HTTP witness: 1 passed.
- Changed Rust files pass rustfmt checks and git diff --check.

Migration 032 rollback is deliberately refused before changing schema or its marker, preserving append-only history. Audit reads reject a missing creation event. Transitions reuse the verified chain head. HTTP 201 assertion uses the supported axum-test API.

Raw logs, complete route/type inventories, counters and blockers are recorded in C:/Users/koosh/workers/agilenum-RESULT.md and JOURNEY-*.log in this worktree. PR: https://github.com/KooshaPari/AgilePlus/pull/1144 (non-draft). No deployment or installable release is claimed.
