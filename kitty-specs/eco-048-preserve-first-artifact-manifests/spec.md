# Preserve-First Artifact Manifests

## Goal

Create a governed evidence packet for agent-produced and cockpit HTML artifacts. Each
artifact custody record has canonical JSON; generated HTML is an informative view whose
embedded manifest digest must equal the canonical JSON digest.

## Scope

- Record artifact custody for agent output, cockpit HTML, and their related evidence.
- Record Git common-dir identity, source ref and commit provenance, stable MIME type,
  producer, artifact hash and size, retention location, and recovery evidence.
- Require restore result, ref-parity result, and `git fsck` evidence where Git content is
  in scope.
- Generate HTML only from canonical JSON and make digest parity testable.

## Out of Scope

- Source-code archiving, deletion, repository retirement, ref rewrites, force pushes,
  cleanup, or destructive workflows.
- Storage of tokens, credentials, private keys, or other secrets.
- Treating HTML as authoritative evidence when canonical JSON is absent or mismatched.

## Functional Requirements

- **FR-PFAM-001:** A manifest must identify schema version, stable manifest identity,
  subject repository, Git common-dir, source ref, source commit, inventory timestamp, and
  producer; schema-valid manifests must be ingested idempotently and invalid input rejected.
- **FR-PFAM-002:** Every artifact entry must include a SHA-256 digest, byte size, stable
  MIME type, provenance, and retention location.
- **FR-PFAM-003:** Recovery evidence must contain restore outcome, ref-parity outcome,
  and `git fsck` outcome when applicable, and append it as an immutable SHA-256 hash-chained
  audit event linked to the canonical manifest digest.
- **FR-PFAM-004:** Generated HTML must embed the SHA-256 of canonical manifest JSON: UTF-8,
  lexicographically sorted object keys, schema-defined artifact arrays sorted by normalized
  path, compact JSON with no trailing newline, standard JSON escaping, and integer-only byte
  sizes. Rendering must be reproducible from those bytes without changing their meaning.
- **FR-PFAM-005:** The feature must reject secret-bearing fields and must not enable a
  destructive operation.
- **FR-PFAM-006:** Completion evidence must use the supported governance contract schema
  and record CI, review approval, and audit-chain attestation without asserting a runtime
  feature binding before the spec engine registers one; it must use an injected UTC clock and
  explicit maximum-age policy to deny stale evidence.

## Acceptance Criteria

- A schema and representative fixture validate all required custody fields.
- Repeating ingestion of the same stable manifest identity creates no duplicate custody record;
  malformed or schema-invalid input is rejected before a digest or audit event is written.
- A deterministic-render check byte-compares the expected and observed HTML, so stale or
  independently edited HTML fails even when its embedded JSON digest is unchanged.
- Validation records no source archive, deletion, or ref-mutation action.
- The evidence contract requires review, CI, audit-chain validity, and independent
  recovery proof before final completion.
