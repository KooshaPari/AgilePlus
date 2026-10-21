# B.6 / B.7 Storage Hygiene smoke (gecko, 2026-10-18T09:46:39Z)

**Verdict**: PASS
**Subagent session**: gecko / swarm subagent dispatched by shrimp @ 2026-10-18T09:42:18Z
**Wall-clock**: 261 s (4 m 21 s, under 10 m cap)
**Database path**: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.agileplus/agileplus.db`
**Database size**: 671744 bytes (before) → 671744 bytes (after; SQLite reused pages)
**Schema tables**: 40 tables identified (features, work_packages, governance_contracts, audit_log, evidence, \_migrations, plus event/cycle/epic/backlog/snapshot/registry/metrics tables — see evidence)
**Corrupt row identified**:

- `features` row `id=1` column `updated_at` (column index 7 of the SELECT in `crates/agileplus-sqlite/src/repository/features.rs:207`). Stored value: `"2026-08-07 10:43:55"` — `SQLite TEXT` form, NOT RFC3339 / `chrono::DateTime<Utc>` parseable. Parser at `features.rs:76-87` wraps the chrono error as `FromSqlConversionFailure(7, Type::Text, ...)` → "Conversion error from type Text at index: 7, premature end of input".
- `backlog_items` rows `id=1..4`, column `updated_at` (column index 10 of the SELECT in `crates/agileplus-sqlite/src/repository/backlog.rs:170`). Same root cause: stored values `"2026-08-07 10:43:52"` (SQLite TEXT form, not RFC3339). Parser at `backlog.rs:105` produces the same `Conversion error from type Text at index: 10, premature end of input` error when `agileplus queue list` is run.

**Fix applied**: YES — both roots repaired.

1. `UPDATE features SET updated_at = created_at WHERE id = 1 AND updated_at NOT GLOB '*T*';`
2. `UPDATE backlog_items SET updated_at = created_at WHERE updated_at NOT LIKE '%T%' OR length(updated_at) < 19;`

Both rows now match their `created_at` (which is itself RFC3339 — `2026-08-07T08:48:11.807225+00:00`). Dev data, no behavioural meaning lost. Backups retained at `.agileplus/agileplus.db.bak-pre-fix-1792316706` (1:1 byte copy pre-fix).

**Re-test result**:

- `./target/debug/agileplus list` exit 0 — returns `1 | portage-fork-clean-followups | validated | # Portage Fork-Clean Follow-Ups`. Error gone.
- `./target/debug/agileplus list --state validated` exit 0 — same row.
- `./target/debug/agileplus queue list` exit 0 — 4 backlog items rendered. Previously this command also failed with `Conversion error from type Text at index: 10, premature end of input`; error now gone.
- `sqlite3 PRAGMA integrity_check` → `ok`.
- Broader sweep: 0 remaining non-RFC3339 `updated_at`/`created_at` rows in features, work_packages, audit_log, evidence, spec_revisions, governance_contracts, cycles, attempts, snapshots, metrics, feature_acceptance_receipts. Backlog was the last category.

**Cell movement recommendation**:

- B.6 Storage Hygiene: NOT_RUN → **HEALTHY**
- B.7 Storage Hygiene: NOT_RUN → **HEALTHY**
- The two are tightly coupled here (same root cause, same row mapper shape, same SQLite TEXT-format pitfall), so I am moving them together.

**Root cause (carry-forward note for whoever owns B.x ingress)**:
There is a code path somewhere in the import / seed / migration story that writes `updated_at` as SQLite's default `TEXT` format ("YYYY-MM-DD HH:MM:SS") instead of RFC3339 ("YYYY-MM-DDTHH:MM:SS.nnnnnn+00:00"). All four `backlog_items` rows + the single `features` row were originally seeded at 2026-08-07 ~08:48 (RFC3339) and then updated at ~10:43 (BAD format) — strongly suggests an `UPDATE` path, not the initial seed. Recommend B.x follow-up: grep the codebase for any `UPDATE ... SET updated_at = ?` that binds a non-rfc3339 string (e.g. `CURRENT_TIMESTAMP` or `strftime('%Y-%m-%d %H:%M:%S', 'now')`).

**Evidence (last relevant output, captured by tee to `/tmp/b67_storage_2026-10-18.log`)**:

```
=== schema (features) ===
CREATE TABLE features (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    slug         TEXT    UNIQUE NOT NULL,
    friendly_name TEXT   NOT NULL,
    state        TEXT    NOT NULL CHECK(state IN (
                     'created','specified','researched','planned',
                     'implementing','validated','shipped','retrospected')),
    spec_hash    BLOB    NOT NULL,
    target_branch TEXT   NOT NULL DEFAULT 'main',
    created_at   TEXT    NOT NULL,
    updated_at   TEXT    NOT NULL
, module_id INTEGER REFERENCES modules(id) ON DELETE SET NULL, labels TEXT NOT NULL DEFAULT '[]');

=== source (crates/agileplus-sqlite/src/repository/features.rs:36-87) ===
fn row_to_feature(row: &Row<'_>) -> rusqlite::Result<Feature> {
    let id: i64 = row.get(0)?;
    ...
    let created_at_str: String = row.get(6)?;
    let updated_at_str: String = row.get(7)?;          // <-- index 7
    ...
    let updated_at = updated_at_str
        .parse::<chrono::DateTime<chrono::Utc>>()      // <-- fails on SQLite TEXT form
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                7,
                rusqlite::types::Type::Text,
                ...
=== before ===
$ ./target/debug/agileplus list
Error: listing features: Storage error: Conversion error from type Text at index: 7, premature end of input

$ sqlite3 .agileplus/agileplus.db "SELECT id, slug, created_at, updated_at FROM features;"
1|portage-fork-clean-followups|2026-08-07T08:48:11.807225+00:00|2026-08-07 10:43:55

$ ./target/debug/agileplus queue list
Error: Storage error: Conversion error from type Text at index: 10, premature end of input

$ sqlite3 .agileplus/agileplus.db "SELECT id, title, created_at, updated_at FROM backlog_items;"
1|Add NIAH 32k oracle contract|2026-08-07T08:48:16.636785+00:00|2026-08-07 10:43:52
2|Add CI workflow entry for NIAH gate on fork-clean|2026-08-07T08:48:22.857834+00:00|2026-08-07 10:43:52
3|Validate qwen35-policy oracle gate|2026-08-07T08:48:22.876132+00:00|2026-08-07 10:43:52
4|Address ruff + pytest collection findings|2026-08-07T08:48:22.894330+00:00|2026-08-07 10:43:52

=== after fix ===
$ sqlite3 .agileplus/agileplus.db "SELECT id, slug, created_at, updated_at FROM features;"
1|portage-fork-clean-followups|2026-08-07T08:48:11.807225+00:00|2026-08-07T08:48:11.807225+00:00

$ sqlite3 .agileplus/agileplus.db "SELECT id, title, created_at, updated_at FROM backlog_items;"
1|Add NIAH 32k oracle contract|2026-08-07T08:48:16.636785+00:00|2026-08-07T08:48:16.636785+00:00
2|Add CI workflow entry for NIAH gate on fork-clean|2026-08-07T08:48:22.857834+00:00|2026-08-07T08:48:22.857834+00:00
3|Validate qwen35-policy oracle gate|2026-08-07T08:48:22.876132+00:00|2026-08-07T08:48:22.876132+00:00
4|Address ruff + pytest collection findings|2026-08-07T08:48:22.894330+00:00|2026-08-07T08:48:22.894330+00:00

$ ./target/debug/agileplus list
ID        SLUG                                  STATE           TITLE
----------------------------------------------------------------------------------------------------
1         portage-fork-clean-followups          validated       # Portage Fork-Clean Follow-Ups

$ ./target/debug/agileplus queue list
#   1 [feature] Add NIAH 32k oracle contract (high, done)
  Oracle: build_prompt(32768) yields 32768-token prompt. Verifier: prompt_tokens==32768. ...
  source: cli
  feature: portage-fork-clean-followups
  tags: portage, fork-clean, niah
#   2 [feature] Add CI workflow entry for NIAH gate on fork-clean (high, done)
  ...

$ sqlite3 .agileplus/agileplus.db "PRAGMA integrity_check;"
ok
```

**Blockers**: none.

**Artifacts on disk**:

- This file: `/tmp/b67_storage_2026-10-18-result.md`
- Full tee log: `/tmp/b67_storage_2026-10-18.log` (29635 bytes, 692 lines)
- DB pre-fix backup: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.agileplus/agileplus.db.bak-pre-fix-1792316706`
- DB post-fix: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.agileplus/agileplus.db` (671744 bytes; -shm/-wal also present and consistent)
