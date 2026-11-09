# Frozen contracts — spec 001 (spec-driven-development-engine)

Per `worklogs/DUPLICATION.md` 2026-04-24 AgilePlus `core.proto` 4-copy SSOT
scoping (issue #121, step 2), the live contract now lives at
`proto/agileplus/v1/`. This directory preserved the design snapshot at spec
freeze. Files below list the frozen SHA-256; to reproduce, run
`git show <tag>:kitty-specs/001-spec-driven-development-engine/contracts/<file>`
(after tag `spec-001-frozen` lands) or `git log -- <path>` to browse history.

Context:
- Step 1 (#123): archive copies under `.archive/` were pruned.
- Step 2 (this PR): spec snapshot frozen as manifest to stop SHA drift.
- Canonical source of truth: `proto/agileplus/v1/`.

## Frozen files

| File | SHA-256 | Size (bytes) |
|---|---|---|
| `agents.proto` | `369ef1213c67aa7e6544557d9cb232906eba433a0bd71d7d74a3dde4a95968a7` | 2514 |
| `agileplus.proto` | `5d3fce2c047ac624e6002b631078ae3e338f7ba62f541f3ac278106c62fe05b5` | 4225 |
| `common.proto` | `e968f7cf46bc8bc2ebec57e44aab45ed6a0d89f0642772b60ac8e91b6d6dcd14` | 2206 |
| `core.proto` | `0684ee78484db003549b0dd562c066aae8fec0c72c705d15643be1f8fc7b89c4` | 2226 |
| `integrations.proto` | `0e71dd0813f7530aa2330f3b05dc45c15a60df7f2bed575c62bd804157eb64f1` | 3614 |
| `mcp-tools.json` | `9121a235dc96ba6357c9786bfbec3baea0fb9035c2281e18a7dddaf38d5eba8c` | 4012 |

Note: `agileplus.proto` and `mcp-tools.json` exist only in the frozen snapshot;
they were design-time artifacts superseded by the modular split under
`proto/agileplus/v1/`.
