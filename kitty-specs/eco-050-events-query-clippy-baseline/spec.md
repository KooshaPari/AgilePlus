# eco-050: Events Query Clippy Baseline

## Goal

Remove the eight `clippy::collapsible_if` diagnostics in `EventQuery::filter`
without changing query filtering, ordering, inclusivity, limits, public APIs,
or dependencies.

## Acceptance Criteria

- `cargo clippy -p agileplus-events --all-targets -- -D warnings` succeeds.
- `cargo test -p agileplus-events --all-targets` succeeds.
- The only source change is the equivalent short-circuit guard normalization.
- The lockfile records the pre-existing declared `uuid` dependency used by the
  crate, with no manifest change.
