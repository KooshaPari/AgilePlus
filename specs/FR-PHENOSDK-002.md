---
id: FR-PHENOSDK-002
title: Sanitize Atoms Identifiers
status: specified
priority: P1
created: 2026-04-01
category: sdk
owner: phenotype-org
source: kitty-specs/phenosdk-sanitize-atoms
---

# FR-PHENOSDK-002: Sanitize Atoms Identifiers

## Description

Remove atoms.tech school capstone identifiers from phenoSDK code for open-source release readiness.


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

- [ ] All `Atoms` / `ATOMS` / `atoms` identifiers replaced
- [ ] pyproject.toml: author changed to Phenotype org
- [ ] ATOMS_MCP_RISK_ASSESSMENT.md reviewed/removed
- [ ] No atoms.tech domain references
- [ ] Tests pass after rename

## Scope

- src/pheno/mcp/entry_points.py
- src/pheno/shared/mcp_entry_points.py
- pyproject.toml

## Notes

Source: `kitty-specs/phenosdk-sanitize-atoms`
Repository: phenoSDK
