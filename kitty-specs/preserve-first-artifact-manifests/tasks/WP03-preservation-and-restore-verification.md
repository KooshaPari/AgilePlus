---
work_package_id: WP03
title: Preservation and Restore Verification
feature: Preserve-First Artifact Manifests
feature_slug: preserve-first-artifact-manifests
sequence: 3
state: planned
created_at: 2026-08-16T01:51:48Z
depends_on:
  - WP01
---

# Work Package: Preservation and Restore Verification

## Objective

Capture non-destructive preservation and independently verified recovery evidence for the
repository content referenced by a custody manifest.

## File Scope

- `crates/agileplus-domain/src/artifacts/recovery_evidence.rs`
- `crates/agileplus-domain/tests/recovery_evidence_test.rs`
- `docs/artifacts/recovery-evidence.schema.json`
- `tests/fixtures/artifacts/recovery-evidence.valid.json`
- `tests/fixtures/artifacts/recovery-evidence.ref-parity-failed.json`

## Dependencies

- WP01 canonical custody-manifest schema and repository/ref identity fields.

## Acceptance Criteria

- Recovery evidence records the retention locator, independent restore actor and timestamp,
  restore outcome, ref-parity outcome, and `git fsck` outcome.
- A successful state requires explicit success evidence for restore, ref parity, and
  `git fsck`; an omitted or failed value is not upgraded to success.
- The verifier compares preserved and restored refs without mutating either repository.
- The verification record links to the relevant canonical manifest digest.
- Fixtures and logs contain no secrets.

## Test-First Commands

1. Add the failed-ref-parity fixture and a test that asserts it cannot produce a successful
   recovery result.
2. Run: `cargo test -p agileplus-domain recovery_evidence_rejects_failed_ref_parity -- --exact`
   Expected before implementation: fail because recovery-evidence evaluation is absent.
3. Add the minimal non-mutating recovery-evidence evaluator.
4. Run: `cargo test -p agileplus-domain recovery_evidence -- --nocapture`
   Expected after implementation: valid recovery and failed-parity cases pass.
5. Run: `python3 -m json.tool docs/artifacts/recovery-evidence.schema.json >/dev/null`

## Preserve-First Prohibitions

- Do not invoke archive, delete, reset, clean, prune, ref rewrite, or force-push commands.
- Do not treat a remote upload, artifact digest, or manifest write as restore proof.
- Do not capture or emit secrets in locators, logs, fixtures, or evidence fields.
