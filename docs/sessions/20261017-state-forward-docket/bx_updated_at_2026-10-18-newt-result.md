# B.x follow-up: find source of non-RFC3339 updated_at (newt, 2026-10-18T09:51Z)

**Verdict**: PARTIAL — source NOT in Rust; the 5 corrupt rows were carried in via a **binary file replacement** in merge commit `ea5a517a` at 2026-08-07 10:43:57 UTC (= 03:43:57 -0700), which overwrote `.agileplus/agileplus.db` with a local copy. All in-tree Rust UPDATE bindings use `chrono::Utc::now().to_rfc3339()` (valid RFC3339).

**Subagent session**: newt single-task dispatch, this session
**Wall-clock**: ~210s (under 15min cap)
**Search commands run**: 14+ `rg`/`sed` invocations (full counts tee'd to /tmp/bx_updated_at_2026-10-18.log)

## Source candidate(s) found

| Candidate                                                               | Format produced                                                           | Verdict                                                     |
| ----------------------------------------------------------------------- | ------------------------------------------------------------------------- | ----------------------------------------------------------- |
| `crates/agileplus-sqlite/src/repository/modules/crud.rs:75-78`          | `now = chrono::Utc::now().to_rfc3339()` → `2026-08-07T10:43:15.123+00:00` | RFC3339 OK — not culprit                                    |
| `crates/agileplus-sqlite/src/repository/users.rs:101` & `:116`          | `now = chrono::Utc::now().to_rfc3339()`                                   | RFC3339 OK — not culprit                                    |
| `crates/agileplus-sqlite/src/repository/epics.rs:133-136`               | `now = chrono::Utc::now().to_rfc3339()`                                   | RFC3339 OK — not culprit                                    |
| `crates/agileplus-sqlite/src/repository/backlog.rs:229`,`:243`          | `now.to_rfc3339()` (via `params!`)                                        | RFC3339 OK — not culprit                                    |
| `crates/agileplus-sqlite/src/repository/stories.rs:146`                 | `now.to_rfc3339()`                                                        | RFC3339 OK — not culprit                                    |
| `crates/agileplus-sqlite/src/repository/cycles/mod.rs:64`               | `now.to_rfc3339()`                                                        | RFC3339 OK — not culprit                                    |
| `crates/agileplus-sqlite/src/repository/features.rs:158`                | `now.to_rfc3339()`                                                        | RFC3339 OK — not culprit                                    |
| `crates/agileplus-sqlite/src/repository/work_packages.rs:188-191`       | `now = chrono::Utc::now().to_rfc3339()`                                   | RFC3339 OK — not culprit                                    |
| `crates/agileplus-sqlite/src/repository/acceptance/mod.rs:170` & `:191` | `now.to_rfc3339()`                                                        | RFC3339 OK — not culprit                                    |
| `crates/agileplus-sqlite/src/repository/promotion.rs:187`               | `now.to_rfc3339()`                                                        | RFC3339 OK — not culprit                                    |
| `desktop/src-tauri/src/work_packages.rs:96`, `:106`, `:112`             | `now = chrono::Utc::now().to_rfc3339()`                                   | RFC3339 OK — not culprit                                    |
| `crates/agileplus-sqlite/src/repository/cycles/mod.rs:33-34`            | `strftime('%Y-%m-%dT%H:%M:%SZ', 'now')`                                   | RFC3339 OK — not culprit                                    |
| `crates/agileplus-dashboard/src/routes/dashboard.rs:54` etc.            | `Utc::now().format("%Y-%m-%d %H:%M:%S UTC")` → **non-RFC3339**            | **Template/display only**, not bound to `updated_at` column |
| `crates/agileplus-sqlite/src/rebuild.rs:155-158`                        | `meta.updated_at.parse::<chrono::DateTime<chrono::Utc>>()`                | Robust — would reject `YYYY-MM-DD HH:MM:SS` format          |
| Negative test markers (`mappers.rs:253`, `cycles/mod.rs:575`)           | literal `'not-a-timestamp'`                                               | Tests — not source                                          |

## Most likely culprit: **binary file replacement in commit `ea5a517a`**

```
commit ea5a517a2f6aa832b1bbe98d85c2b58f3ca97b7c
Author: KooshaPari <koosha@example.com>
Date:   Fri Aug 7 03:43:57 2026 -0700   (= 2026-08-07 10:43:57 UTC)

    chore: resolve dashboard Cargo.toml merge conflict (keep agileplus-api dep)

 .agileplus/agileplus.db | Bin 532480 -> 532480 bytes
 1 file changed, 0 insertions(+), 0 deletions(-)
```

- **Timestamp matches exactly**: 2026-08-07 10:43:57 UTC matches the corruption window (~10:43).
- **Only file changed**: the SQLite database itself (binary, no source touched).
- **No Rust source line wrote those 5 rows at that moment** — the corruption was imported wholesale from the local working copy of `.agileplus/agileplus.db` into the merge resolution.

The local working copy of the DB was last written by whatever `agileplus` CLI / dev session ran between 2026-08-06 (last clean state at `3bfd04cb`) and 2026-08-07 10:43 (the corrupt merge commit). That dev session used the source tree as of those two commits — which already use `to_rfc3339()` everywhere. So either:

1. **An older binary was running** (compiled from a pre-`to_rfc3339()` version of the code) — the SQL UPDATE in that binary used `Utc::now()` raw DateTime<Utc> which rusqlite serializes as `YYYY-MM-DD HH:MM:SS.fff+00:00` (the documented rusqlite default).
2. **A direct sqlite3 / dump import** wrote rows in `YYYY-MM-DD HH:MM:SS` format into the local `.agileplus/agileplus.db`, then the merge conflict resolution carried those 5 rows into the committed binary.

**There is no current Rust source line that produces non-RFC3339 `updated_at`.** The fix (per task instruction: do not apply) would be:

- The actual binding code is already correct (`to_rfc3339()` everywhere in `crates/agileplus-sqlite/src/repository/*` and `desktop/src-tauri/src/work_packages.rs`).
- The carry-forward hazard is the **committed `.agileplus/agileplus.db`** itself. Recommend:
    - Add `.agileplus/*.db` to `.gitignore` (currently committed — bad).
    - Or: regenerate from migrations via `cargo run -p agileplus-sqlite --bin seed_db` and never commit a populated db.
    - Or: if db commits are needed for review, run a hook that `cargo run -p agileplus-sqlite --bin validate_timestamps -- .agileplus/agileplus.db` and rejects the commit if any non-RFC3339 row is found.

## Recommended fix (do not apply, just describe)

**No Rust source line to change.** The corruption is already gone (gecko dispatch 2026-10-18T09:46:39Z UPDATE'd the 5 rows). The durable fix is gitignore the dev DB:

```gitignore
# .gitignore addition
.agileplus/*.db
.agileplus/*.db-*
```

…and re-seed via `cargo run -p agileplus-sqlite --bin seed_db -- .agileplus/agileplus.db` on first checkout.

## Carry-forward cell movement

B.x follow-up: B.x (source-finding for non-RFC3339 `updated_at`) → **CLOSED-PARTIAL** (no source-line culprit; root cause is committed binary DB, not Rust).

Move next follow-up into: **B.y (`.gitignore` the `.agileplus/*.db` binary + pre-commit hook to reject non-RFC3339 rows in committed DBs)** — that is the durable fix the original B.x was trying to find.

## Evidence

(last 30 lines of /tmp/bx_updated_at_2026-10-18.log — full log tee'd to /tmp/bx_updated_at_2026-10-18.log)

```
=== ea5a517a commit details ===
commit ea5a517a2f6aa832b1bbe98d85c2b58f3ca97b7c
Author: KooshaPari <koosha@example.com>
Date:   Fri Aug 7 03:43:57 2026 -0700

    chore: resolve dashboard Cargo.toml merge conflict (keep agileplus-api dep)

 .agileplus/agileplus.db | Bin 532480 -> 532480 bytes
 1 file changed, 0 insertions(+), 0 deletions(-)

=== prior commits to .agileplus/agileplus.db ===
ea5a517a chore: resolve dashboard Cargo.toml merge conflict (keep agileplus-api dep)
3bfd04cb feat(dashboard): wire Plane.so settings, agent sessions, and absorbed frontend attempts (#927)
c5db4e11 wip: auto-commit daemon 2026-07-19T11:45:19Z
dc5c4006 fix: correct Vale action syntax and remove accidentally committed .agileplus/ directory (#866)

=== Summary of findings ===

ALL repository UPDATE bindings use to_rfc3339() — none produce non-RFC3339 format

Most likely source: ea5a517a commit at 2026-08-07 10:43:57 UTC
(= 03:43:57 -0700) modified ONLY .agileplus/agileplus.db (binary)

5 corrupt rows were carried in via the binary file replacement,
NOT written by any Rust source-line binding at runtime.

=== git log 2026-08-07 around 10:43 ===
e4ca91ea airlock: absorb AgilePlus-recovery-20260714 into .archive/ (2026-08-08) (#947)
7b1b3894 wip: airlock snapshot wip/20260808T0235-18c9b3e891092eb8 (#942)
658a0bcb ci: repair coverage action and Go module paths (#941)
2514d22d ci(agileplus): update coverage workflow for v4 actions + correct paths
acf7410d fix(ci): point gitleaks at tracked config (#940)
ea5a517a chore: resolve dashboard Cargo.toml merge conflict (keep agileplus-api dep)
c449f806 feat(dashboard): wire seed_bridge → agileplus-api HTTP calls (#939)
18114a9d fix(ci): restore AgilePlus quality gates (#937)
d4850939 chore: resolve triple merge conflict in dashboard Cargo.toml (keep agileplus-api dep)
```

## Blockers (if any)

None for diagnosis. Cannot fully prove the **upstream writer** of the 5 corrupt rows (was it an old binary, a dump import, or a manual edit?) because the original `.agileplus/agileplus.db` pre-corruption is not recoverable — `3bfd04cb` was the last clean DB state, and the corruption existed at `ea5a517a`. The pre-corruption local copy would only exist in the operator's local git stash or working tree from 2026-08-07 ~10:43, which is outside this subagent's scope.

The plausible answer remains: the pre-`to_rfc3339()` source tree, or an external dump tool. Either way: no current Rust line needs fixing; the durable fix is gitignore + hook.
