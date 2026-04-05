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

## User Stories

### US-1: Developer Initializes Project (P0)
**Given** a developer with a new project idea,  
**When** they run `agileplus init`,  
**Then** a git+SQLite project structure is created with CLAUDE.md and AGENTS.md templates.

### US-2: Spec Creation with Agent Assistance (P0)
**Given** a developer wants to capture requirements,  
**When** they use natural language to describe the feature,  
**Then** AgilePlus generates a structured FR spec with acceptance criteria and work packages.

### US-3: Plan Generation with Work Packages (P1)
**Given** an approved FR spec,  
**When** the developer runs `agileplus plan`,  
**Then** work packages are created with estimates, dependencies, and assigned repositories.

### US-4: Evidence-Backed State Transitions (P1)
**Given** a work package in "implementing" state,  
**When** the developer provides commit evidence,  
**Then** the state transitions to "verify" with hash-chained audit log entry.

### US-5: Cross-Repo Coordination (P1)
**Given** a feature spanning multiple repositories,  
**When** work packages are assigned to different repos,  
**Then** gRPC communication ensures synchronized state across the ecosystem.

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
