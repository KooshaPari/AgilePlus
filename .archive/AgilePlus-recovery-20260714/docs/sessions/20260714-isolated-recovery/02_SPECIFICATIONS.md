# Specifications

## Acceptance Criteria

- Clone from the live upstream default branch.
- Use a uniquely named path and recovery branch.
- Preserve the dirty checkout without state-changing Git commands.
- Preserve tracked diffs, untracked payloads, refs, stash, worktree heads, local-only
  commits, reflogs, and unreachable objects before any future cleanup.
- Verify remote identity, object integrity, clean status, and baseline ancestry.
- Do not implement product features in this recovery establishment step.

## ARUs

- Assumption: GitHub `origin/main` is the intended clean recovery baseline.
- Risk: case-colliding pull-request template names cannot both materialize on the default
  macOS case-insensitive filesystem.
- Mitigation: retain both objects in Git history and record the clone warning; do not infer
  loss from the working-tree representation alone.
