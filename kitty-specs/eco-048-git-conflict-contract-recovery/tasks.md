# Tasks: eco-048-git-conflict-contract-recovery

| ID | Work package | State | Evidence |
|---|---|---|---|
| WP01 | Preserve and classify #1022 | complete | Preserved ref `3ca2caab`; recovery ledger records each delta. |
| WP02 | Recompose parser contract | complete | Additive commit `cef5ee3b`; only `agileplus-git` source and merge test changed. |
| WP03 | Focused test and lint evidence | in_progress | Four parser tests and two merge flows pass; full baseline gates remain separately red. |
| WP04 | Governance, review, and hosted CI | blocked | Draft PR #1030 needs spec/governance metadata, review, and green broad gates. |

## Gate Ledger

- Focused parser tests: pass.
- Focused divergent merge tests: pass.
- Full formatter: blocked by preserved formatter-only lane #1028.
- Full crate test: blocked by leaked global temporary worktree path.
- Clippy: blocked by existing warnings outside `agileplus-git`.
