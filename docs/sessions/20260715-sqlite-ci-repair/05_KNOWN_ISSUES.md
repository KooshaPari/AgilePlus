# Known issues

## Repaired in this slice

- Workspace rustfmt drift across 84 files prevented the CI `core-check` job
  from reaching build or test stages.
- A partial tonic/prost dependency upgrade made the configured protobuf build
  API unavailable to `rust/build.rs`.
- The `agileplus-domain` keychain feature was used in cfg attributes but not
  declared in its manifest; strict Clippy also identified three local
  simplifications in domain/traceability validation code.

## Preserved, not merged

- `chore/sqlite-test-migration-runner` contains an uncommitted
  `test_support.rs` module and a corresponding `lib.rs` edit. It has no open
  PR and is not a failure on `origin/main`; this repair deliberately leaves it
  in place for its owning worktree.

## Remaining quality debt outside this slice

- `cargo test --workspace --all-features` emits existing graph `neo4j`
  `unexpected_cfgs` warnings. The default-feature strict Clippy gate is clean;
  enabling graph's unsupported optional feature requires separate feature
  contract work.
