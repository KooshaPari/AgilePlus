# Implementation Strategy

## Architectural direction

Retain the existing Rust domain/application/adapter boundaries. Introduce no compatibility
shims: replace competing environment readers and fake runnable server paths with a single
runtime resolver and a real generated service boundary. Keep the Python MCP server as a
client of that server, not an alternate source of workflow truth.

## Focused implementation lanes

### Lane A: build and runtime

Modify `crates/agileplus-proto/build.rs`, server crate manifest/source, configuration
loader, `process-compose.yml`, platform health, launcher, and dashboard/MCP configuration
consumers. Add an explicit test profile for generated proto code; only non-runnable static
analysis may use stubs. Make the resolver emit a redacted structured diagnostic used by all
subprocesses and probes.

### Lane B: credentials

Consolidate `crates/agileplus-domain/src/credentials/` around one production keychain
factory. Remove production selection of `file.rs`; add a test-only memory fixture. Thread
credential references, never values, through integration configuration and audit events.

### Lane C: events, artifacts, and streaming

Add repository migrations and adapters for typed event queries and cursors; bind the query
and stream RPCs to the real service. Add an S3/MinIO artifact adapter with content digest
and metadata. Implement API stream framing and MCP forwarding from the same cursor source.

### Lane D: evidence and consumers

Add a versioned evidence-manifest schema and verifier owned by AgilePlus. Dogfood workers
must invoke real commands against the launched platform, retain outputs by project scope,
and use the verifier before declaring a rollout passed.

## Rollout safety

- Work in focused branches; each lane includes a migration/reversibility assessment.
- Use development keychain entries and isolated MinIO buckets, never personal credentials.
- Do not start or kill shared services until resolved port ownership is checked.
- The evidence verifier rejects missing references, wrong project scope, secret-like values,
  invalid event ordering, digest mismatches, or failed audit chains.
