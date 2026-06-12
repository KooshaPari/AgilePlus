# Contributing

Thanks for contributing to AgilePlus. This document captures the minimum process for changes that touch this repository.

## Workflow

1. Branch from `main`: `git checkout -b <type>/<short-topic> origin/main`.
2. Keep commits small and provenance-scoped (no omnibus commits).
3. Before opening a PR, run the local quality gate from the repo root:
   - `task quality` — boundaries, Vale on invariant Markdown, Ruff (`src/` + `tests/`), phenotype CLIProxy unit tests.
   - `task quality:full` — same plus `ruff format --check`.
4. Open a PR with a clear title (`<type>(scope): summary`) and link any related spec or worklog.

## Commit Types

`feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`, `build`, `ci`.

## Code Style

- Follow repo linters and formatters — do not bypass them.
- All Markdown must be UTF-8: `agileplus validate-encoding --all --fix` (run from the AgilePlus repo root).
- Specs and plans must not include code; write specs, acceptance criteria, and handoffs.

## Reporting Issues

File issues with a minimal reproduction, expected vs. actual behavior, and environment details.

## Code of Conduct

By participating, you agree to abide by the project's `CODE_OF_CONDUCT.md`.
