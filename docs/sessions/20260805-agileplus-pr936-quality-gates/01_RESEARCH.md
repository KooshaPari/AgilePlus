# Research

- `routes/mod.rs` top-level `Arc` and `RwLock` imports duplicate test-local imports and are unused.
- `routes/pages.rs` imports `DashboardFilter` without use.
- The five Plane/coverage helpers in `routes/helpers.rs` have no callers; live implementations and callers are in `routes/settings.rs`.
- `routes/features.rs::build_feature_reports` accepts `workpackages` but does not read it; callers still pass the same value.
- Coverage workflow runs from `python` after `uv sync`; the requested locked/no-sync/no-build invocation preserves that sequence.
- `python/tests/test_mcp_tools.py` nests tool lookup and invocation in one `pytest.raises` body.
- The Rust coverage job invokes `cargo llvm-cov --workspace --all-features`; the
  `agileplus-proto` build script uses `prost-build`/`tonic-build`, which requires
  `protoc` for this full-featured path. Existing `autograder.yml` and
  `gate-check.yml` install Ubuntu's `protobuf-compiler` package.
