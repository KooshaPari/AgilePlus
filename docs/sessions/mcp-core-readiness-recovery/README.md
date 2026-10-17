# Canonical MCP/core readiness recovery

The canonical Python package is `python/pyproject.toml` (`agileplus-python`):
its `agileplus-mcp` entrypoint is `agileplus_mcp.server:main`, and its wheel
includes `src/agileplus_mcp` and generated `src/agileplus_proto`.
The separate `agileplus-mcp/` package is legacy. Earlier broad descriptions of
"the MCP" as a stub do not describe this canonical proto-backed implementation.
This correction preserves the historical evidence rather than rewriting it.

This candidate keeps the grpcio channel and generated stubs after initial
readiness fails. grpcio can reconnect the same channel when the core starts or
restarts. `connect()` reuses it. Readiness waits default to one second; each
retrying unary operation has a five-second total budget including backoff.
Finite audit queries have the same deadline and classified failures. Long-lived
agent event subscriptions retain their existing streaming semantics. Backlog
create retains its separate no-retry semantics; this change does not claim a
new timeout policy for that operation.

`health_check` preserves `status`, `mcp_server`, `grpc_core`, `version` and error
keys and adds independent boolean `live` and `ready` fields. A reachable MCP
with unavailable core reports live=true, ready=false. Readiness executes a real
project-scoped feature query. Bad protobuf payloads and missing feature messages
cannot produce a successful feature result. The context manager closes a
channel even when initial readiness fails.

The real local journey uses the existing CLI adapter to specify a feature in a
disposable Git repository/SQLite store. One stdio MCP session starts before a
core bound to an ephemeral loopback port, then queries feature and dashboard,
observes core failure, and recovers the same durable feature ID and dashboard
after restarting only its owned core process. `real-e2e.json` contains exact
inputs/artifact hashes and output observations. The cached core's source SHA
and version are UNKNOWN; the cached CLI reports 0.2.4. These Rust artifacts are
independent from the new Python candidate. No shared service was changed.
Core shutdown drains open channels, so the fixture allows two seconds then
kills only its own child; MCP intentionally retains its channel across restart.

Validation: `uvx ruff check python/` passed. Full Python suite passed: 91 passed,
10 skipped, including the real process journey. The skipped tests are existing
external-consumer contract cases; skipped does not mean passed. New controls
cover initial unavailable core, real late startup/restart, hung unary deadline
and cancellation, total retry budget, malformed protobuf wire bytes, absent
feature message, hung audit stream cancellation, and context cleanup.
Rust checks/clippy/workspace tests were NOT_RUN: no Rust source changed and this
bounded slice deliberately reused cached artifacts. Placement: all on device,
lightweight Python plus cached Rust executables; no heavy Rust build dispatched.

Reproduce from this worktree (Python 3.14; explicit binaries required):

```sh
uvx ruff check python/
AGILEPLUS_TEST_CORE_BINARY=/absolute/path/agileplus-grpc \
AGILEPLUS_TEST_CLI_BINARY=/absolute/path/agileplus \
AGILEPLUS_TEST_RECEIPT=/tmp/agileplus-recovery.json \
uv run --project python pytest python/tests
```

This is engineering/local consumer execution evidence for a bounded recovery
journey. User validation is NOT_RUN. It is not release, deployment, or portfolio
completion. State & Forward Docket authority: PhenoRegistry PR660,
`05e163820f1a207f2f827a5485ce6ca04084107f`,
`handbook/specs/assessment/state-forward-docket-v1/CONTRACT.md`.
Numeric implementation, verified engineering, and usable journey axes remain
unscored pending the owner-held denominator and independent disposition;
user validation is separate. This SOURCE_REPORT alone earns no operational
credit.

## Source-bound core followup

A new core was built directly from the clean tracked candidate
`b045f195311a683a12f65e71d02ce6490683e0b3` (tree
`6f35ba5d9c1538ed55b5fba60d6988080a00e790`) with unchanged Cargo.lock
`503449ad658a0f1d2415b6981898745e455a53ec20c765aca1a353f1eff8d80d`.
All workspace/path dependencies are inside that same checkout, with no external
path dependency. The scoped `cargo build --locked --offline -j 1 -p
agileplus-grpc --bin agileplus-grpc` completed in 3m39s using an owned isolated
`.recovery-target`; SDKROOT and macOS deployment target were explicit.

The new core SHA-256 is
`ef3da2c32b3928500765a7dae92e792616cea2cc53feac39080fad6f283ecdd1`.
Its package version is 0.2.4; no version string was inserted into source.
`source-bound-core-e2e.json` preserves the repeated real MCP/SQLite journey:
1 passed in 10.44s, with late core startup and stop/restart recovery of identical
feature ID and dashboard in the same MCP session. The cached seed CLI remains
an independently hashed fixture with unknown source identity in this receipt.
The earlier `real-e2e.json` is preserved unchanged and still describes its
original cached core as UNKNOWN.

Placement was all on device: measured 16GiB physical RAM, 48% memory free by
`memory_pressure -Q`, 121GiB disk available, and one Cargo job. Resource peaks
were not instrumented. No shared target, process, runtime, or global config was
changed. Whole-workspace offline metadata had an unrelated uncached Android
package; the actual scoped core build succeeded fully offline.

The optional source CLI build also completed offline in 3m03s from the same
candidate/lock/owned target (`-p agileplus-cli --bin agileplus`). The same real
journey rerun using BOTH newly built artifacts passed in 8.47s.
`source-bound-core-cli-e2e.json` binds their exact paths, hashes, commands,
source revision and outputs. This removes the cached seed CLI identity gap for
this new execution while retaining both earlier executions unchanged in meaning.
No source files changed for the provenance upgrade; user validation remains
NOT_RUN and release/deployment acceptance remains unclaimed.
