---
id: FR-CRATES-001
title: Phenotype Crates Ecosystem Adoption
status: specified
priority: P1
created: 2026-04-04
category: ecosystem
owner: phenotype-org
source: kitty-specs/017-phenotype-crates-ecosystem-adoption
---

# FR-CRATES-001: Phenotype Crates Ecosystem Adoption

## Description

Drive adoption of phenotype foundational crates across all Rust projects in the ecosystem. 50+ crates provide infrastructure services that are being duplicated.

## Context

Phenotype ecosystem has 50+ foundational crates in `/repos/crates/`:
- logging (phenotype-logging)
- metrics (phenotype-metrics)
- config (phenotype-config-core)
- validation (phenotype-validation)
- error handling (phenotype-error-core)
- async traits (phenotype-async-traits)
- contracts (phenotype-contracts)

## Problem Statement

Projects not using phenotype crates:
- bare-cua: Custom config, validation, error handling
- heliosCLI: Custom telemetry, metrics, health checks
- HexaKit: Custom contracts, port traits
- agentapi-plusplus: Custom error types, async patterns

## Goals

- Audit current duplication across projects
- Migrate projects to phenotype crates
- Document adoption patterns
- Establish shared infrastructure benefits


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

- [ ] bare-cua migrated to phenotype crates
- [ ] heliosCLI using phenotype telemetry
- [ ] HexaKit using phenotype contracts
- [ ] Adoption guide published
- [ ] ROI metrics documented

## Notes

Original: `kitty-specs/017-phenotype-crates-ecosystem-adoption/`
