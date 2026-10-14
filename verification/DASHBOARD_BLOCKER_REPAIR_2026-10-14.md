# Dashboard blocker repair candidate — 2026-10-14

Candidate extends PR1121 at `1bf61655b0161213e5deaee5bde55b84b9f32cc8`.
The original PR branch and draft checkpoint are preserved. Instinct reviews the
candidate first and coordinates the single final integration path; no merge or
deployment is authorized by this evidence.

## Counterexamples and repair

Three regressions failed against the original head using the real native binary
and disposable loopback Git/SQLite fixtures: invented evidence/report entries on
a real feature page, GET consulting an unselected database override, and a static
startup projection claiming an ongoing live connection after the database went
away. A fourth regression exposed unbound process-CWD evidence through direct
artifact/gallery URLs and failed before the additional provenance guard.

The candidate uses request-scoped source metadata to mark seeded demo fixtures. Only that mode may
render generated events, artifact/media entries and coverage illustrations.
Actual project snapshots have no canonical evidence/evaluation receipt query;
their artifact/gallery URLs therefore return 503 `evidence_source_unavailable`
rather than reading arbitrary process-CWD files. This is an explicit capability
limitation, not a claim that no evidence exists. Canonical evidence projection
remains later work; no missing records are fabricated to fill this gap.

Epics/stories resolve only the selected project's `.agileplus/agileplus.db`, use
SQLITE_OPEN_READ_ONLY and query_only, and read within one transaction. Environment
and CWD overrides cannot replace that source. Unavailable sources/schema return
503; GET creates no database. Data and linkage assertions in the legacy tests
remain; unsafe fallback/write expectations are replaced with selected-source,
unchanged-file and unavailable-source assertions. Synthetic artifact tests now
explicitly choose demo mode, keeping their original content/escaping checks.

The main work-state projection remains a startup snapshot. Its mode is now
`project_database_snapshot`, `connected` is false because its load connection is
closed, and its banner displays `loaded_at` and the reload action. It does not
claim ongoing connectivity or current data. No live refresh daemon is added.

The source metadata is installed as an Axum extension, preserving exhaustive
construction of the existing public `DashboardStore`. The public zero-argument
`epics_stories_json` facade also remains callable and returns unavailable without
source context. Configured routes select the explicit source; direct illustrative
fixture handlers require the `demo` module. Two downstream-style API regressions
failed before this compatibility repair and pass afterward.

## Executed acceptance

All commands ran in the isolated candidate after sourcing
`/workspace/.onboarding/env.sh`, with two Cargo jobs and dev/test debug disabled.

| Command                                                                         | Actual result                     |
| ------------------------------------------------------------------------------- | --------------------------------- |
| `cargo test --locked -p agileplus-dashboard --no-fail-fast -- --test-threads=1` | 520 passed, zero failures/ignored |
| `cargo clippy --locked -p agileplus-dashboard --all-targets -- -D warnings`     | Exit 0                            |
| `cargo fmt --all -- --check` and `git diff --check`                             | Exit 0                            |

The four new `source_truth_regressions` execute real native HTTP/HTML requests,
including selected requirement references and no fabricated artifacts. They are
not browser rendering/a11y witnesses. Demo fixture tests retain their artifact
assertions; snapshot tests require unavailable unbound evidence.

No dependency, credential, authority or deployment change is made. Live refresh,
canonical evidence receipts, worker deduplication and product-to-task applicability
remain unproven. Existing PR1122/1123 frontend/history gate repairs remain separate
owned work. This candidate does not establish mature product readiness.
