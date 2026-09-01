# Core/MCP QE and Performance Harness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace AgilePlus's nonexistent QE/performance assets with a locally reproducible, hosted core/MCP contract and load harness that produces immutable candidate evidence.

**Architecture:** A Python orchestrator owns ephemeral loopback ports, a temporary SQLite database, the exact Rust gRPC binary, and the canonical Python MCP process. QE calls the real Python gRPC client and FastMCP HTTP client. Performance reuses the same core startup contract and drives `ListFeatures` through k6 gRPC. Every process, log, report, Git SHA, and binary digest belongs to the harness; the live daemon and full platform stack remain untouched.

**Tech Stack:** Rust/Tonic, Python 3.14, uv lockfile, pytest/pytest-asyncio, FastMCP 3, k6 gRPC, GitHub Actions, SQLite.

---

## Invariants and sequencing gates

- Work only in `feat/core-mcp-qe-perf-harness-20260901`; preserve every other branch and worktree.
- Integrate `origin/main` after merged PR #1056 before changing workflows. Do not rewrite history.
- Never probe gRPC reflection: `agileplus-grpc` exposes no reflection or tonic health service.
- Never accept TCP readiness alone. Core readiness is `AgilePlusCoreClient.list_features()`; MCP readiness is `health_check` returning `status=healthy` and `grpc_core=ok`.
- Never point the CLI, core, or MCP at the live daemon, live ports, candidate worktree database, or candidate Git checkout.
- Use `Popen(start_new_session=True)` and signal only harness-owned process groups.
- Evidence allowlists environment keys; it never serializes the inherited environment or secrets.
- Preserve hosted job names exactly: `QE suite (100% green)`, `qe-gate (100% pass required)`, `perf (k6 -> 95p budget)`, and `perf-qe-gate (100% green)` (the workflow may retain its existing Unicode arrow spelling).
- Do not push until local QE plus all three k6 scenarios pass and both independent reviews approve.

## Task 1: Shared runtime orchestration, test-first

**Files:**

- Create: `qe/__init__.py`
- Create: `qe/runtime.py`
- Create: `qe/unit/__init__.py`
- Create: `qe/unit/test_runtime.py`

- [ ] Write failing unit tests for `reserve_loopback_port()`, `candidate_sha(repo)`, `binary_sha256(path)`, `build_core_environment(address, db)`, `build_mcp_environment(core_address, port)`, `start_process(...)`, `wait_until(...)`, and `stop_process(...)`.

The public shapes are:

```python
@dataclass
class ManagedProcess:
    name: str
    process: subprocess.Popen[bytes]
    stdout_path: Path
    stderr_path: Path

def reserve_loopback_port() -> int: ...
def candidate_sha(repo: Path) -> str: ...
def binary_sha256(path: Path) -> str: ...
def build_core_environment(address: str, database: Path) -> dict[str, str]: ...
def build_mcp_environment(core_address: str, port: int) -> dict[str, str]: ...
def start_process(name: str, argv: Sequence[str], env: Mapping[str, str], logs_dir: Path) -> ManagedProcess: ...
async def wait_until(name: str, probe: Callable[[], Awaitable[None]], process: ManagedProcess, timeout: float) -> None: ...
def stop_process(process: ManagedProcess, grace: float = 5.0) -> None: ...
```

Core environment must contain `AGILEPLUS_GRPC_BIND=127.0.0.1:<port>` and `AGILEPLUS_CORE_DATABASE_PATH=<temp>/core.db`. MCP must contain `AGILEPLUS_GRPC_ADDRESS`, `AGILEPLUS_MCP_TRANSPORT=http`, loopback host, ephemeral port, and `/mcp` path.

- [ ] Run RED:

```bash
uv run --project python --locked pytest -c python/pyproject.toml -q qe/unit/test_runtime.py
```

Expected: collection/import failure before `qe.runtime` exists, then assertion failures for unimplemented behavior.

- [ ] Implement minimally. Use files instead of `PIPE`, bounded polling, last-log diagnostics, TERM/wait/KILL only for the owned process group, and an idempotent cleanup registry used by `atexit`/signal handling.
- [ ] Run the same command GREEN, then `uv run --project python --locked ruff check qe` and `git diff --check`.
- [ ] Commit: `test(qe): add owned runtime orchestration`.

## Task 2: Real core fixture and readiness

**Files:**

- Create: `qe/conftest.py`
- Create: `qe/contract/__init__.py`
- Create: `qe/contract/test_core_contract.py`
- Modify: `qe/runtime.py`

- [ ] Write a failing async test that starts `target/debug/agileplus-grpc` with a fresh temp database, connects with `AgilePlusCoreClient(address)`, and requires `await client.list_features() == []`.
- [ ] Add `RuntimeHarness.start_core()` and `wait_for_core()`; reserve/close/start immediately and retry a bounded number of bind races. The fixture accepts prebuilt binary paths so pytest never hides compilation.
- [ ] Build exact candidates:

```bash
cargo build --locked -p agileplus-grpc --bin agileplus-grpc -p agileplus-cli --bin agileplus
```

- [ ] Run RED then GREEN:

```bash
uv run --project python --locked pytest -c python/pyproject.toml -q qe/contract/test_core_contract.py
```

- [ ] Commit: `test(qe): start the real grpc core`.

## Task 3: Backlog and persistence contracts

**Files:**

- Modify: `qe/contract/test_core_contract.py`
- Create: `qe/contract/test_persistence.py`
- Modify: `qe/runtime.py`

- [ ] Test `create_backlog_item(item_type="task", title=..., feature_id="qe-feature")`, then filtered `list_backlog(feature_slug="qe-feature")`; assert the same item id/title/body.
- [ ] Test `wp_id="WP01"` raises `GrpcCallError` whose code is `grpc.StatusCode.INVALID_ARGUMENT` and whose message identifies `wp_id`.
- [ ] Test restart: create an item, stop the old core, assert its PID exited, restart against the same DB (a new port is allowed), and find the exact item.
- [ ] Run both files RED then GREEN; intentional startup failure must show candidate paths, resolved ports, exit code, and last log lines.
- [ ] Commit: `test(qe): prove backlog persistence contracts`.

## Task 4: Canonical CLI-seeded audit contract

**Files:**

- Create: `qe/contract/test_audit_contract.py`
- Modify: `qe/conftest.py`

- [ ] Initialize a temporary Git repository and commit a seed file. Write a temp spec and seed the same temp DB before the core starts:

```bash
target/debug/agileplus --db <tmp>/core.db --repo <tmp>/repo \
  specify --feature qe-audit --from-file <tmp>/spec.md --target-branch main
target/debug/agileplus --db <tmp>/core.db --repo <tmp>/repo \
  specify --feature qe-audit --from-file <tmp>/spec-v2.md --target-branch main --force
```

Do not use gRPC `run_command("specify")`; it queues a command and does not mutate SQLite.

- [ ] Require page one `get_audit_trail("qe-audit", limit=1)`, page two using `after_id=page1[-1]["id"]`, no duplicate/gap, and `verify_audit_chain` returning `valid=True` with at least two verified entries.
- [ ] Run RED then GREEN and commit: `test(qe): verify paginated audit chain`.

## Task 5: Real HTTP MCP bridge

**Files:**

- Create: `qe/contract/test_mcp_bridge.py`
- Modify: `qe/runtime.py`

- [ ] Start MCP with `sys.executable -m agileplus_mcp`; do not nest another `uv run` process.
- [ ] Use `fastmcp.Client("http://127.0.0.1:<port>/mcp")` and `call_tool(name, arguments)`; decode `.data`.
- [ ] Require `health_check`, `list_features`, `agileplus_queue_add`, and filtered `agileplus_queue_list` to round-trip through the real core.
- [ ] Start MCP once with an absent core and prove `health_check` is unhealthy/`grpc_core=unreachable`; an open HTTP socket must not certify readiness.
- [ ] Run RED then GREEN and commit: `test(qe): prove mcp reaches rust core`.

## Task 6: QE evidence and hosted workflow

**Files:**

- Create: `scripts/qe/triage.py`
- Modify: `.github/workflows/qe.yml`
- Modify: `qe/runtime.py`

- [ ] Write a manifest test requiring candidate Git SHA, Rust binary SHA-256, Python lockfile SHA-256, sanitized ports/paths, readiness durations, process exits, and log paths under `.qe-evidence/`.
- [ ] Replace Node/Playwright/Docker/Process Compose and nonexistent `qe/fuzz`, `qe/bdd`, `qe/e2e_bdd`, `wait_health.sh`, and `triage.sh` steps. Use Python 3.14, install locked uv, build exact Rust binaries, run unit plus contract suites, and always upload `.qe-evidence/**` plus JUnit.
- [ ] Keep both QE job names and the hard gate unchanged. No skips or advisory success paths.
- [ ] Verify:

```bash
uv run --project python --locked pytest -c python/pyproject.toml -q qe/unit qe/contract --junitxml=.qe-evidence/junit.xml
uv run --project python --locked ruff check qe scripts/qe
actionlint .github/workflows/qe.yml
```

- [ ] Commit: `ci(qe): exercise the core mcp runtime`.

## Task 7: Performance budget parser, test-first

**Files:**

- Create: `perf/__init__.py`
- Create: `perf/tests/__init__.py`
- Create: `perf/tests/test_budget.py`
- Create: `perf/budget.json`
- Create: `scripts/perf/check_budget.py`

- [ ] Define configuration for request error rate, `grpc_req_duration` p95, and `core_crash_count`.
- [ ] Test pass, missing metric, latency breach, error-rate breach, malformed summary, and nonzero crash count.
- [ ] Run RED then GREEN:

```bash
uv run --project python --locked pytest -c python/pyproject.toml -q perf/tests/test_budget.py
uv run --project python --locked ruff check perf scripts/perf
```

- [ ] Commit: `test(perf): enforce configured runtime budgets`.

## Task 8: k6 gRPC scenarios and runner

**Files:**

- Create: `perf/lib/core.js`
- Create: `perf/smoke.js`
- Create: `perf/load.js`
- Create: `perf/stress.js`
- Create: `perf/runner.py`

- [ ] Load `agileplus/v1/core.proto` from `proto/` with `k6/net/grpc`; invoke `agileplus.v1.AgilePlusCoreService/ListFeatures` using `{ stateFilter: "" }`.
- [ ] Track a custom request-error `Rate`, `grpc_req_duration`, and a runner-owned crash count. Smoke is one VU/short duration; load is bounded steady concurrency; stress is a short capped ramp, never a soak.
- [ ] `perf.runner` reuses `RuntimeHarness`, writes raw JSON and summary JSON per scenario, checks the core remains alive after each scenario, writes the evidence manifest, runs `check_budget.py`, and tears down unconditionally. Before budget validation, inject `core_crash_count.count` into each summary from the runner-owned process check.
- [ ] Validate syntax and execute all scenarios:

```bash
k6 inspect -e AGILEPLUS_GRPC_ADDRESS=127.0.0.1:1 perf/smoke.js
k6 inspect -e AGILEPLUS_GRPC_ADDRESS=127.0.0.1:1 perf/load.js
k6 inspect -e AGILEPLUS_GRPC_ADDRESS=127.0.0.1:1 perf/stress.js
uv run --project python --locked python -m perf.runner --scenario all --output .perf-reports
```

- [ ] Commit: `perf(core): add bounded grpc scenarios`.

## Task 9: Hosted performance workflow

**Files:**

- Modify: `.github/workflows/perf.yml`

- [ ] Replace Node/Docker/Process Compose and missing `warmup.sh` with Python 3.14, locked uv, Rust setup/build, pinned k6 v0.50.0, and `python -m perf.runner --scenario all`.
- [ ] Always upload `.perf-reports/**`, runtime logs, candidate manifest, and budget result. Keep both perf job names and the hard gate unchanged.
- [ ] Validate k6 syntax against both local 1.6.1 and hosted 0.50.0-compatible APIs; hosted version is authoritative.
- [ ] Run `actionlint .github/workflows/perf.yml` and `git diff --check`.
- [ ] Commit: `ci(perf): run bounded grpc budgets`.

## Task 10: Review, push, hosted proof

- [ ] Run all targeted tests, then existing affected gates:

```bash
cargo test --locked -p agileplus-grpc
uv run --project python --locked pytest -c python/pyproject.toml -q qe perf/tests
uv run --project python --locked ruff check qe perf scripts/qe scripts/perf
actionlint .github/workflows/qe.yml .github/workflows/perf.yml
git diff --check
```

- [ ] Search the plan and implementation for placeholders and unsafe shortcuts:

```bash
rg -n 'TBD|TODO|similar to|appropriate|handle edge|localhost:50051|process-compose|docker compose' \
  docs/superpowers/plans/2026-09-01-core-mcp-qe-perf-harness.md qe perf scripts/qe scripts/perf .github/workflows/{qe,perf}.yml
```

Any intentional documentation/reference match must be reviewed; implementation matches must be removed or justified.

- [ ] Obtain independent specification-compliance and code-quality reviews. Fix findings and rerun affected tests.
- [ ] Push the branch normally, open a PR documenting local evidence and #1056 hosted failure boundaries, and wait for protected review/checks. Never admin-bypass.
- [ ] Require hosted QE to reach all real contract tests and hosted perf to produce all three reports within budgets. A queued or locally green run is not hosted success.
- [ ] Only after merge, run the installed MCP health/dashboard/governance/audit verification against the merged SHA and record immutable run URLs, merge SHA, artifact digests, and runtime evidence.
