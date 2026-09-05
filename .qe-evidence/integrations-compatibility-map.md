# AgilePlus integration compatibility map

Captured 2026-09-05 from detached HEAD `4104639fd376fdd432455ebe6e4d55f16fab9214`.

## Current surfaces

| Surface | Current truth | Consequence |
| --- | --- | --- |
| Generated service | `IntegrationsService` requires 11 RPC methods: six sync methods plus `classify_input`, `create_backlog_item`, `list_backlog`, `promote_backlog_item`, and `generate_router`. | A concrete server implementation must provide every method; unrelated methods may return explicit `UNIMPLEMENTED`. |
| Server registration | `server/bootstrap.rs` already adds `IntegrationsServiceServer::new(service)`. | The service is intended to be live once a trait implementation is compiled. |
| Module declaration | `server/mod.rs` did not declare `mod integrations;`. | The stale file was previously excluded, so the binary returned tonic's default `UNIMPLEMENTED`. |
| Backlog storage | `StoragePort` does not expose backlog operations; `ContentStoragePort` exposes `create_backlog_item`, `get_backlog_item`, `list_backlog_items`, status/priority updates, and pop. | `AgilePlusCoreServer<S: StoragePort>` cannot call backlog APIs without an architectural adapter or a bound change. |
| Backlog handlers | `server/integrations.rs` has handlers, but they target removed `triage`, `router`, conversion, and proto surfaces. | The file cannot be enabled as-is. |
| Generated create request | Fields are `type`, `title`, `body`, `priority`, `feature_id`, `wp_id`, `triaged_by`. | Handler references to `description`, `source`, `feature_slug`, and `tags` are stale. |
| Generated list request | Fields are `type_filter`, `state_filter`, `feature_slug` only. | Handler references to `priority_filter`, `source_filter`, `limit`, and `sort` are stale. |
| Domain triage/router | Current checkout has no `agileplus_domain::domain::triage`, `server::triage`, or `server::router` module matching the stale handler. | `classify_input` and `generate_router` need a new implementation or explicit `UNIMPLEMENTED`. |
| Conversion | `crates/agileplus-grpc/src/conversions.rs` has no `backlog_item_to_proto`. | A compatible conversion function must be added before create/list can return proto items. |

## Verified compile evidence

With `mod integrations;` enabled, `cargo build --locked -p agileplus-grpc --bin agileplus-grpc` failed with 23 errors, including unresolved removed messages/modules, missing `ContentStoragePort` methods on `StoragePort`, stale request fields, and missing backlog conversion. The exact failing command was run in this worktree; no commit or push was made.

## Smallest viable implementation plan

1. Decide whether `AgilePlusCoreServer` should be generic over a combined storage bound (`StoragePort + ContentStoragePort`) or whether `StoragePort` should absorb the backlog methods. This is an architecture/API decision, not a local harness change.
2. Add a current-proto `backlog_item_to_proto` conversion using `BacklogItem` fields (`id`, `intent`, `title`, `description`, `priority`, `status`, `source`, `created_at`).
3. Replace the stale create/list handlers with current fields, map `feature_id` to the domain feature slug, and reject non-empty `wp_id` with `Status::invalid_argument` before storage.
4. Implement only `create_backlog_item` and `list_backlog` through the generated trait; return explicit `UNIMPLEMENTED` for the remaining integration RPCs until their current domain ports exist.
5. Add `mod integrations;`, rebuild the candidate, and run the QE backlog contracts. Then run the existing core/unit/Ruff gates.

## Decision boundary

Do not continue this slice without an owner decision on the storage-bound change. Enabling the current stale module without that decision broadens into domain-port and conversion changes and cannot be represented as a narrow harness patch.

## Recommended smallest architecture-safe implementation

Use the existing `SqliteStorageAdapter` dual implementation rather than changing
the domain `StoragePort` contract:

1. In `crates/agileplus-grpc/src/server/mod.rs`, import
   `ContentStoragePort` only where needed and declare `mod integrations;`.
2. In `crates/agileplus-grpc/src/server/bootstrap.rs`, add
   `S: ContentStoragePort` to `start_server`'s bounds. This is the sole wiring
   boundary that constructs `IntegrationsServiceServer`; it preserves every
   existing `StoragePort` consumer and is satisfied by
   `agileplus_sqlite::SqliteStorageAdapter`, which implements both ports.
3. Replace the stale `integrations.rs` body with a current-proto adapter:
   - retain the generated `IntegrationsService` trait implementation;
   - forward `create_backlog_item` and `list_backlog` to small handlers;
   - reject non-empty `CreateBacklogItemRequest.wp_id` with
     `Status::invalid_argument` before storage;
   - map `body -> BacklogItem.description`, `feature_id -> feature_slug`,
     `triaged_by` to `BacklogItem.source` (default `grpc`), and use current
     `type_filter/state_filter/feature_slug` list fields;
   - return explicit `UNIMPLEMENTED` for the six Plane/GitHub methods and
     `classify_input`, `promote_backlog_item`, and `generate_router` until
     current ports exist. Do not retain dead imports or handlers for removed
     `triage`, `router`, import/pop/update RPCs.
4. Add `backlog_item_to_proto` to
   `crates/agileplus-grpc/src/conversions.rs` using the current generated
   fields: `id.unwrap_or_default()`, `intent.to_string()`, `title`,
   `description`, `priority.to_string()`, `status.to_string()`, `source`, and
   `created_at.to_rfc3339()`. Add a focused conversion unit test.

### Risk and migration boundary

This is a medium-risk Rust service-bound change, but lower risk than modifying
`StoragePort`: only the gRPC bootstrap's storage generic must gain
`ContentStoragePort`; existing core service implementations remain on
`StoragePort`. Any alternate `start_server` caller with a storage adapter that
does not implement `ContentStoragePort` will fail at compile time and must be
adapted or kept off the integration-service bootstrap. The protobuf wire schema
does not change.

### Required verification

- `cargo fmt --check` for the changed Rust files.
- `cargo check --locked -p agileplus-grpc --bin agileplus-grpc`.
- `cargo build --locked -p agileplus-grpc --bin agileplus-grpc`.
- Existing Rust gRPC tests plus conversion tests.
- `uv run --project python --locked pytest -c python/pyproject.toml -q qe/unit qe/contract`.
- The QE contract must prove: fresh empty list, create/list filtered by
  `feature_id`, and non-empty `wp_id` returns `INVALID_ARGUMENT` without a
  persisted item.
- Ruff and bytecode-artifact checks remain green.
