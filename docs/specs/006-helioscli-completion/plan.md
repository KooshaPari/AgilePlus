# Implementation Plan: heliosCLI Multi-Runtime Agent CLI Completion

**Spec**: [spec.md](spec.md) | **Date**: 2026-03-25 | **Branch**: `006-helioscli-completion`

## Summary

Complete the heliosCLI multi-runtime AI coding CLI by finishing Bazel build optimization, integrating all supported runtimes (Codex, Claude, Gemini, Cursor, Copilot), and enabling full thegent orchestration with agent lifecycle management.

## Technical Context

**Language/Version**: Rust (codex-rs core) + TypeScript (codex-cli) + Bazel (build)
**Primary Dependencies**: codex-rs, codex-cli, Bazel, thegent, tokio, clap
**Storage**: Filesystem (local agent state)
**Testing**: cargo test, pnpm test, Bazel test targets
**Target Platform**: macOS (primary), Linux (CI), Windows (cross-platform)
**Performance Goals**: CLI startup <100ms, runtime dispatch <50ms, agent spawn <200ms

## Phased WBS

| Phase | WP | Description | Depends On |
|---|---|---|---|
| Build | WP-010 | Bazel build optimization — caching, remote execution, incremental builds | — |
| Integration | WP-020 | Multi-runtime integration — Codex, Claude, Gemini, Cursor, Copilot runtimes | WP-010 |
| Orchestration | WP-021 | thegent orchestration — full integration, agent lifecycle management | WP-020 |
| Validate | WP-022 | End-to-end runtime dispatch tests across all 5 runtimes | WP-020 |
| Validate | WP-023 | thegent agent lifecycle validation — spawn, dispatch, collect, retire | WP-021 |

## Verification Criteria

- All Bazel targets pass with remote caching enabled
- Each runtime dispatches correctly via the unified CLI interface
- thegent agent lifecycle (spawn → dispatch → collect → retire) completes without error
- CLI startup remains under 100ms after all integrations land
