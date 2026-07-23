# phench Implementation Plan

**Status:** Active
**Stack:** Python 3.11+, typer, rich, orjson

## Phase 1: Core CLI

| Task | Description | Depends On |
|------|-------------|------------|
| P1.1 | CLI scaffold with typer + subcommand groups | -- |
| P1.2 | Target init: create `.phench/<name>/` metadata | P1.1 |
| P1.3 | Repo binding: add-repo with ref validation | P1.2 |
| P1.4 | Target lock: generate JSON lockfile from repo SHAs | P1.3 |

## Phase 2: Materialization

| Task | Description | Depends On |
|------|-------------|------------|
| P2.1 | Precondition checker (disk, git, repo access) | P1.1 |
| P2.2 | Materialization engine: clone repos at locked SHAs | P1.4, P2.1 |
| P2.3 | Dual-store sync (`.phench/` <-> `~/phench/`) | P1.2 |

## Phase 3: Runtime Operations

| Task | Description | Depends On |
|------|-------------|------------|
| P3.1 | Status command with rich formatted output | P1.4 |
| P3.2 | Timeline event log and display | P1.2 |
| P3.3 | Sync command: pull latest refs, update lock | P1.4 |
| P3.4 | Run command: execute in target context | P2.2 |
| P3.5 | Env doctor: prerequisite health checks | P2.1 |

## Phase 4: Quality

| Task | Description | Depends On |
|------|-------------|------------|
| P4.1 | Unit tests for target lifecycle | P1.4 |
| P4.2 | Integration tests for materialization | P2.2 |
| P4.3 | CI pipeline with ruff + pytest | P4.1 |
