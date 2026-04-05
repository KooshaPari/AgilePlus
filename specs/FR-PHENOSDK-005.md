---
id: FR-PHENOSDK-005
title: phenoSDK MCP Package Extraction
status: specified
priority: P1
created: 2026-03-25
category: sdk
owner: phenosdk-team
source: kitty-specs/phenosdk-decompose-mcp
---

# FR-PHENOSDK-005: phenoSDK MCP Package Extraction

## Description

Extract pheno-mcp package from monolith (104 files). MCP tooling layer — FastMCP wrappers, tool registry, agent orchestration — as standalone package.

## Problem

pheno/mcp is atoms-specific and embedded in monolith. Should be usable by any MCP server.

## Key Extractions

- `pheno/mcp/tools/decorators.py` (349 LOC)
- `pheno/mcp/agents/orchestration.py` (372 LOC)
- `pheno/mcp/entry_points.py` (generalized)


## User Stories

### US-1: SDK Integration (P0)
**Given** a developer integrating phenoSDK,
**When** they use SDK features,
**Then** the SDK provides consistent, well-documented interfaces.

### US-2: SDK Reliability (P1)
**Given** a production system using phenoSDK,
**When** SDK operations are performed,
**Then** they complete successfully without NotImplementedError.

## Acceptance Criteria

- [ ] pheno-mcp package with mcp/, tools/, agents/ modules
- [ ] No atoms-specific references
- [ ] FastMCP integration abstracted via port
- [ ] CrewAI adapter generalized
- [ ] pheno-mcp depends on pheno-core only
- [ ] Integration tests with mock MCP server
- [ ] Published to Phenotype GitHub Packages

## Notes

Original: `kitty-specs/phenosdk-decompose-mcp/`
