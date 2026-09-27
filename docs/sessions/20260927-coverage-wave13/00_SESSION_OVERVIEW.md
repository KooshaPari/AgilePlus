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

Result: 14 passed, deterministic across 5 runs. `cargo clippy --all-targets` clean;
`cargo build -p agileplus-desktop` still succeeds, confirming the added `test` feature
does not affect non-test builds.

## Measurement honesty

Two bases are reported and they disagree structurally. This is not a bug in either:

| Basis | Meaning | Workspace | `desktop/src-tauri` |
|---|---|---|---|
| production-only | non-test rlib objects only | 50.17% (15625/31145) | 0.00% before wave 13 |
| test-inclusive | harness objects, counts a file's own `#[cfg(test)]` | 93.10% (82271/88370) | 0.00% before wave 13 |

The production-only basis structurally cannot see tests that live in the test
harness. `commands/review_loop.rs` reports **0.00%** production-only while reporting
**97.47%** test-inclusive. The same artifact pins `agileplus-api` at 17.27% and
`agileplus-import` at 12.81% despite those crates having 99%+ test-inclusive coverage.
Documented in `scripts/coverage-complete.sh` lines 104-111.

## Findings not fixed (production changes, out of scope for a test-only wave)

1. **`ReviewOutcome::Cancelled` is dead code.** In `commands/review_loop.rs` it is
   declared at line 20 and constructed nowhere except its own in-file assertion at
   line 190. `run_review_loop` never returns it.
2. **Two load-sensitive network tests flake under instrumentation.** Both pass in a
   normal build and both pass 3x under `cargo llvm-cov` in isolation, but they fail
   when the whole instrumented workspace suite runs concurrently:
   - `agileplus-api` `probe_tcp_url_reports_timeout_for_unroutable_address` expects
     `"connection timed out"` but receives `"Network is unreachable (os error 51)"`.
   - `agileplus-p2p` `probe_agileplus_unroutable_ip_times_out_to_unknown` expects
     `PeerStatus::Unknown` but receives `PeerStatus::Offline`.

   Both target `192.0.2.1` (RFC 5737 TEST-NET-1) and assume the failure mode is a
   *timeout*. Under load the OS returns ENETUNREACH first, so the premise is wrong.
   Neither crate is touched by this campaign. This is a real test defect worth fixing,
   but it is a production-file edit, so it is left for a separate change.

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
