# Specifications

- Remove the three unused dashboard declarations/imports, five duplicate helpers, and the
  unused `agileplus-api` dashboard dependency.
- Rename the unused feature-report parameter to `_workpackages`; preserve its signature and call sites.
- Change only the coverage XML invocation to `uv run --locked --no-sync --no-build coverage xml -o ../.coverage/coverage.xml`.
- Exercise registered MCP tools through an in-memory FastMCP `Client.call_tool` invocation,
  retain the `ValueError` assertion, and verify exact mocked command arguments.
- Add only an explicit `protobuf-compiler` installation to the hosted Rust coverage job,
  including a version check before the Rust toolchain runs.
- Do not add dead-code allowances or alter runtime behavior.
