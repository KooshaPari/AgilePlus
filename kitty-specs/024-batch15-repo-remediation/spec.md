# Batch 15 Repo Remediation

## Meta

- **ID**: 024-batch15-repo-remediation
- **Title**: Remediate Batch 15 Repos (AgentMCP, BytePort, Httpora)
- **Created**: 2026-04-02
- **State**: specified
- **Scope**: Shelf-level (cross-repo)

## Context

Batch 15 audit revealed:
- **AgentMCP**: Python MCP server, missing README.md
- **BytePort**: Missing CHANGELOG.md, VERSION, AgilePlus
- **Httpora**: Missing CHANGELOG.md, VERSION, AgilePlus, README.md, docs/

## Problem Statement

Batch 15 repos have critical scaffolding gaps:
- **AgentMCP**: No README.md despite being an active project
- **BytePort**: No CHANGELOG, VERSION, or AgilePlus
- **Httpora**: No CHANGELOG, VERSION, README, docs, or AgilePlus

## Goals

- Add README.md to AgentMCP
- Add CHANGELOG.md and VERSION to BytePort
- Add CHANGELOG.md, VERSION, README.md to Httpora
- Add AgilePlus scaffolding to all three

## Repositories Affected

| Repo | Issues | Action |
|------|--------|--------|
| AgentMCP | No README.md | Add README.md |
| BytePort | No CHANGELOG, VERSION, AgilePlus | Add all |
| Httpora | No CHANGELOG, VERSION, README, docs, AgilePlus | Add all |

## Technical Approach

### Phase 1: Add README.md to AgentMCP
1. Create README.md based on pyproject.toml description

### Phase 2: Add CHANGELOG/VERSION to BytePort and Httpora
1. Create CHANGELOG.md with initial release notes
2. Create VERSION file

### Phase 3: Add AgilePlus scaffolding
1. Create .agileplus/worklog.md for all three

## Success Criteria

- AgentMCP has README.md
- BytePort has CHANGELOG.md, VERSION, AgilePlus
- Httpora has README.md, CHANGELOG.md, VERSION, AgilePlus

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Incorrect version | Low | Use project file version |
| Over-scoping | Medium | Focus on scaffolding only |

## Work Packages

| ID | Description | State |
|----|-------------|-------|
| WP001 | Add README to AgentMCP | specified |
| WP002 | Add CHANGELOG/VERSION to BytePort, Httpora | specified |
| WP003 | Add AgilePlus scaffolding to all | specified |

## Traces

- Related: 023-batch14-repo-remediation
- Related: SHELF_AUDIT_COMPLETE_2026-04-02
