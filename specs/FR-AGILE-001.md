---
id: FR-AGILE-001
title: Spec-Driven Development Engine
status: draft
priority: P0
created: 2026-02-27
category: platform
owner: phenotype-org
source: kitty-specs/001-spec-driven-development-engine
---

# FR-AGILE-001: Spec-Driven Development Engine

## Description

AgilePlus is a local, git+SQLite-backed spec-driven development engine that runs as a CLI sidecar alongside Claude Code and Codex. It harmonizes the best of OpenSpec (simplicity), spec-kitty (structured granularity, worktree isolation), bmad (enterprise depth), and GSD (automation) into a streamlined workflow.

## Objectives

- Provide 7-command workflow for spec-driven development
- Enable agent-first architecture through MCP primitives
- Support multi-repo architecture with CLEAN/SOLID/Hexagonal boundaries
- Implement governance with smart contract system for evidence-backed transitions
- Maintain hash-chained audit logs and policy-driven quality gates

## Acceptance Criteria

- [ ] CLI with 7 primary commands: `init`, `specify`, `plan`, `implement`, `verify`, `deliver`, `archive`
- [ ] MCP server (FastMCP 3.0) for agent integration
- [ ] SQLite source of truth for operational state
- [ ] Git source of truth for all artifacts
- [ ] Plane.so bidirectional sync for visual PM
- [ ] Prompt router pattern with CLAUDE.md and AGENTS.md generation
- [ ] Cross-repo gRPC communication via agileplus-proto

## Work Packages

| WP | Title | Repository | Status |
|----|-------|------------|--------|
| WP-001 | Core Domain & CLI | agileplus-core | planned |
| WP-002 | MCP Server | agileplus-mcp | planned |
| WP-003 | Agent Dispatch | agileplus-agents | planned |
| WP-004 | Plane.so Integration | agileplus-integrations | planned |
| WP-005 | Proto Contracts | agileplus-proto | planned |

## Dependencies

- Rust toolchain
- SQLite
- gRPC
- FastMCP 3.0
- Plane.so API

## Traceability

- Test Framework: Rust built-in test
- Coverage Target: ≥80%
- Trace to: All WP test suites

## Notes

Original: `kitty-specs/001-spec-driven-development-engine/`
