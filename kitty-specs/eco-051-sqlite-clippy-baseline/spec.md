# eco-051: SQLite Clippy Baseline

## Goal

Remove the three `clippy::collapsible_if` diagnostics in the SQLite catalog
parser and migration binary without changing parsing, migration-path creation,
public APIs, dependencies, or workflow policy.

## Acceptance Criteria

- `cargo clippy -p agileplus-sqlite --all-targets -- -D warnings` succeeds.
- `cargo test -p agileplus-sqlite --all-targets` succeeds.
- Changes are limited to compiler-suggested short-circuit guard normalization
  in `seed/catalog.rs` and `bin/migrate.rs`.
