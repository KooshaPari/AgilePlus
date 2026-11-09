# Frozen contracts — spec 002 (org-wide-release-governance-dx-automation)

Per `worklogs/DUPLICATION.md` 2026-04-24 AgilePlus `core.proto` 4-copy SSOT
scoping (issue #121, step 2), live contracts now live under
`proto/agileplus/v1/`. This directory preserved the design snapshot at spec
freeze. Files below list the frozen SHA-256; to reproduce, run
`git show <tag>:kitty-specs/002-org-wide-release-governance-dx-automation/contracts/<file>`
(after tag `spec-002-frozen` lands) or `git log -- <path>` to browse history.

Context:
- Step 1 (#123): archive copies under `.archive/` were pruned.
- Step 2 (this PR): spec snapshot frozen as manifest to stop SHA drift.
- Canonical source of truth for proto contracts: `proto/agileplus/v1/`.

## Frozen files

| File | SHA-256 | Size (bytes) |
|---|---|---|
| `registry-adapter.md` | `8415986b076f9af50e41a59c599b0e646fff86a58e476e443c8388f65a32a174` | 2905 |
