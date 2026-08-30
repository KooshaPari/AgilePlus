# GitHub JWT Dependency Removal

**Status:** approved by sponsor continuation (`proc`) on 2026-08-29.

## Goal

Remove the unused Octocrab read facade so `RUSTSEC-2023-0071` is eliminated
without adding an advisory exception or substituting a different JWT backend.

## Context

`cargo tree -i rsa@0.9.10` resolves only through:

```
agileplus-cli -> agileplus-github -> octocrab -> jsonwebtoken -> rsa
```

The working GitHub synchronization path is `client.rs` and `sync.rs`, which
use `reqwest`. No workspace caller uses the Octocrab-only `octo` facade.

## Decision

Remove `octo.rs`, its public re-export, and the `octocrab` dependency. Retain
the raw authenticated GitHub client and all create, update, get, and sync
behavior. Do not switch Octocrab to AWS-LC: that would retain unused JWT
capability and expand the native crypto surface.

The Cargo-deny wildcard findings are intentionally separate: they require
explicit versions on internal path dependencies and must not be bundled with
this security change.

## Acceptance Criteria

1. `octocrab`, `jsonwebtoken`, and `rsa` are absent from the resolved graph.
2. `cargo deny check advisories` passes with no ignore/suppression entry.
3. `cargo test --locked -p agileplus-github` and `cargo check --workspace --locked` pass.
4. No live GitHub sync behavior changes.

## Risks and Rollback

Removing the unconsumed public re-export is a compatibility change for
external callers. The successor PR must state this explicitly. Rollback is a
normal revert of that focused commit; it must not reintroduce an advisory
exception.
