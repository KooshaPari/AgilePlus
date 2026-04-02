# shelf-infra Specification

## Overview

Infrastructure management scripts and tooling for the shelf/repos workspace. Contains shared scripts for managing git worktrees, project scaffolding, and shelf-level operations.

## Contents

- `scripts/infra-manage.sh` - Main infrastructure management script
- `hooks/` - Git hooks for project management
- `.github/workflows/` - CI/CD for infrastructure tooling

## Tools

- Shell scripts (bash/zsh)
- Git worktree management
- Pre-commit hooks

## Usage

```bash
./scripts/infra-manage.sh <command>
```

## Dependencies

- Git
- Standard Unix tools