---
work_package_id: WP20
title: 'portage + trace: Code Quality Remediation'
lane: planned
dependencies: []
subtasks: [T059, T060, T061, T062]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
---

# WP20: portage + trace — Code Quality Remediation

**Implementation command**: `spec-kitty implement WP20`

## Objective

Remediate the top code quality issues in portage and trace: extract a shared BaseAdapter from 8 duplicate definitions in portage, decompose god functions in both repos, split trace's 9,274-line god module, remove deprecated dependencies, and clean up Python type debt. This WP has no dependencies and can proceed in parallel with other work packages.

## Context

- portage has 103K Python LOC with 8 independently defined BaseAdapter classes sharing identical method signatures
- portage's `create_app` (1,164 lines) and `jobs.py::start` (856 lines) are god functions
- trace has 678K LOC (Go+Python+TS) with `api/main.py` at 9,274 lines — the most severe god module in the entire ecosystem
- trace uses gorilla/mux (archived/deprecated) alongside echo — redundant routers
- trace has 390 Python type suppressions (`type: ignore` + `noqa`) indicating retrofitted typing
- trace has 5 `.go.bak` dead backup files in handlers/

## Subtasks

### T059: portage — Extract shared BaseAdapter

**Steps**:
1. Audit all 8 independent BaseAdapter definitions across the 41 adapters:
   - Document the method signatures each defines: `run`, `score`, `name`, and any additional methods
   - Identify the `RequireNameMeta` metaclass duplicated across at least 4 adapters
   - Catalog any per-adapter deviations from the common interface
2. Create `src/harbor/adapters/base.py` with canonical definitions:
   - `BaseAdapter` abstract class with standard methods: `run()`, `score()`, `name()`
   - `RequireNameMeta` metaclass (single definition)
   - Common adapter utilities: shared scoring patterns, result formatting helpers
3. Refactor all 8 adapters to import from `src/harbor/adapters/base.py`:
   - Remove local BaseAdapter class definitions
   - Remove local RequireNameMeta definitions
   - Update imports
4. Extract additional shared adapter utilities where common patterns exist

**Validation**:
- `grep -r "class BaseAdapter" adapters/` returns 0 results (all use shared import)
- `grep -r "RequireNameMeta" adapters/` shows only imports, no definitions
- All 41 adapters import from the shared base
- All tests pass

### T060: portage — Split god functions

**Steps**:
1. Decompose `server.py::create_app` (1,164 lines):
   - `server.py` — app factory and setup (<200 lines)
   - `routes.py` — route registration and URL mapping
   - `handlers.py` — request handler implementations
   - `middleware.py` — middleware chain configuration
2. Decompose `jobs.py::start` (856 lines):
   - `jobs.py` — job orchestration and scheduling (<200 lines)
   - `job_runner.py` — individual job execution logic
   - `job_config.py` — job configuration parsing and validation
3. Split `terminus_2.py` (1,838 lines):
   - `terminus_parser.py` — input parsing and tokenization
   - `terminus_renderer.py` — output rendering and formatting
   - `terminus_state.py` — state machine and lifecycle management

**Validation**:
- No function exceeds 200 lines in the split files
- `create_app` factory body is <200 lines
- `jobs.py::start` orchestration body is <200 lines
- All tests pass
- No behavioral changes — identical API surface

### T061: trace — Split api/main.py (9,274 lines) + remove deprecated deps

**Steps**:
1. Decompose `api/main.py` (9,274 lines — most severe god module in ecosystem):
   - `main.py` — app setup and entry point only (<200 lines)
   - `routers/` — route handlers (partially exists, complete the extraction)
   - `middleware/` — middleware definitions and configuration
   - `startup.py` — initialization, database connections, service registration
2. Remove gorilla/mux (archived/deprecated):
   - Identify all routes currently using gorilla/mux
   - Migrate to echo (already in use for other routes)
   - Remove gorilla/mux from `go.mod` and `go.sum`
3. Consolidate pgxmock versions:
   - Remove pgxmock v3 dependency
   - Standardize all test files on pgxmock v4
   - Update import paths in affected test files
4. Delete 5 `.go.bak` dead backup files in `handlers/`:
   - Verify they are not referenced anywhere
   - Remove from repository

**Validation**:
- `api/main.py` is <500 LOC after split
- `go mod tidy` shows no gorilla/mux in dependency tree
- `grep -r "pgxmock/v3" --include='*.go'` returns 0 results
- No `.go.bak` files remain in handlers/
- `go build ./...` succeeds
- All Go and Python tests pass

### T062: trace — Clean Python type debt (390 suppressions)

**Steps**:
1. Catalog all 390 `type: ignore` and `noqa` suppressions in `src/`:
   - Group by file (identify files with highest concentration)
   - Categorize by suppression type (missing types, incompatible types, import errors, etc.)
2. Fix the top 50 suppressions by file concentration:
   - Add proper type annotations where suppressions hide missing types
   - Fix incompatible type assignments with proper casting or interface changes
   - Resolve import-related suppressions by fixing the underlying import structure
3. For suppressions that remain, add specific error codes and justification:
   - Change bare `# type: ignore` to `# type: ignore[specific-error] -- reason`
   - Change bare `# noqa` to `# noqa: EXXX -- reason`

**Validation**:
- Total suppression count reduced from 390 to <100
- All remaining suppressions have specific error codes and inline justification comments
- No bare `# type: ignore` or `# noqa` without error codes
- Type checking passes (`mypy` or `pyright`)
- All tests pass

## Definition of Done

- [ ] Shared BaseAdapter extracted in portage — 0 local definitions remain
- [ ] portage god functions decomposed — no function >200 lines
- [ ] trace api/main.py split from 9,274 to <500 LOC
- [ ] gorilla/mux removed, all routing on echo
- [ ] pgxmock consolidated on v4
- [ ] 5 .go.bak files deleted
- [ ] Python type suppressions reduced from 390 to <100, all justified
- [ ] All tests pass in both repos

## Risks

- **BaseAdapter extraction may reveal adapter-specific quirks**: Some adapters may have overloaded or extended the common interface in incompatible ways. Allow the shared trait to support optional extension methods.
- **api/main.py split may break initialization order**: A 9,274-line file likely has implicit ordering dependencies. Map the initialization sequence before splitting.
- **gorilla/mux removal may affect middleware chains**: gorilla/mux middleware and echo middleware have different signatures. Verify middleware compatibility before migration.
- **Type suppression removal may surface real type errors**: Some suppressions may mask genuine type incompatibilities that require interface changes to fix. Budget time for actual type fixes, not just annotation additions.

## Reviewer Guidance

- Verify the shared BaseAdapter captures all methods used by the 8 original definitions, not just the common subset
- Check that create_app split preserves the exact middleware ordering
- Confirm api/main.py split maintains the startup initialization sequence
- Verify gorilla/mux removal does not drop any routes — compare route count before and after
- Check that type suppression removals include actual type fixes, not just suppression replacement
- Verify .go.bak files are truly unreferenced before deletion
