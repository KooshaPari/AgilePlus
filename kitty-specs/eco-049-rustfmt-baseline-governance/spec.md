# eco-049: Rustfmt Baseline Governance

## Goal

Establish a governed, mechanically verified Rust formatting baseline for the
current `main` tree without changing runtime behavior, public APIs,
dependencies, or workflow policy.

## Scope

- Preserve the original formatter candidate branch and PR as evidence.
- Apply only the six-file `rustfmt` output already reviewed against `main`.
- Register this work in the kitty-spec index and attach the PR to this spec.

## Acceptance Criteria

- `cargo fmt --all -- --check` succeeds on the linked branch.
- The source diff contains only formatter-normalized Rust layout and imports.
- `git diff --check` succeeds.
- The linked PR carries `spec: eco-049-rustfmt-baseline-governance` and the
  required governance body sections.
- Existing broader CI failures are recorded as independent baseline debt; they
  must be green before this PR is approved for merge.
