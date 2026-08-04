# Research

- `AgilePlusCoreClient` uses generated grpcio async stubs from
  `agileplus_proto.gen.agileplus.v1.core_pb2_grpc`.
- The generated package exposes `AgilePlusCoreServiceServicer` and
  `add_AgilePlusCoreServiceServicer_to_server`, enabling an ephemeral
  loopback `grpc.aio.server()` fixture without Docker or a remote service.
- Stable CPython 3.14.6 installs the locked grpcio 1.80.0 wheels. The prior
  local failure was specific to the free-threaded 3.14 ABI.
- The in-process generated service test exposed and corrected a wire-contract
  bug: the streaming client must unwrap `GetAgentEventsResponse.event` before
  converting the event to a dictionary.
- `grpc_backlog.py` referenced fields and RPC messages absent from the checked-in
  integrations proto, and it plus its duplicate error, serialization, and
  streaming helper modules had no imports. These disconnected helpers were
  removed rather than hidden from coverage.
- Hosted CI initially used uv 0.5.31, which selected CPython 3.14.0a5 and then
  failed `uv sync` because the project requires stable `>=3.14`. The coverage
  workflow pins uv 0.11.29, the stable toolchain used for the passing local
  CPython 3.14.6 validation.
- The setup action is pinned to the immutable v8.1.0 commit documented by
  astral-sh/setup-uv. Its major-version tag is not published, so using `@v8`
  fails before the job starts.
- The hosted Rust coverage build compiles the non-test library under
  `-D warnings`; `synthetic_platform_health` is consumed only by the platform
  test module, so it is correctly compiled under `cfg(test)` rather than
  retained as dead production code.
- The `agileplus-events` integration smoke test targeted an unexported,
  dormant envelope model. It now exercises the supported public in-process
  event bus, including serialization and subscriber fan-out; the bus doctest
  uses its asynchronous publishing API.
