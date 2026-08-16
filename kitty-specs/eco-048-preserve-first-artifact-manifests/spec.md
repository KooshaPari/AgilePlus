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

- **FR-PFAM-001:** A manifest must identify schema version, subject repository, Git
  common-dir, source ref, source commit, inventory timestamp, and producer.
- **FR-PFAM-002:** Every artifact entry must include a SHA-256 digest, byte size, stable
  MIME type, provenance, and retention location.
- **FR-PFAM-003:** Recovery evidence must contain restore outcome, ref-parity outcome,
  and `git fsck` outcome when applicable.
- **FR-PFAM-004:** Generated HTML must embed the canonical manifest digest and must be
  reproducible from the canonical JSON without changing its meaning.
- **FR-PFAM-005:** The feature must reject secret-bearing fields and must not enable a
  destructive operation.

## Acceptance Criteria

- A schema and representative fixture validate all required custody fields.
- A digest-parity check fails for stale or independently edited HTML.
- Validation records no source archive, deletion, or ref-mutation action.
- The evidence contract requires review, CI, audit-chain validity, and independent
  recovery proof before final completion.
