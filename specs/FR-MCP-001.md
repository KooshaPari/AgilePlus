---
id: FR-MCP-001
title: MCP Server Alignment — Unify helios-mcp-server with phenotype-mcp-*
status: specified
priority: P1
created: 2026-04-04
category: integration
owner: phenotype-org
source: kitty-specs/018-mcp-server-alignment
---

# FR-MCP-001: MCP Server Alignment

## Description

Unify helios-mcp-server with phenotype-mcp-* crates to eliminate duplication and ensure compatibility.

## Context

heliosCLI has `helios-mcp-server` for MCP functionality. Phenotype crates have `phenotype-mcp-core`, `phenotype-mcp-asset`, `phenotype-mcp-testing`.

## Problem Statement

- Type duplication between implementations
- Incompatible JSON serialization
- Wasted maintenance on similar code
- Integration friction between systems

## Goals

- Audit current MCP implementations
- Extract shared types to unified crate
- Establish `phenotype-mcp-core` as canonical
- Align helios-mcp-server on phenotype types


## User Stories

### US-1: Core Functionality (P1)
**Given** a user of the system,
**When** they interact with this feature,
**Then** the system behaves as specified with proper traceability.

### US-2: Integration Scenario (P2)
**Given** the component is part of the ecosystem,
**When** integrated with other components,
**Then** it maintains FR traceability and governance compliance.

## Acceptance Criteria

- [ ] MCP duplication audit complete
- [ ] Shared types extracted to phenotype-mcp-core
- [ ] helios-mcp-server aligned on phenotype types
- [ ] Compatibility tests passing
- [ ] Migration guide published

## Notes

Original: `kitty-specs/018-mcp-server-alignment/`
