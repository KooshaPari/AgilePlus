# Research: kooshapari-stale-repo-triage (Post-Specify)
**Date**: 2026-04-01 | **Mode**: delete-readiness validation

## Validated Case: `thegent-cache`

### Decision

`KooshaPari/thegent-cache` should remain **archived**, not deleted.

### Evidence

- Local active code still exists in:
  - `/Users/kooshapari/CodeProjects/Phenotype/repos/thegent/crates/thegent-cache`
  - `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/crates/thegent-cache`
- The active `thegent` workspace still includes `thegent-cache` in `crates/Cargo.toml`.
- Package identity has **not** been migrated:
  - Rust package: `thegent-cache-rs`
  - Rust lib: `thegent_cache`
  - Python package: `thegent_cache_rs`
- No local `pyfacet` references were found.
- Shelf docs still refer to `thegent-cache` as a live crate and component.

### Best-Practice Interpretation

The standalone GitHub repo was archived correctly as part of consolidation, but deletion is blocked
because the code and naming are still live inside the active `thegent` trees.

### Delete-Readiness Blockers

1. active workspace membership
2. old package names still present in manifests
3. no local successor identity under `pyfacet`
4. widespread docs references
5. incomplete mirror cleanup in `platforms/thegent`

## Recommended Approach

- keep archived repos archived by default
- delete only after active local code identity, manifests, and docs are fully migrated
- treat stale repo triage as a decision log, not a pressure to hard-delete provenance too early
