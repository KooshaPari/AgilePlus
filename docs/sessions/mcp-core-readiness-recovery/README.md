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
