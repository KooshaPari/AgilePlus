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
