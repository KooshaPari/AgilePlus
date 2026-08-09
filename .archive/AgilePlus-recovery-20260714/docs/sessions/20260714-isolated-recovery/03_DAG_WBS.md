# Recovery DAG and WBS

| ID | Dependency | State | Evidence |
|---|---|---|---|
| R1 Verify live remote | - | complete | `origin/main` resolved to `a83a7677` |
| R2 Inventory dirty source | R1 | complete | sibling evidence archive |
| R3 Preserve Git and worktree evidence | R2 | complete | bundle, tar, patches, checksums |
| R4 Create isolated clone | R3 | complete | unique recovery path |
| R5 Create recovery branch | R4 | complete | `recovery/isolated-20260714` |
| R6 Validate recovery boundary | R5 | in progress | integrity, ancestry, status gates |
| R7 Recover selected historical work | R6 | pending | requires reviewed adopt matrix |

No feature work may depend on R7 until R6 is complete.
