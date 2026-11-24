# agileplus-grpc

Tonic gRPC adapter layer for AgilePlus core and work-item services.

## Public API Index

- Server exports: `start_server`, `AgilePlusCoreServer`, `domain_error_to_status`.
- Modules: `conversions`, `event_bus`, `proxy`, `server`, `streaming`, `work_items`.
- Proto types come from `agileplus-proto`.

## Validation

```bash
cargo test -p agileplus-grpc
```

## Queue service trust boundary

Queue create and list requests require the configured `AGILEPLUS_API_KEY` bearer
credential and must include the core's canonical project root as UTF-8 bytes
in the `agileplus-project-root-bin` gRPC metadata field.
Missing or mismatched scope is rejected. This is repository routing protection,
not authentication; possession of the configured credential is checked separately.
Older unscoped queue clients fail closed.
Work-package associations and backlog promotion are not supported.
