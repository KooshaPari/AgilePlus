# Batch 14 Repo Remediation

## Meta

- **ID**: 023-batch14-repo-remediation
- **Title**: Remediate Batch 14 Repos (bare-cua, colab, devenv-abstraction, clikit)
- **Created**: 2026-04-02
- **State**: specified
- **Scope**: Shelf-level (cross-repo)

## Context

Batch 14 audit (bare-cua, colab, devenv-abstraction, clikit) revealed:
- **bare-cua**: Rust project with good README, needs CI/CD, AgilePlus, CHANGELOG
- **colab**: Well-configured project (323 commits, 6 workflows), needs CHANGELOG, VERSION, AgilePlus
- **devenv-abstraction**: Has docs and workflows, needs CHANGELOG, VERSION, AgilePlus
- **clikit**: Empty directory, not a git repo, needs removal

## Problem Statement

Batch 14 repos have critical gaps:
- **bare-cua**: Only 1 commit, no CI, no AgilePlus, no CHANGELOG
- **colab**: 323 commits but missing CHANGELOG, VERSION
- **devenv-abstraction**: Missing CHANGELOG, VERSION, AgilePlus
- **clikit**: Empty, not a git repo, consuming space

## Goals

- Add CI/CD workflows to bare-cua
- Add CHANGELOG.md to bare-cua, colab, devenv-abstraction
- Add VERSION file to bare-cua, colab, devenv-abstraction
- Create AgilePlus scaffolding for bare-cua, colab, devenv-abstraction
- Remove clikit (empty directory)

## Repositories Affected

| Repo | Issues | Action |
|------|--------|--------|
| bare-cua | No CI, no AgilePlus, no CHANGELOG | Add CI, AgilePlus, CHANGELOG |
| colab | No CHANGELOG, no VERSION | Add CHANGELOG, VERSION |
| devenv-abstraction | No CHANGELOG, no VERSION, no AgilePlus | Add CHANGELOG, VERSION, AgilePlus |
| clikit | Empty, not git repo | Remove |

## Technical Approach

### Phase 1: Remove clikit
1. rmdir clikit (empty directory)

### Phase 2: Scaffold bare-cua
1. Create .github/workflows/ with Rust CI (fmt, clippy, test)
2. Create CHANGELOG.md
3. Create VERSION file
4. Create .agileplus/ with worklog.md

### Phase 3: Add CHANGELOG/VERSION to colab and devenv-abstraction
1. Create CHANGELOG.md for both
2. Create VERSION file for both
3. Add AgilePlus scaffolding if missing

## Success Criteria

- clikit removed
- bare-cua has CI/CD, CHANGELOG, VERSION, AgilePlus
- colab has CHANGELOG, VERSION
- devenv-abstraction has CHANGELOG, VERSION, AgilePlus

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking bare-cua build | Low | Test CI before committing |
| Over-scoping | Medium | Focus on scaffolding only |

## Work Packages

| ID | Description | State |
|----|-------------|-------|
| WP001 | Remove clikit | specified |
| WP002 | Scaffold bare-cua | specified |
| WP003 | Add CHANGELOG/VERSION to colab, devenv-abstraction | specified |

## Traces

- Related: 022-batch13-repo-remediation
- Related: SHELF_AUDIT_COMPLETE_2026-04-02
