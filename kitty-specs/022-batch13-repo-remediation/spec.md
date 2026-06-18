# Batch 13 Repo Remediation

## Meta

- **ID**: 022-batch13-repo-remediation
- **Title**: Remediate 7 Empty/Abandoned Repos from Batch 13 Audit
- **Created**: 2026-04-02
- **State**: specified
- **Scope**: Shelf-level (cross-repo)

## Context

The Batch 13 audit (KaskMan, dotfiles, harnesses, kits, packs, portage, zen) revealed 7 repos in critical condition:
- 5 repos are completely empty or near-empty (KaskMan, kits, packs, portage, zen)
- 2 repos have partial content but lack basic scaffolding (dotfiles, harnesses)
- Zero repos have README.md, CI/CD, AgilePlus specs, or changelogs
- 1 repo (kits) is not even a git repository
- All repos share the same orphaned worktree branch (chore/gitignore-and-test-infra)

## Problem Statement

Batch 13 repos are abandoned and consuming shelf space:
- **Empty repos** — 5 repos with zero meaningful content
- **No documentation** — Zero README files across all 7 repos
- **No CI/CD** — Zero GitHub Actions workflows
- **No AgilePlus** — Zero specs, zero worklogs
- **Orphaned worktrees** — All share same stale branch with deleted archive files
- **kits not a git repo** — Completely empty directory, not initialized

## Goals

- Archive 5 empty repos (KaskMan, kits, packs, portage, zen) to .archive/
- Populate dotfiles with README.md, .gitignore, .agileplus/, basic scaffolding
- Populate harnesses with README.md, .gitignore, .agileplus/, basic scaffolding
- Update projects/INDEX.md to reflect archival status
- Write worklog entry documenting the remediation

## Non-Goals

- Creating new functionality in dotfiles or harnesses
- Migrating content between repos
- Setting up CI/CD workflows (future spec)
- Creating GitHub Pages for any repo

## Repositories Affected

| Repo | Action | Rationale |
|------|--------|-----------|
| KaskMan | Archive | Empty directory, no content |
| kits | Archive | Not a git repo, empty directory |
| packs | Archive | Empty directory, warfare-starwars already archived |
| portage | Archive | Empty directory |
| zen | Archive | Empty directory |
| dotfiles | Populate | Has governance/ and hooks/ content, needs scaffolding |
| harnesses | Populate | Has agent config files, needs scaffolding |

## Technical Approach

### Phase 1: Archive Empty Repos
1. Move KaskMan, kits, packs, portage, zen to .archive/
2. Update projects/INDEX.md to mark as archived
3. Remove orphaned worktree references

### Phase 2: Populate dotfiles
1. Create README.md with project description and structure
2. Create .gitignore for dotfiles/governance/hooks patterns
3. Create .agileplus/ directory with worklog.md
4. Create AGENTS.md for agent guidance

### Phase 3: Populate harnesses
1. Create README.md explaining harness purpose
2. Create .gitignore for harness patterns
3. Create .agileplus/ directory with worklog.md
4. Create AGENTS.md for agent guidance

### Phase 4: Documentation
1. Update projects/INDEX.md with archival status
2. Write worklog entry at worklogs/GOVERNANCE.md
3. Update shelf-level docs if needed

## Success Criteria

- 5 empty repos moved to .archive/
- dotfiles has README.md, .gitignore, .agileplus/, AGENTS.md
- harnesses has README.md, .gitignore, .agileplus/, AGENTS.md
- projects/INDEX.md updated with archival status
- Worklog entry written

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Losing worktree references | Low | Document before archival |
| Breaking existing references | Low | Update INDEX.md |
| Over-scoping dotfiles/harnesses | Medium | Strict scope: scaffolding only |

## Work Packages

| ID | Description | State |
|----|-------------|-------|
| WP001 | Archive empty repos (KaskMan, kits, packs, portage, zen) | specified |
| WP002 | Populate dotfiles scaffolding | specified |
| WP003 | Populate harnesses scaffolding | specified |
| WP004 | Update INDEX.md and write worklog | specified |

## Traces

- Related: 021-polyrepo-ecosystem-stabilization
- Related: 018-template-repo-cleanup
- Related: 012-github-portfolio-triage
