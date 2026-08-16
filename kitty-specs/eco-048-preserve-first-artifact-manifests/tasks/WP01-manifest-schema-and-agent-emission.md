---
work_package_id: WP01
title: Manifest Schema and Agent Emission
feature: Preserve-First Artifact Manifests
feature_slug: eco-048-preserve-first-artifact-manifests
sequence: 1
state: planned
created_at: 2026-08-16T01:51:48Z
depends_on: []
---

# Work Package: Manifest Schema and Agent Emission

## Objective

Define canonical JSON custody records emitted by agents for their output and cockpit HTML.
The schema is the sole evidence authority; no source-code archive or mutation is permitted.

## File Scope

- `crates/agileplus-domain/src/artifacts/custody_manifest.rs`
- `crates/agileplus-domain/tests/custody_manifest_test.rs`
- `docs/artifacts/custody-manifest.schema.json`
- `tests/fixtures/artifacts/custody-manifest.valid.json`
- `tests/fixtures/artifacts/custody-manifest.missing-producer.json`
- `tests/fixtures/artifacts/custody-manifest.synthetic-secret.json`

## Dependencies

None.

## Acceptance Criteria

- The schema requires schema version, repository identity, Git common-dir, source ref,
  source commit, inventory timestamp, and producer.
- Each artifact entry requires SHA-256, byte size, stable MIME type, provenance, and
  retention location.
- The schema permits recovery evidence for restore, ref parity, and `git fsck` without
  treating their absence as a successful preservation claim.
- Agent emission rejects credential-bearing fields and values; fixtures use no secrets.
- The emitted JSON has the exact canonical serialization defined by FR-PFAM-004 before its
  digest is calculated, and a stable manifest identity makes repeat ingestion idempotent.
- Synthetic, non-reusable token, password, and private-key-shaped free-text inputs are rejected
  before digest generation or audit-event emission.

## Test-First Commands

1. Add invalid missing-producer and synthetic-secret fixtures plus tests that assert rejection.
2. Run: `cargo test -p agileplus-domain custody_manifest_rejects_missing_producer -- --exact`
   Expected before implementation: fail because the custody-manifest validator is absent.
3. Add the minimal schema and emitter implementation.
4. Run: `cargo test -p agileplus-domain custody_manifest -- --nocapture`
   Expected after implementation: passing valid, missing-field, idempotent-ingestion, digest,
   and synthetic-secret rejection cases.
5. Run: `python3 -m json.tool docs/artifacts/custody-manifest.schema.json >/dev/null`

## Preserve-First Prohibitions

- Do not archive, delete, rename, reset, clean, or rewrite any source repository or ref.
- Do not emit credentials, tokens, passwords, private keys, or unredacted secrets.
- Do not make the manifest imply that a digest alone proves restoration or source parity.
