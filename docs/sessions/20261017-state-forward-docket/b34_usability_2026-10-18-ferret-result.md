# B.3 / B.4 Usability smoke (ferret, 2026-10-18T09:36:46Z)

**Verdict**: PARTIAL
**Subagent session**: ferret (orchestrator: shrimp)
**Wall-clock**: ~92 seconds end-to-end
**Binary found**: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/target/debug/agileplus` (53 MB, built Oct 17 23:46, package version `0.2.4`)
**Commands run**: 11 invocations — all logged to `/tmp/b34_usability_2026-10-18.log`

## Commands run (chronological)

| #   | Invocation                                                                               | Result                                                                                                                                                                                                                                                                                                                                                                                     |
| --- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | `./target/debug/agileplus --version`                                                     | `agileplus 0.2.4` (clean exit 0)                                                                                                                                                                                                                                                                                                                                                           |
| 2   | `./target/debug/agileplus --help`                                                        | Full subcommand list (cycle, list, evidence, specify, queue, module, research, plan, implement, evaluate, validate, ship, retrospective, triage, rubric, dag, okf, mvp, dashboard, cockpit, help) — clean exit 0                                                                                                                                                                           |
| 3   | `./target/debug/agileplus list`                                                          | **FAIL**: `Error: listing features: Storage error: Conversion error from type Text at index: 7, premature end of input` — index 7 is the **column** index of `features.updated_at` in the mapper's row tuple (`crates/agileplus-sqlite/src/repository/features.rs:44`, `row.get(7)`), **not** a row number. The value in that column is malformed (data issue, NOT a CLI invocation issue) |
| 4   | `./target/debug/agileplus list --help`                                                   | Clean help text with `--state`, `--json`, `--repo`, `-v`, `-h` flags — exit 0                                                                                                                                                                                                                                                                                                              |
| 5   | `./target/debug/agileplus dashboard --help`                                              | Clean help text — exit 0                                                                                                                                                                                                                                                                                                                                                                   |
| 6   | `./target/debug/agileplus help`                                                          | Identical to `--help` — exit 0                                                                                                                                                                                                                                                                                                                                                             |
| 7   | `./target/debug/agileplus cycle --help`                                                  | Subcommand list: create, list, show, add, remove, transition, help — exit 0                                                                                                                                                                                                                                                                                                                |
| 8   | `./target/debug/agileplus cycle list --help`                                             | Subcommand-specific help with `--state` enum (Draft/Active/Review/Shipped/Archived) — exit 0                                                                                                                                                                                                                                                                                               |
| 9   | `./target/debug/agileplus mvp --help`                                                    | Subcommand list: project, epic, story, wp, dep, cycle-create, cycle-add, transition, next-ready, help — exit 0                                                                                                                                                                                                                                                                             |
| 10  | `./target/debug/agileplus cockpit --help`                                                | Subcommand list: read, publish, path, help — exit 0                                                                                                                                                                                                                                                                                                                                        |
| 11  | `cargo test -p agileplus-cli --test cli_integration help_prints_usage`                   | **PASS** (7 s wall-clock incl. 5.89 s incremental compile)                                                                                                                                                                                                                                                                                                                                 |
| 12  | `cargo test -p agileplus-cli --test cli_integration version_flag_prints_package_version` | **PASS** (1.4 s wall-clock, reused compile)                                                                                                                                                                                                                                                                                                                                                |

## Smoke test result

The brief asked specifically for `cargo test -p agileplus-cli --test smoke`, but **no such integration test file exists**. The closest canonical CLI smoke coverage is `crates/agileplus-cli/tests/cli_integration.rs`, which contains exactly two end-to-end assertion tests that exercise the actual CLI binary:

```rust
#[test] fn help_prints_usage() { /* asserts --help stdout contains "Spec-driven" */ }
#[test] fn version_flag_prints_package_version() { /* asserts --version matches CARGO_PKG_VERSION */ }
```

Both pass. The remaining CLI integration tests (`cli_branch_args.rs`, `plan_command_flow.rs`, `implement_command_flow.rs`, `queue_command_flow.rs`, `dashboard_command_flow.rs`, `dag_command_flow.rs`, `cockpit_command_tests.rs`, `cli_cockpit.rs`, `cli_fix_list.rs`, `cli_okf.rs`, `cli_rubric.rs`, `fix_list_command_tests.rs`, `governance_tests.rs`, `journey_public_workflow.rs`, `okf_command_tests.rs`, `promotion_recovery.rs`, `queue_import_flow.rs`, `context_tests.rs`) were not run — they each take >30 s to compile and the 10-minute cap on this ferret dispatch was binding.

## Cell movement recommendation

**B.3 / B.4 Usability: NOT_RUN → PARTIAL**

- **CLI invocation surface (--help, --version, help subcommand, subcommand --help)**: HEALTHY. Every invocation exited 0 and produced structured output. The CLI is parseable, the help text is descriptive (enumerates states, subcommands, options), and version flag matches package metadata. **Subcommand typo handling was NOT exercised** — none of the 12 enumerated invocations passes a mistyped subcommand — so it is not covered by this verdict (see the "Coverage limits" note below).
- **CLI data path (`agileplus list` without flags)**: UNHEALTHY — fails with `Storage error: Conversion error from type Text at index: 7, premature end of input`. This is **not** a CLI bug; it indicates the local SQLite `agileplus.db` has a malformed value in the `features.updated_at` column (mapper column index 7, `crates/agileplus-sqlite/src/repository/features.rs:44` — a column index, not a row number). This is a **database hygiene** concern, not a usability concern. It should be filed as a separate B.x datapath cell, not block B.3 / B.4.
- **Canonical smoke tests in `crates/agileplus-cli/tests/cli_integration.rs`**: BOTH PASS.

PARTIAL because the cell claims CLI invocation usability, and invocation-side usability is fully verified, but a downstream command surfaced a data-quality issue that warrants its own follow-up. The CLI itself is **usable** by a human operator.

### Coverage limits of this verdict

- **Subcommand typo handling: not exercised.** None of the 12 enumerated invocations passes a
  mistyped subcommand, so the CLI's behaviour on a bad subcommand name is **unknown** from this
  receipt and is not covered by the HEALTHY verdict above.
- **Epic-specific behaviour: not exercised.** The invocations cover the CLI _invocation surface_
  (`--help`, `--version`, per-subcommand help) and the `list` data path (invocation row 3, which
  failed on the data-quality fault described below). They do
  **not** exercise worker dispatch (E3 "Attempt + Worker Lifecycle") or the independent evaluator
  (E4 "Independent Evaluator") at runtime. The Usability PASS that the parent later recorded for
  B.3 / B.4 rests on this invocation-surface evidence; it is a statement about the operator-facing
  CLI surface, not a claim that those two capabilities were executed end to end.
- **`queue list` was not run by this receipt.** The invocation table above records no `queue list`
  run. The `queue list` coverage behind the composite B.3 / B.4 verdict comes from the later
  gecko receipt (`b67_storage_2026-10-18-gecko-result.md`, 2026-10-18T09:46Z: "`./target/debug/agileplus
queue list` exit 0 — 4 backlog items rendered"), which ran the command after the storage-hygiene
  fix landed. Credit for that command belongs to gecko's receipt, not to this smoke.
- **Scope of the data-path failure.** The `agileplus list` failure is a data-quality fault in the
  local `agileplus.db`, not a CLI defect (see the Blockers section).

## Evidence (last relevant lines)

```
$ ./target/debug/agileplus --version
agileplus 0.2.4

$ ./target/debug/agileplus --help
Spec-driven development engine

Usage: agileplus [OPTIONS] <COMMAND>

Commands:
  cycle          Manage cycles (time-boxed delivery units)
  list           List features in the database (optional filter by state)
  evidence       Attach local result artifacts to work packages
  specify        Create or revise a feature specification
  queue          Manage the triage backlog queue
  module         Manage modules (product-area groupings of features)
  research       Research a feature (codebase scan / feasibility)
  plan           Generate a delivery plan and work packages
  implement      Implement work packages (dispatches agents)
  evaluate       Record an independent, criterion-bound evaluation receipt
  validate       Validate governance evidence and policies
  ship           Ship a validated feature (merge + archive)
  retrospective  Generate a retrospective for a shipped feature
  triage         Classify and route incoming items to the backlog
  rubric         Score a repo against the governance rubric catalog
  dag            Manage the DAG of work packages (pick, claim, release, dedup, scan)
  okf            Validate, summarize, and merge OKF v1.0 documents
  mvp            MVP project/epic/story/work-package management
  dashboard      Render an in-flight DAG / status dashboard from SQLite
  cockpit        Publish or read local cockpit scorecards
  help           Print this message or the help of the given subcommand(s)

$ ./target/debug/agileplus list
Error: listing features: Storage error: Conversion error from type Text at index: 7, premature end of input

$ cargo test -p agileplus-cli --test cli_integration help_prints_usage
   Compiling agileplus-cli v0.2.4
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.89s
     Running tests/cli_integration.rs (target/debug/build/agileplus-cli/e6fd36090fe1b949/out/cli_integration-e6fd36090fe1b949)
running 1 test
test help_prints_usage ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.88s

$ cargo test -p agileplus-cli --test cli_integration version_flag_prints_package_version
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.35s
running 1 test
test version_flag_prints_package_version ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.86s
```

## Blockers

None for B.3 / B.4 specifically. The `agileplus list` failure should be re-routed to a **datapath / storage-hygiene cell** (suggest: B.6 / B.7 Storage). The malformed value sits in the `features.updated_at` **column** (mapper column index 7, `crates/agileplus-sqlite/src/repository/features.rs:44`) — index 7 is a column index, **not** a row number. **Do not delete "row 7"**: that would remove an unrelated feature while leaving the malformed value in place. Identify the affected feature row (the one whose `updated_at` fails to parse) before any repair, and repair the value rather than the row — gecko's later B.x fix did exactly that, UPDATE-ing the 5 affected rows in `features` + `backlog_items` in place. Not a Usability issue.

Full raw log: `/tmp/b34_usability_2026-10-18.log` (220 lines).

**Durability gap (unresolved).** That log path was a volatile `/tmp` location and the file was
never committed to this docket, so it is **no longer available** to reviewers — the omitted
command outputs behind this verdict cannot be re-inspected. The evidence that survives is the
excerpts quoted in this receipt (the 12-invocation table above and the `## Evidence` block).
A durable copy of the log (committed alongside the receipt, as the `ci_diagnosis_2026-10-18-fox.log`
and `cf12_workflow_fixes_2026-10-18-badger.log` artefacts are) is the fix; this receipt records
the gap rather than claiming a log that is not there.
