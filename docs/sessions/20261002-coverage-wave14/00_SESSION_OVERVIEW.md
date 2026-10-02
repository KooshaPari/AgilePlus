---
session: wave-14-plan-coverage
date: 2026-10-01 to 2026-10-02
agent: jcode
---

# Coverage Campaign: wave 14 (`plan`), plus the `agileplus-nats` clippy fix

## Goal

Raise behavior-level test coverage in AgilePlus toward 85%, exercising real
public command paths rather than test doubles, with no production-code changes.
A bounded clippy fix (18.5) was folded into the same session.

## What landed

### `agileplus-nats` clippy — `94f39b71` (pushed to `origin/main`)

`cargo clippy -p agileplus-nats --all-targets -- -D warnings` reported 42 lints
(25 lib + 17 test-only) across 6 files. All 42 cleared, plus 4 pre-existing
`useless_format` lints in `agileplus-sync`, the only workspace crate that
depends on it.

- `bus.rs`: `#[must_use]` on `new`/`in_memory`/`backend`; `# Errors` sections on
  `publish`/`subscribe`/`unsubscribe`/`request`; `# Panics` on `published()`;
  merged two identical `Timeout` match arms.
- `envelope.rs`: `#[must_use]` on `new`/`with_reply_to`/`with_correlation`;
  `# Errors` on `to_bytes`/`from_bytes`.
- `subject.rs`: `#[must_use]` on `for_event`/`all_for_entity`/`all_of_type`/
  `as_str`/`matches`.
- `config.rs`, `lib.rs`: `#[must_use]` on `with_auth`/`with_prefix`, backticked
  `AgilePlus`.
- Test-side: dropped a provably-always-true `assert_eq!(nanos >= 0, true)`
  tautology; replaced a PI-approximating payload `3.14` with `3.5`; merged
  identical match arms; `assert!(a == b)` -> `assert_eq!`/`assert_ne!`;
  `contains('*')`; and `re_exports_are_accessible` now asserts real behavior
  instead of binding five `let _x` values.

**Why only these two crates.** The `#[must_use]` additions are the one change
that can break code outside the edited files, so the dependent crate was checked
explicitly. A full `cargo clippy --workspace --all-targets` was also run: it
fails in `agileplus-agent-review` (10 pre-existing lints) and 5 more in
`agileplus-cli` (`context_tests` unused import, `fix_list.rs` x2 useless
conversion, `repl.rs` `approx_constant`, `branch.rs` derivable impl,
`validate/evidence.rs` items-after-test-module). None are in this wave's files,
none are `must_use` fallout, and all are pre-existing. The chain continues across
~40 crates, so CI work stops here by prior decision.

### Wave 14 — `plan` command (not yet committed at time of writing)

`src/commands/plan.rs` is 1079 lines: 589 production, 490 in-file
`#[cfg(test)]`. It had **zero tests under `tests/`**, so nothing counted on the
production-only basis. It is wired into the `Commands` enum
(`main.rs:212` -> `agileplus_cli::commands::plan::run_plan`).

- `tests/support/plan.rs` (238 lines): `PlanVcs`, a `VcsPort` that reads and
  writes **real files** under a temp root, so `plan.md`, the per-WP prompts and
  the governance contract are asserted where they land on disk. Git-lifecycle
  methods a planning run never calls are `unimplemented!` so an unexpected call
  fails loudly. Plus `seed_feature`, `args`/`args_with_max_wps`,
  `spec_with_frs`/`spec_without_frs`, `write_spec`.
- `tests/plan_command_flow.rs` (367 lines): 9 tests over real
  `SqliteStorageAdapter::in_memory()`.

Coverage: unknown-feature guidance, the already-`Planned` guard, the empty-spec
placeholder WP, FR grouping under two `max_wps` caps, the `Researched ->
Planned` transition plus audit entry, the non-`Researched` warn-and-proceed path
(no transition, no audit), all three artifacts written to disk, rerun reuse of
work packages and the governance contract, and the reconcile refusal when a
persisted WP does not match the generated plan.

## Findings not fixed (production changes, out of scope for a test-only wave)

1. **A successful plan is never re-plannable, so reconcile only ever sees a
   partial run.** `run_plan` transitions `Researched -> Planned`, and the guard
   at `plan.rs:66-72` then rejects any second run with *"already in 'Planned'
   state"* before reconciliation is reached. The comment at `plan.rs:126-128`
   describes planning as "retry-safe", but that path is reachable only when the
   first run created work packages and then failed *before* the state
   transition, or when the feature sits in a state that neither transitions nor
   blocks (e.g. `Created`). The retry test therefore deliberately seeds
   `FeatureState::Created`; seeding `Researched` makes the second run impossible
   to observe. The transition test asserts the second run's rejection, so the
   guard itself is pinned down rather than assumed.

2. **The governance artifact is written with `id: 0` and only becomes aligned
   with the database on a later write.** `run_plan` discards the id returned by
   `create_governance_contract` and serialises the in-memory contract, so after
   the first run the artifact reads `"id": 0` while the row holds its real id
   (observed `1`). The comment at `plan.rs:197-199` claims the artifact and
   database stay "byte-for-byte aligned"; that is true of `bound_at` and `rules`
   (which is what proves reuse) but not of `id`. The test asserts the
   post-second-run alignment and records the first-run divergence in a comment
   rather than encoding the misalignment as expected behaviour.

Both were found by writing assertions that failed, not by reading the code.

## Mutation verification

Four mutations, each applied to `src/commands/plan.rs`, run, and reverted; the
file was byte-identical to `HEAD` (`git diff` empty) after every one.

| # | Mutation | Caught by |
|---|---|---|
| 1 | Reconcile mismatch condition forced to `false` | `a_persisted_wp_..._requires_explicit_replan` |
| 2 | `group_frs_into_wps` ignores `max_wps` (always 3) | `functional_requirements_are_grouped...` (`left: 3, right: 2`) |
| 3 | `.filter(\|_\| false)` on the contract lookup (always rebuild) | `rerun_reuses_...` (second run: "persisting governance contract") |
| 4 | State-transition condition forced to `false` | `planning_transitions_...` (`left: Researched, right: Planned`) |

## Measurement honesty

**Coverage was NOT re-measured for this wave.** The instrumented run needs
~16G in `target/llvm-cov-target` and the disk had 5.3Gi free at the time
(previously 9.4Gi), so `scripts/coverage-complete.sh` cannot complete. No
coverage percentage is claimed for wave 14 anywhere in this document.

The last measured production-only figure remains **50.71% (16575/32685)**,
taken on `49a3862d` during wave 13. This wave adds tests but that number has
not moved on the record until a clean instrumented run is possible.

`plan.rs`'s 490 in-file test lines contribute to the test-inclusive basis only;
this wave's 9 tests link the rlib and therefore count on the production-only
basis.

## Verification record

Every claim below was produced on this machine.

- `cargo test -p agileplus-cli --test plan_command_flow`: 9 passed, 0 failed.
- Determinism: 5 consecutive runs, 9 passed each time (0.16s-0.30s).
- Full `cargo test -p agileplus-cli`: exit 0, every target `ok`, 0 failed. The
  single `ignored` is a pre-existing Doc-test, not in this wave's files.
- `cargo fmt --all --check`: clean (CI's rustfmt via `RUSTUP_TOOLCHAIN=1.98.1`).
- `cargo clippy -p agileplus-cli --all-targets -- -D warnings`: the 5 reported
  lints are all outside this wave's files (locations listed above); zero in
  `tests/plan_command_flow.rs` and `tests/support/plan.rs`.
- `git diff -- crates/agileplus-cli/src/commands/plan.rs`: empty after all four
  mutations. No production line was changed by this wave.

Not verified, and therefore not claimed:

- No coverage percentage for wave 14 (disk-blocked, see above).
- No same-tree baseline, same as prior waves.
- The two findings above rest on observed test failures and on reading
  `run_plan`'s control flow, not on a fix-and-retest cycle.
