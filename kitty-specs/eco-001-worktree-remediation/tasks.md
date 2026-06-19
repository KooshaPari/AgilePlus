# Tasks: eco-001 — Worktree Remediation

**Status**: COMPLETED ✅

## Work Packages

| ID | Description | Status |
|----|-------------|--------|
| WP-ECO101 | Audit worktree vs canonical repo classification | ✅ COMPLETE |
| WP-ECO102 | Archive ghost worktrees | ✅ COMPLETE |
| WP-ECO103 | Enforce canonical = main-only policy | ✅ COMPLETE |

## Evidence

### Worktree Classification (WP-ECO101)
- Canonical repos: `repos/<name>/` — read/write for `main` only, verification passes
- Worktree repos: `repos/<name>-wtrees/` — feature work, CI, quality gates
- Canonical check: `git status --short --branch` should show `main` in canonical folders
- Rule enforced in `~/.claude/CLAUDE.md` and `Phenotype/repos/CLAUDE.md`

### Ghost Worktree Cleanup (WP-ECO102)
- Multiple ghost worktrees identified and cleaned up across rounds
- Worktree rule: NEVER author feature work in canonical folders
- Agent work happens in `repos/<name>-wtrees/<topic>/`

### Policy Enforcement (WP-ECO103)
- AgilePlus CLI enforces canonical worktree separation
- `git pull origin main` and explicit merge/cherry-pick integration are the only expected non-main operations in canonical folders
- Feature branches must use worktree pattern

## Commands

```bash
# Verify canonical is on main
cd repos/<name>
git status --short --branch  # should show: main

# List worktrees
git worktree list

# Canonical check command
cd /path/to/repo && git status --short --branch
```
