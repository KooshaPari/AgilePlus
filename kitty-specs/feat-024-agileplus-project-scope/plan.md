# feat-024: Plan

## Atomic Commit Plan

This work is delivered as 8 atomic commits on `feat/agileplus-mcp-project-scope`,
each independently revertable:

| # | Subject | Files |
|---|---------|-------|
| C1 | `feat(proto): add ProjectScope message and project_scope to stateful core requests` | 4 proto files (local + agents) |
| C2 | `chore(proto-gen): regenerate Python protobuf bindings from updated protos` | 6 generated `_pb2*.py` |
| C3 | `feat(agileplus-git): add ProjectContext and migrate artifacts to docs/agileplus` | 6 Rust files (lib + tests + new module) |
| C4 | `feat(agileplus-mcp-intent): root intent storage in current Git worktree` | 2 files (Cargo.toml + storage.rs) |
| C5 | `feat(agileplus-cli): derive state from repo root, deprecate --db flag` | 2 files (main.rs + trace.rs) |
| C6 | `feat(agileplus-grpc): validate ProjectScope in 10 stateful handlers + thread repo root` | 4 files (main.rs + runtime.rs + server/mod.rs + tests) |
| C7 | `feat(agileplus-mcp): per-session root binding + client ProjectScope injection + roots://workspace` | 4 files (server.py + grpc_client.py + tests) |
| C8 | `docs(agileplus): repo-local state contract, migration plan, and tooling` | 4 files (docs + migration test + export script) |

Plus post-landing follow-up commits on the same branch:

| # | Subject | Reason |
|---|---------|--------|
| C9 | `chore(agileplus-cli): remove deprecated --db flag from cli_integration and e2e roundtrip tests` | Breaks the Autograder / e2e gate; needs C5 follow-up |
| C10 | `fix(agileplus-grpc): allow too_many_arguments on AgilePlusCoreServer::new` | Clippy `-D warnings` regression from C6's added arg |
| C11 | `docs(agileplus): register feat-024 kitty-spec for spec-first gate` | spec-first gate requires `kitty-specs/feat-024-.../` |

## Merge Strategy

- Branch: `feat/agileplus-mcp-project-scope` (renamed from `agilep/agilep-mcp-project-scope` to satisfy `pr-governance-gate`).
- Base: `origin/main` at `7dd6e2dc` (one commit ahead of local `main`).
- Mergify queue: enabled after pr-governance-gate passes.

## Rollback

Each commit is independently revertable. The C5/C6 commit pair is the only
contractual break (removal of `--db` flag). If a consumer depends on the
flag, C5 can be reverted and `--db` reintroduced without disturbing C1..C4
or C7..C8.
