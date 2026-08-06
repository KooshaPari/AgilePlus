# Implementation strategy

Use additive, minimal edits with `apply_patch`. Keep test-only synchronization imports in the
test module, retain all live settings helpers, and preserve existing feature-report call flow.
The sparse worktree does not materialize the workspace `Cargo.toml`, so no compiler command is
run from this checkout.
