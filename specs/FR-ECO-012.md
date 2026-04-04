---
id: FR-ECO-012
title: OrgOps Capital Ledger & Resource Management
status: specified
priority: P2
created: 2026-03-31
category: infrastructure
owner: phenotype-org
source: kitty-specs/eco-012-orgops-capital-ledger
---

# FR-ECO-012: OrgOps Capital Ledger & Resource Management

## Description

Centralized resource management for Phenotype agents: API keys, cloud credits, free-tier accounts, browser authentication sessions across 8+ repositories. Track consumption, manage secret rotation, persist profiles.

## Problem Statement

1. **No resource awareness** — agents don't know what LLM subscriptions, cloud credits, or free-tier accounts are available
2. **Secret fragility** — authkit keys expire every 5 minutes, agents bypass ggshield, rotation is manual
3. **No consumption tracking** — token budgets, API calls, compute hours untracked
4. **No profile persistence** — browser user agent profiles lose auth state between sessions
5. **Worktree chaos** — no systematic gix-based worktree management

## Functional Requirements

### FR-CAP-001: Capital Registry (Org Level)
Parse `repos/capital.toml` to build registry of organizational resources: LLM accounts, cloud credits, free-tier accounts, API keys, browser profiles.

### FR-CAP-002: Resource Allocation (Project Level)
Projects declare resource needs via `.agileplus/capital.toml`; track allocations vs. capacity.

### FR-CAP-003: Consumption Tracking
Record token usage, API calls, compute hours per agent session in `.agileplus/capital.db`.

### FR-CAP-004: Budget Enforcement
Per-project and per-agent budget limits. Return `BudgetExceeded` error when exceeded.

### FR-SEC-001: Secret Inventory
Maintain inventory of all secrets with metadata: env_var name, rotation interval, last validated, freshness status.

### FR-SEC-002: Secret Validation
Automated secret freshness validation before agent sessions start.

## Acceptance Criteria

- [ ] Capital registry parsing implemented
- [ ] Resource allocation tracking working
- [ ] Consumption database operational
- [ ] Budget enforcement active
- [ ] Secret inventory complete
- [ ] Automated validation in CI

## Notes

Original: `kitty-specs/eco-012-orgops-capital-ledger/`
