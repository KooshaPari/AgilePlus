# Specifications

- Remove only the two unused imports and five duplicate helpers.
- Rename the unused feature-report parameter to `_workpackages`; preserve its signature and call sites.
- Change only the coverage XML invocation to `uv run --locked --no-sync --no-build coverage xml -o ../.coverage/coverage.xml`.
- Acquire the MCP tool before `pytest.raises`; keep the same `ValueError` assertion and payload.
- Add only an explicit `protobuf-compiler` installation to the hosted Rust coverage job,
  including a version check before the Rust toolchain runs.
- Do not add dead-code allowances or alter runtime behavior.
