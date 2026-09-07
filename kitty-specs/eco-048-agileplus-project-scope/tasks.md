# eco-048: Tasks

- [x] **C1** — Add `ProjectScope` to `proto/agileplus/v1/common.proto` and mirror
  to `agileplus-agents/proto/agileplus/v1/common.proto`; add `project_scope` to
  all 10 stateful core requests in both `core.proto` files; verify byte parity.
- [x] **C2** — Regenerate Python protobuf bindings (`*.pb2.py`,
  `*_pb2_grpc.py`) from the updated protos.
- [x] **C3** — Add `agileplus-git::ProjectContext` and migrate artifact paths
  from legacy `kitty-specs/...` to repo-local `docs/agileplus/<feature>/...`.
- [x] **C4** — `agileplus-mcp-intent`: bind intent storage to the current
  Git worktree root.
- [x] **C5** — `agileplus-cli`: derive state from repo root; deprecate
  (hide) `--db` flag.
- [x] **C6** — `agileplus-grpc`: thread `canonical_repo_root` through
  `main.rs → start_server → AgilePlusCoreServer::new`; add
  `validate_project_scope` helper; wire it into all 10 stateful handlers.
- [x] **C7** — Python MCP: implement `ProjectRootMiddleware` (session-bound
  root validation); make `AgilePlusCoreClient` inject `ProjectScope` on every
  stateful request; expose canonical root through `roots://workspace`
  resource.
- [x] **C8** — Add `docs/agileplus/repo-state-contract.md`,
  `docs/agileplus/central-state-owner-map.example.yaml`,
  `docs/superpowers/plans/2026-09-06-repo-local-state-boundary.md`,
  `scripts/export-central-state.py`, and the migration test
  `python/tests/migration/test_export_central_state.py`.
- [x] **C9** — Update `crates/agileplus-cli/tests/cli_integration.rs` and
  `tests/e2e/roundtrip.rs` to stop passing `--db` (post-C5 follow-up to keep
  the Autograder and CLI round-trip gates green).
- [x] **C10** — Add `#[allow(clippy::too_many_arguments)]` to
  `AgilePlusCoreServer::new` to keep Clippy `-D warnings` clean.
- [x] **C11** — Register `kitty-specs/eco-048-agileplus-project-scope/`
  with `meta.json`, `spec.md`, `plan.md`, `tasks.md` for the spec-first gate.
- [ ] **Merge** — Once all gates green, Mergify merge queue.
