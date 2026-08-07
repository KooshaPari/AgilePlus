# Testing strategy

- Run `git diff --check` after edits.
- Re-scan helper names and imports with `rg` to confirm duplicate helpers have no remaining callers.
- Run the narrow Python MCP test if its local dependencies are available.
- Do not run full workspace, release, or compiler builds from this sparse worktree.
- Run `actionlint` and `git diff --check`; do not run a local Cargo coverage build due to
  disk/process safety constraints.
