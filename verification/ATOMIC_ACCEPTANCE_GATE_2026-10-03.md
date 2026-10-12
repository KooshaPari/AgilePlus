# Atomic acceptance verification gate

Date: 2026-10-03
Scope: AgilePlus terminal work acceptance only.

## Canonical implementation

Terminal acceptance is owned by `AtomicAcceptancePort` and the SQLite
`repository::acceptance::commit_acceptance` transaction. Generic HTTP/gRPC
state-transition surfaces are not acceptance authorities.

The SQLite transaction uses `BEGIN IMMEDIATE` before acceptance-relevant
reads. Inside the transaction it re-resolves feature state, governance contract,
work packages, governance evidence/policies/metrics, Assignment/SpecRevision
scope, Evaluation/criterion receipts, Attempts, and the existing audit chain.
Only after all checks pass does it compare-and-set WP and Feature terminal
states, append audit entries, append the feature event, persist the immutable
request/receipt, and commit.

## Required executable witnesses

The canonical module owns tests for:

1. successful atomic commit of WP state, Feature state, audits, event and receipt;
2. exact request replay returning the original receipt without duplicate writes;
3. same request ID with changed immutable command returning Conflict;
4. invalid pre-existing audit chain causing zero terminal mutation;
5. forced failure at the final acceptance-receipt INSERT rolling back prior WP
   state, Feature state, audits and event.

## CI truth rule

`ci / test` must not report green merely because an aggregate job ran.
When Rust is present, the Rust job must complete successfully so `cargo test`
was actually reached and passed. A format/Clippy/build failure therefore leaves
the test gate red/unknown rather than falsely green.

## Current evidence status

The previous native candidate was blocked before Rust tests by a duplicate
module (`repository/acceptance.rs` and `repository/acceptance/mod.rs`).
The weaker duplicate and duplicate trait implementation were removed. The
stronger canonical module remains.

The witnesses above are authored on the current candidate. They are not claimed
passing until a fresh native Rust run executes them.
