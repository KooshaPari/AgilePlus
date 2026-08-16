# Plan: Preserve-First Artifact Manifests

**Date**: 2026-08-15 | **Work Packages**: 4

## Work Packages

### WP01: Canonical custody-manifest schema

Define the JSON schema, stable identity, idempotent ingestion, fixtures, and validation command
for immutable artifact custody records. Depends on no prior WP.

### WP02: Agent and cockpit HTML projection

Define a deterministic renderer contract that consumes WP01 canonical JSON and embeds its
SHA-256 digest in generated HTML. Depends on WP01.

### WP03: Preservation and recovery evidence

Define retention, independent restore, ref-parity, and `git fsck` evidence contracts;
exclude any archive, deletion, or ref-changing command. Depends on WP01.

### WP04: Governance and contract checks

Bind the feature transitions to schema, digest-parity, secret-safety, review, CI, audit,
and recovery evidence. Depends on WP01, WP02, and WP03.

## Execution Waves

- **Wave 1:** WP01
- **Wave 2:** WP02 and WP03
- **Wave 3:** WP04

## Validation Strategy

- JSON: `python3 -m json.tool <manifest-or-contract>.json`
- Markdown: require all four packet documents and all planned WP files to be non-empty.
- Digest parity: canonicalize JSON to the exact FR-PFAM-004 bytes, compare the embedded
  SHA-256 digest, and byte-compare an observed view with a deterministic re-render.
- Governance: validate supported transition evidence through `contracts/governance-v1.json`,
  with an injected UTC clock and explicit maximum evidence age.
