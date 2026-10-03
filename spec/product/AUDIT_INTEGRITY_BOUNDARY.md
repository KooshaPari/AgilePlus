# AgilePlus Audit Integrity Boundary Finding

**Date:** 2026-09-30  
**Severity:** high semantic/documentation finding; historical hashes must not be rewritten.

## Finding

`hash_entry` currently hashes:
- feature_id;
- wp_id when present;
- timestamp;
- actor;
- transition;
- prev_hash.

It does not hash:
- entry id;
- evidence_refs;
- event_id;
- archived_to.

Tests explicitly pin part of this behavior, including evidence_refs not changing the hash.

The function documentation says the hash covers "all mutable fields." That statement is false.

## Risk

If consumers interpret the hash chain as tamper-evidence for evidence bindings/event/archive metadata, those fields can change without breaking the current entry hash.

This does not mean the current chain is useless. It means its integrity envelope is narrower than described.

## Non-destructive correction options

### A — define v1 envelope honestly

Declare the current hash schema as `audit-hash-v1` covering only transition-core fields. Protect evidence/event/archive references through their own immutable identities/receipts.

Pros: preserves all hashes and behavior.
Cons: chain does not directly attest those references.

### B — append-only v2 hash schema for new entries

Add `hash_schema_version` and define v2 canonical serialization including desired fields.

Historical v1 entries remain verifiable under v1.

Pros: stronger future envelope without rewriting history.
Cons: versioned verification complexity.

## Recommendation for mature program

Use explicit hash schema versioning. Do not silently change `hash_entry` for old rows.

Before v2 decide whether evidence references should be content hashes/immutable receipt IDs rather than mutable database IDs/paths.

## Required tests

- v1 historical fixture remains valid;
- v1 mutation of non-covered field demonstrates documented limitation;
- v2 mutation of every covered field breaks verification;
- canonical serialization stable across platforms;
- chain can cross v1→v2 boundary with explicit semantics;
- archive operation cannot silently change covered content.

## Gate

Do not claim "immutable audit log covers evidence" until this boundary is resolved/documented.
