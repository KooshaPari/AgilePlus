---
id: FR-HELIOSCLI-001
title: heliosCLI Multi-Runtime Agent CLI Completion
status: specified
priority: P1
created: 2026-03-25
category: cli
owner: helioscli-team
source: kitty-specs/006-helioscli-completion
---

# FR-HELIOSCLI-001: heliosCLI Multi-Runtime Agent CLI Completion

## Description

Complete heliosCLI multi-runtime AI coding CLI (153 commits since 2025-01-01). Rust core with TypeScript CLI wrapper, Bazel monorepo build system, and thegent integration.

## Architecture

- **codex-rs**: Rust core runtime
- **codex-cli**: TypeScript CLI interface
- **Bazel monorepo**: Build and test system
- **thegent integration**: Agent orchestration layer

## Completed Work

- WP001: Expect Pattern Cleanup (shipped)
- Core Rust runtime
- TypeScript CLI wrapper

## Present Work

- WP010: Bazel Build Optimization
  - Build caching
  - Remote execution
  - Incremental builds

## Future Work

- WP020: Multi-Runtime Integration
  - Codex runtime
  - Claude runtime
  - Pluggable runtime architecture

## Acceptance Criteria

- [ ] Bazel build optimization complete
- [ ] Multi-runtime integration working
- [ ] Codex and Claude runtimes functional
- [ ] Agent orchestration via thegent
- [ ] CLI stable for daily use

## Repository

`/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI`

## Notes

Original: `kitty-specs/006-helioscli-completion/`
