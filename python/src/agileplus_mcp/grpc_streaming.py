"""Streaming helpers for the AgilePlus gRPC client."""

from __future__ import annotations

import asyncio
from collections.abc import AsyncIterator
from typing import Any, Protocol, cast

from agileplus_mcp.grpc_errors import GrpcCallError, GrpcConnectionError


class _StreamingGrpcHost(Protocol):
    """Host operations supplied by the concrete gRPC client."""

    def _require_stub(self) -> Any: ...

    async def connect(self) -> None: ...


class AgilePlusGrpcStreamingMixin:
    """Streaming RPC helpers for `AgilePlusCoreClient`."""

    async def stream_agent_events(self, feature_slug: str) -> AsyncIterator[dict[str, Any]]:
        """Stream agent status events from the Rust core."""
        import grpc

        from agileplus_proto.gen.agileplus.v1 import core_pb2  # type: ignore[import]

        host = cast(_StreamingGrpcHost, self)
        stub = host._require_stub()
        request = core_pb2.StreamAgentEventsRequest(feature_slug=feature_slug)

        while True:
            try:
                async for event in stub.StreamAgentEvents(request):
                    yield {
                        "event_type": event.event_type,
                        "feature_slug": event.feature_slug,
                        "wp_sequence": event.wp_sequence,
                        "agent_id": event.agent_id,
                        "payload": event.payload,
                        "timestamp": event.timestamp,
                    }
                break
            except grpc.aio.AioRpcError as exc:
                if exc.code() == grpc.StatusCode.UNAVAILABLE:
                    await asyncio.sleep(2.0)
                    try:
                        await host.connect()
                        stub = host._require_stub()
                    except GrpcConnectionError:
                        return
                else:
                    raise GrpcCallError(exc.code(), exc.details()) from exc
