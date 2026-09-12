# eco-048: AgilePlus ProjectScope per-session repo-root binding

## Goal

Make every stateful gRPC request carry a server-validated `ProjectScope`
identifying the canonical repository root, and make every MCP session bind its
own root on the wire so cross-session leakage is structurally impossible.
Bundle this with a coordinated move of AgilePlus state from
`kitty-specs/...` (legacy global paths) to repository-local
`docs/agileplus/<slug>/` so a fresh clone is self-contained.

## Acceptance Criteria

- Kitty-spec registered and indexed.
- Linked PR references `spec: eco-048-agileplus-project-scope`.
- Governance gates (policy-gate, pr-governance-gate, spec-first) satisfied
  after merge to `main`.
- `ProjectScope { canonical_repo_root }` defined in `proto/agileplus/v1/common.proto`
  and mirrored byte-identically in `agileplus-agents/proto/agileplus/v1/common.proto`.
- All 10 stateful core request messages carry a `project_scope` field.
- Python `AgilePlusCoreClient` injects `ProjectScope` from its
  canonicalized `project_root` argument.
- Rust `AgilePlusCoreServer` validates `project_scope` in every stateful
  handler (`validate_project_scope` helper, 3 focused tests).
- FastMCP `ProjectRootMiddleware` rejects malformed roots, binds the canonical
  root per session, and exposes it through `roots://workspace` resource.
- All artifact paths in the CLI move from `kitty-specs/...` to
  `docs/agileplus/<feature>/...`; legacy paths removed.

## Out of Scope

- Telemetry / scoring / dashboards for the new `ProjectScope`.
- Cross-repo federation (single-root only for this iteration).
- `.agileplus/agileplus.db` (runtime SQLite) — remains gitignored, not part
  of any commit.
