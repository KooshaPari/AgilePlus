---
session: wave-12-salvage-4 + wave-13-desktop
date: 2026-09-24 to 2026-09-27
agent: jcode
---

# Coverage Campaign: salvage 4 and wave 13

## Goal

Raise behavior-level test coverage in AgilePlus toward 85%, exercising real public
command paths rather than test doubles, with no production-code changes.

## What landed

### Salvage 4 — `ca6fb610` (pushed to `origin/main`)

Closed the "real public path" gap flagged after salvage 3. `run_review_loop` was
covered directly, but the production caller `run_implement` (dispatched from
`main.rs:220` via `Commands::Implement`) had only helper-level in-file tests. No
test had ever called `run_implement` itself.

- `crates/agileplus-cli/tests/support/implement.rs` (420 lines): `TempVcs` does real
  filesystem writes under a temp root; `ScriptedAgent` records dispatched tasks,
  instructions, and polls.
- `crates/agileplus-cli/tests/implement_command_flow.rs` (385 lines): 9 tests driving
  the real `run_implement` over a real `SqliteStorageAdapter::in_memory()`.

Covers: wrong-state rejection without polling, unknown-feature guidance, no-WP
rejection, happy path Planned→Implementing→Done with worktree create+cleanup, max-cycles
exhaustion blocking the WP while retaining the worktree and feeding back the last agent
output verbatim, hard agent failure, single-WP and unknown-WP selectors, and missing
prompt artifacts aborting before dispatch.

Result: 9 passed, deterministic across 5 runs. `agileplus-cli` suite 972 passed.

### Wave 13 — `96958614` (pushed to `origin/main`)

`desktop/src-tauri` was 1467 lines, zero tests, and **0.00% on both coverage bases**.

Obstacle: every command takes `tauri::State<'_, AppState>`, whose tuple field is
private, so no external test can call any of them. Solved with Tauri's own `test`
feature, which provides `MockRuntime` plus real IPC dispatch.

- `desktop/src-tauri/src/test_support/desktop.rs` (426 lines), declared `#[cfg(test)]`
  in `lib.rs` so no production path changes.
- Handler list mirrors `lib.rs::run`, which itself cannot be called in a test (it
  installs a panic hook, builds a tray icon, and enters the platform event loop).

14 tests covering repo-path round-trip, ADR listing with numeric-prefix parsing and
`Status:` extraction, ADR read plus both failure modes, trace listing with extension
filtering and descending sort, trace read preferring `.jsonl` over `.md`,
`open_project` rejecting a non-project dir, and uninitialized-database errors.

Two behaviors were discovered by writing assertions rather than assuming them:
- Tauri maps `snake_case` command params to `camelCase` over IPC (`feature_id` →
  `featureId`).
- `list_adrs` sorts the id **string** descending, so unprefixed `notes` outranks
  `0002`. The test now documents that ordering.

Result: 14 passed, deterministic across 5 runs. `cargo clippy --all-targets` clean.

The tauri's `test` feature is declared under `[dev-dependencies]`, not
`[dependencies]`, so release builds do not compile the mock runtime. Verified
against the graph rather than by inspection: `cargo tree --edges normal` lists
tauri's features with no `test` entry. This was initially committed into
`[dependencies]` and corrected in `0a8579b1`.

## Measurement honesty

Two bases are reported and they disagree structurally. This is not a bug in either:

| Basis | Meaning | Before | After | Delta |
|---|---|---|---|---|
| production-only | non-test rlib objects only | 50.17% (15625/31145) | 50.72% (16578/32685) | +0.55pp |
| test-inclusive | harness objects, counts a file's own `#[cfg(test)]` | 93.10% (82271/88370) | 93.67% (83024/88630) | +0.57pp |
| `desktop/src-tauri` (test-inclusive) | — | 0.00% | 36.50% (438/1200) | +36.50pp |

Note the denominators moved too (31145→32685 production, 88370→88630
test-inclusive). The before/after figures are not a like-for-like
denominator comparison; the line count grew because the instrumented build
was a clean rebuild after `target/llvm-cov-target` had been cleared, not
because production source was added.

`desktop/src-tauri` still reports **0.00%** on the production-only basis after
wave 13. That crate's lib is `crate-type = ["staticlib", "cdylib", "rlib"]` but
the instrumented build produced no rlib for the report to key against, so the
harness objects are the only source of counts. The tests genuinely execute; the
production basis structurally cannot see them.

The production-only basis structurally cannot see tests that live in the test
harness. `commands/review_loop.rs` reports **0.00%** production-only while reporting
**97.47%** test-inclusive. The same artifact pins `agileplus-api` at 17.27% and
`agileplus-import` at 12.81% despite those crates having 99%+ test-inclusive coverage.
Documented in `scripts/coverage-complete.sh` lines 104-111.

## Findings not fixed (production changes, out of scope for a test-only wave)

1. **`ReviewOutcome::Cancelled` is dead code.** In `commands/review_loop.rs` it is
   declared at line 20 and constructed nowhere except its own in-file assertion at
   line 190. `run_review_loop` never returns it.
2. **Two network tests are intermittently fragile.** Both target `192.0.2.1`
   (RFC 5737 TEST-NET-1) and assume the failure mode is a *connection timeout*.
   When the OS instead returns `ENETUNREACH` first, the premise is wrong:
   - `agileplus-api` `probe_tcp_url_reports_timeout_for_unroutable_address`
     expects `"192.0.2.1:80: connection timed out"` but received
     `"192.0.2.1:80: Network is unreachable (os error 51)"`.
   - `agileplus-p2p` `probe_agileplus_unroutable_ip_times_out_to_unknown`
     expects `PeerStatus::Unknown` but received `PeerStatus::Offline`.

   Observed exactly once, in the first full instrumented workspace run. The
   second full run passed both. They also pass in a normal build, and passed 3x
   each under `cargo llvm-cov` in isolation (`agileplus-api` 200/200 three
   times, `agileplus-p2p` 224/224 three times). So the trigger is
   nondeterministic rather than strictly load-dependent, and the honest
   characterisation is an environment-dependent assumption, not a
   reproducible defect in these tests. The underlying fragility is real: the
   test hard-codes one of two legitimate failure modes for an unroutable
   address.

   Neither crate is touched by this campaign, and the fix would be a
   production-file edit, so it is left for a separate change.

## Environment notes

- Instrumented coverage needs ~16G in `target/llvm-cov-target`. Disk exhaustion
  (`ld: write() failed, errno=28`) previously killed a run at link time and looked
  like a test failure. Reclaimed by clearing git-ignored regenerable caches:
  `target/llvm-cov-target` and `target/debug/incremental` (26G).
- `scripts/coverage-complete.sh` uses `set -e`, so one failing test aborts before the
  merge step and no report is written. The underlying profile data survives, but a
  rerun is needed.
- `cargo fmt -p <crate>` reformats unrelated files; use
  `rustfmt --edition 2021 <file>` directly.
