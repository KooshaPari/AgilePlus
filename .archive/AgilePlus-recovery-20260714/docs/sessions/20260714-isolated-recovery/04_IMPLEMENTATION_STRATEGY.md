# Implementation Strategy

The recovery uses a fresh clone rather than a worktree attached to the dirty source Git
directory. This isolates refs, index, object maintenance, hooks, and future branch changes.

Historical work will be assessed from immutable evidence and cherry-picked or reimplemented
only after a commit-by-commit adoption review. The source checkout remains untouched until
its unique work has been reconciled and independently verified.
