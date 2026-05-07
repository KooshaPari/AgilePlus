# Space backlog cleanup

Audit the workspace for reclaimable generated artifacts and remove safe disk-heavy outputs to reduce disk usage without disturbing active worktrees.

## Scope
- Identify large generated directories and files in the current workspace.
- Remove safe build artifacts, caches, and other generated outputs.
- Leave source, worktrees, and tracked project files intact.

## Outcome
- Reduced disk usage.
- Clear summary of what was removed and what was left untouched.
