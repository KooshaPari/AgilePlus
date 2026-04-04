---
id: FR-AGILE-002
title: Org-Wide Release Governance & DX Automation
status: draft
priority: P0
created: 2026-03-01
category: platform
owner: phenotype-org
source: kitty-specs/002-org-wide-release-governance-dx-automation
---

# FR-AGILE-002: Org-Wide Release Governance & DX Automation

## Description

Implement comprehensive DX infrastructure and release governance automation across all 47 Phenotype repositories. Covers quality gates, automated versioning, GitHub Actions publishing workflows (npm, PyPI, crates.io), pre-commit/pre-push hooks, Taskfile/CLI DX commands, and 5-tier release channel governance (alpha → canary → beta → rc → prod).

## Context

- 47 repositories spanning Rust, Python, TypeScript, Go, Elixir
- 5-tier release channel governance exists as documentation but only implemented in 3 repos
- Publishing to registries is currently manual
- Only 9/47 repos have pre-commit hooks
- 31/47 have Taskfile.yml but with no standardized targets
- No cross-repo orchestration CLI exists

## Objectives

- Implement automated quality gates for all releases
- Enforce semantic versioning across polyrepo ecosystem
- Provide release orchestration with rollback capability
- Enable developer experience automation (lint, format, test, build)
- Support policy-driven governance
- Automated publishing to npm, PyPI, crates.io

## Acceptance Criteria

- [ ] Quality gate system with configurable checks
- [ ] Automated semantic versioning based on conventional commits
- [ ] Cross-repo release coordination
- [ ] Reusable GitHub Actions workflows for all registries
- [ ] 5-tier release channel automation (alpha → canary → beta → rc → prod)
- [ ] Pre-commit/pre-push hooks standardized across all repos
- [ ] Standardized Taskfile.yml with consistent targets
- [ ] Cross-repo orchestration CLI for bulk operations

## User Stories

### US-1: Pre-Release Publishing (P1)
**Given** a Rust crate at version `0.2.0` on the `alpha` channel,  
**When** developer triggers channel promotion,  
**Then** version `0.2.0-alpha.1` publishes to crates.io automatically.

### US-2: Standardized DX Commands (P1)
**Given** a developer clones any Phenotype repo,  
**When** they run `task build` or `task test`,  
**Then** consistent commands work regardless of language/build system.

## Dependencies

- FR-AGILE-001: Spec-Driven Development Engine
- FR-ECO-006: Governance Sync

## Notes

Original: `kitty-specs/002-org-wide-release-governance-dx-automation/`

- [ ] Rollback mechanism for failed releases
- [ ] Developer experience automation in CI/CD
- [ ] Governance policy enforcement
- [ ] Audit trail for all release decisions

## Work Packages

| WP | Title | Repository | Status |
|----|-------|------------|--------|
| WP-001 | Quality Gate Framework | quality-gate | planned |
| WP-002 | Release Orchestrator | release-orchestrator | planned |
| WP-003 | DX Automation Scripts | dx-automation | planned |
| WP-004 | Governance Policy Engine | governance-engine | planned |

## Dependencies

- FR-AGILE-001 (AgilePlus Core)
- GitHub/GitLab API
- CI/CD infrastructure

## Traceability

- Test Framework: pytest (Python), Rust test
- Coverage Target: ≥85%

## Notes

Original: `kitty-specs/002-org-wide-release-governance-dx-automation/`
