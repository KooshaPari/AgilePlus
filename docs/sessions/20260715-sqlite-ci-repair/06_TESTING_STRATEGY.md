# Testing strategy

1. Reproduce `cargo fmt --all -- --check` on the clean base.
2. Reproduce `cargo test -p agileplus-sqlite` on the clean base.
3. Add a focused regression test or compilation-level reproduction before the
   source repair.
4. Verify format, SQLite package tests, workspace build, and workspace tests.

## Evidence

- Baseline SQLite package suite: 80 passed, 0 failed.
- After formatter normalization: `cargo fmt --all -- --check` passed.
- After formatter normalization: `cargo test --workspace -q` passed.
- `cargo clippy --workspace -- -D warnings` passed after dependency and
  feature-contract repairs.
- `cargo build --workspace --locked -q` passed.
- `cargo test --workspace --all-features -q` passed (with the documented
  existing graph feature warnings).
