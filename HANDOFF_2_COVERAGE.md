# HANDOFF 2 — Test Coverage Push (22% -> 85%)
**Date:** 2026-09-16
**Repo:** agileplus (~/CodeProjects/Phenotype/repos/AgilePlus)
**Branch:** main
**Version:** v0.2.4

## Current State
- **22.44% line coverage** (33,456 total lines, 25,947 missed)
- Target: **85%** = need to cover ~20,929 more lines
- Coverage measured via `cargo llvm-cov --workspace --summary-only` (takes ~8-10 min)

## Coverage Already Committed (this session)
| Crate | Before | After | Lines Added |
|-------|--------|-------|-------------|
| traceability-core | 4.1% | 67.2% | +647 |
| events | 8.6% | 58.5% | +208 |
| governance | low | has 238 tests | committed |
| triage | low | has 83 tests | committed |
| application | low | has 31 tests | committed |
| cli | 12.7% | 14.8% | +145 |
| dashboard | low | +700 lines | committed |
| plane | low | 30+ tests | committed |
| api | low | tests across 15 files | committed |

## Biggest Targets (by estimated uncovered lines)
These crates have the most source code and lowest coverage — prioritize here:

### Tier 1: Large crates, very low coverage (~5000+ missed lines each)
1. **agileplus-sqlite** — massive repository layer (work_packages, features, cycles, modules, backlog, sync_mappings, users)
2. **traceability-core** — intent_graph.rs (556 lines), impact.rs (374 lines), matrix.rs (211 lines) still have gaps
3. **agileplus-plane** — daemon, labels, outbound, runtime, sync, client modules
4. **agileplus-domain** — cycle, work_package, ids modules
5. **agileplus-cli** — many commands (dag, mvp, repl, worklog, specify, plan, retrospective, implement)

### Tier 2: Medium crates, very low coverage (~1000-3000 missed lines each)
6. **agileplus-application** — use_cases, dto modules
7. **agileplus-graph** — graph operations
8. **agileplus-events** — event processing
9. **agileplus-dashboard** — routes (dashboard, settings, features, health)
10. **agileplus-git** — lib, materialize, integration

### Tier 3: Smaller crates, zero/minimal coverage
11. **agileplus-github** — GitHub API integration
12. **agileplus-governance** — scoring_engine (13 functions)
13. **agileplus-telemetry** — adapter, logs, metrics, traces
14. **agileplus-import** — import logic
15. **agileplus-sync** — sync operations
16. **agileplus-p2p** — export, git_merge, import
17. **agileplus-nats** — bus module
18. **agileplus-grpc** — server module
19. **agileplus-api-types** — zero tests
20. **agileplus-error-core** — zero tests
21. **agileplus-config** — configuration

## CRITICAL RULES FOR WORKERS
- **NEVER modify production source files** — only create new test files or append `#[cfg(test)]` blocks at END of files
- **Tests must verify real behavior** — no mocked abstractions
- Use `#[cfg(test)] mod tests { ... }` blocks at bottom of source files, or separate `tests/` directory
- Run `cargo test -p <crate-name>` to verify tests pass before committing
- Workers that modify production code must be reverted immediately
- Worker model: `opencode-go/mimo-v2.5` (note: ~50% failure rate on API calls)

## Coverage Measurement
```bash
# Full workspace coverage (takes ~10 min)
cargo llvm-cov --workspace --summary-only

# Single crate (faster)
cargo llvm-cov -p agileplus-sqlite --summary-only
```

## Math: Path to 85%
- Current: 22.44% (7,509 covered / 33,456 total)
- Target: 85% (28,438 covered / 33,456 total)
- Need: +20,929 covered lines
- If each worker adds ~200-500 lines of tests: need ~42-105 worker tasks
- With 8 workers parallel: 6-14 rounds of workers

## Session docs
- `docs/sessions/` — session artifacts
