---
id: FR-AGILE-017
title: CLI Tools Consolidation
status: specified
priority: P1
created: 2026-04-01
category: cli
owner: phenotype-org
source: kitty-specs/017-cli-tools-consolidation
---

# FR-AGILE-017: CLI Tools Consolidation

## Description

Consolidate 7 CLI-related repositories with overlapping functionality into a unified CLI ecosystem.

## Repositories

- cliproxyapi-plusplus (LLM proxy)
- agentapi-plusplus (Agent API)
- Cmdra (CLI framework)
- forgecode (Git workflows)
- thegent-sharecli / thegent-cli-share (deduplicate)
- thegent-subprocess


## User Stories

### US-1: Developer Experience (P1)
**Given** a developer using the system,
**When** they perform core operations,
**Then** they receive consistent, predictable behavior with proper feedback.

### US-2: Integration Scenario (P1)
**Given** the component is integrated with the ecosystem,
**When** data flows through the system,
**Then** all traceability and governance requirements are met.

## Acceptance Criteria

- [ ] cliproxyapi-plusplus: 8+ provider support
- [ ] agentapi-plusplus: HTTP API complete
- [ ] Cmdra: Universal CLI framework
- [ ] forgecode: Git workflow framework
- [ ] Deduplicate sharecli variants
- [ ] thegent-subprocess: Subprocess management

## Dependencies

- FR-AGILE-006 (HeliosCLI)
- FR-AGILE-007 (thegent)
- FR-AGILE-013 (infrakit)

## Notes

Original: `kitty-specs/017-cli-tools-consolidation/`
