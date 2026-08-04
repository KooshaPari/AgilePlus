# Python coverage repair

## Goal

Make the Python coverage gate exercise hermetic tests on stable CPython 3.14,
while retaining skips solely for integration tests that require an externally
deployed AgilePlus gRPC endpoint.

## Current finding

The integration-local pytest collection hook marked every collected test as
skipped when `AGILEPLUS_GRPC_URL` was absent. The repair scopes that policy to
tests beneath `python/tests/integration/`; follow-up coverage work will use an
in-process gRPC server for real client-path validation.

The coverage configuration measures authored `agileplus_mcp` modules rather
than generated protobuf bindings. The generated bindings retain their separate
schema and compatibility controls and are not product code that Python unit
tests should inflate or suppress.
