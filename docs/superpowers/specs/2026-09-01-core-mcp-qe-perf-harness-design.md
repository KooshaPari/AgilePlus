# Core/MCP QE and Performance Harness Design

## Purpose

Restore meaningful QE and performance gates for the AgilePlus Rust gRPC core and Python MCP bridge. The previous workflows referenced assets that do not exist in the repository, preserved history, or overlapping local repositories. This design replaces those synthetic references with a minimal harness derived from the current runtime contracts.

## Scope

The harness validates the repaired runtime represented by PR #1055:

- `agileplus-grpc` starts on an ephemeral loopback port with an ephemeral SQLite database.
- `agileplus-mcp` connects to that exact gRPC address.
- QE exercises readiness, gRPC contract round trips, MCP-to-core behavior, and shutdown.
- Performance exercises smoke, steady-load, and bounded-stress scenarios with explicit error-rate and p95 budgets.

The harness does not restore the full Plane, Neo4j, NATS, MinIO, or dashboard platform. Those services are outside the repaired core/MCP path and require a separate platform-harness design.

## Architecture

```text
test orchestrator
    |
    +-- agileplus-grpc
    |     +-- 127.0.0.1:<ephemeral>
    |     +-- <temporary>/core.db
    |
    +-- agileplus-mcp
          +-- 127.0.0.1:<ephemeral>
          +-- AGILEPLUS_GRPC_ADDRESS=<core address>

QE clients --------> gRPC and MCP behavioral contracts
k6 clients --------> bounded runtime endpoints and budgets
```

The orchestrator owns process startup, readiness polling, log capture, and teardown. It must terminate both child processes on success, failure, or interruption.

## Components

### Runtime orchestration

- Resolve free loopback ports before startup.
- Create a temporary state directory and SQLite database path.
- Build or locate the exact candidate binaries and Python package.
- Start the Rust core first and require a real gRPC readiness response.
- Start MCP with the resolved core address and require an MCP readiness response.
- Capture stdout, stderr, exit status, environment shape, and readiness timing without recording secrets.
- Tear down only processes started by the harness.

### QE suite

The initial suite covers:

1. Core and MCP readiness.
2. Feature/list contract behavior against an empty database.
3. Backlog create/list round trip, including feature association and unsupported WP rejection.
4. Audit pagination and chain verification behavior.
5. MCP tool invocation reaching the Rust core rather than returning scaffolded `not_implemented` responses.
6. Clean shutdown and database persistence across one controlled core restart.

Each test must identify the failing boundary: process startup, transport, contract decoding, domain behavior, or persistence.

### Performance suite

The initial k6 scenarios are:

- Smoke: one virtual user, short duration, proves scenario correctness.
- Load: bounded concurrency representative of local agent use.
- Stress: a short, capped concurrency increase intended to expose saturation without becoming a soak test.

Budgets are configuration, not hard-coded in workflow YAML. Initial gates cover request error rate, p95 latency, and process crash count. Reports include raw k6 JSON, summary JSON, runtime logs, and the exact candidate SHA.

## Error handling and evidence

- A readiness timeout fails with the last process logs and resolved ports.
- A child-process exit fails with its exit code and captured stderr.
- A contract failure records the request name and response status without sensitive payloads.
- Performance failure records the breached threshold and observed value.
- Teardown runs unconditionally and reports any surviving harness-owned PID.
- Hosted workflows upload evidence on both success and failure.

## CI integration

- Replace the nonexistent QE/performance asset references only after the new harness passes locally.
- Keep the existing hosted check names so branch protection remains stable.
- Use locked Rust and Python dependencies where lockfiles exist.
- Do not contact or mutate the live AgilePlus daemon, database, or ports.
- Do not weaken gates with unconditional skips or advisory-only success paths.

## Ownership and sequencing

Two implementation lanes may proceed independently after the shared orchestrator interface is defined:

- QE lane owns the orchestrator, readiness probes, behavioral tests, and QE workflow.
- Performance lane owns k6 scenarios, budgets, report validation, and performance workflow.

The QE lane lands first because performance scenarios depend on a proven startup and readiness contract. The performance lane may develop fixtures in parallel but integrates only after that contract is stable.

## Acceptance criteria

- The harness uses only ephemeral ports, paths, and harness-owned processes.
- Local QE passes against a freshly built candidate.
- Local smoke, load, and stress scenarios produce machine-readable reports.
- Intentional core or MCP startup failure produces actionable diagnostics.
- Hosted QE and performance jobs execute the restored assets without missing-file errors.
- Evidence artifacts contain the candidate SHA and no secrets.
- Existing core/MCP unit, contract, formatting, and lint gates remain green.
- No live deployment occurs until reviewer approval and all required hosted gates pass.

## Deferred work

- Full platform orchestration with Plane, Neo4j, NATS, MinIO, or dashboard services.
- Long-duration soak testing and distributed load generation.
- Production SLO selection based on hosted telemetry.
- Replacement of unrelated unmaintained transitive Rust dependencies.
