---
description: Show AgilePlus worktree layout and branch posture
---

# ap-worktree

Inspect current worktree shape and guard against accidental edits outside feature scope.

## What this does

1. Lists git branches and current branch.
2. Prints matching `*-wtrees` and `worktrees` dirs.
3. Suggests safe next branch naming if detached or out of scope.

## Steps

```pwsh
git status -sb
git branch --show-current
Get-ChildItem -Path . -Directory -Filter "*-wtrees" | Select-Object -ExpandProperty Name
if (Test-Path ".worktrees") {
    Get-ChildItem -Path ".worktrees" -Directory | Select-Object -ExpandProperty Name
}
```

## Usage

```
/ap-worktree
```

