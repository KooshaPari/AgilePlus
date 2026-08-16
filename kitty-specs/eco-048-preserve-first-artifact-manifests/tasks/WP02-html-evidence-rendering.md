---
work_package_id: WP02
title: HTML Evidence Rendering
feature: Preserve-First Artifact Manifests
feature_slug: eco-048-preserve-first-artifact-manifests
sequence: 2
state: planned
created_at: 2026-08-16T01:51:48Z
depends_on:
  - WP01
---

# Work Package: HTML Evidence Rendering

## Objective

Render a read-only agent/cockpit HTML evidence view solely from canonical custody JSON and
bind it to that exact JSON digest.

## File Scope

- `crates/agileplus-dashboard/src/artifacts/custody_html.rs`
- `crates/agileplus-dashboard/tests/custody_html_test.rs`
- `templates/artifacts/custody-manifest.html`
- `tests/fixtures/artifacts/custody-manifest.valid.json`
- `tests/fixtures/artifacts/custody-manifest.expected.html`

## Dependencies

- WP01 canonical custody-manifest schema and deterministic JSON emission.

## Acceptance Criteria

- The renderer accepts only a WP01-valid canonical JSON record.
- HTML displays repository/ref/source provenance, artifact hashes and sizes, stable MIME,
  producer, retention, and recovery evidence without rendering secret fields.
- The renderer embeds the SHA-256 of the canonical JSON bytes in a machine-readable HTML
  attribute or metadata element.
- Re-rendering unchanged JSON yields byte-identical HTML.
- Independently edited or stale HTML fails digest-parity validation.

## Test-First Commands

1. Add a digest-parity test for an HTML fixture with an intentionally incorrect digest.
2. Run: `cargo test -p agileplus-dashboard custody_html_rejects_digest_mismatch -- --exact`
   Expected before implementation: fail because no renderer/parity validator exists.
3. Add the smallest renderer and parity validator that consume WP01 JSON.
4. Run: `cargo test -p agileplus-dashboard custody_html -- --nocapture`
   Expected after implementation: deterministic-render and mismatch-rejection cases pass.
5. Run: `cargo fmt --check -p agileplus-dashboard`

## Preserve-First Prohibitions

- HTML is a derived view and must never become the custody authority.
- Do not add controls that archive, delete, rewrite refs, force-push, reset, or clean.
- Do not display or persist credentials, tokens, passwords, private keys, or secrets.
