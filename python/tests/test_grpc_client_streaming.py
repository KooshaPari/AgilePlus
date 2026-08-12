"""Streaming behavior tests for the AgilePlus core gRPC client."""

from __future__ import annotations

from types import SimpleNamespace
from unittest.mock import AsyncMock

import pytest

from agileplus_mcp.grpc_client import AgilePlusCoreClient, GrpcConnectionError


class _StreamingStub:
    """A small fake that models the generated server-streaming RPC shape."""

    def __init__(self) -> None:
        self.request = None

    def StreamAgentEvents(self, request):  # noqa: N802 - generated RPC naming
        self.request = request
        return self._responses()

    async def _responses(self):
        yield SimpleNamespace(
            event=SimpleNamespace(
                event_type="agent_started",
                feature_slug="coverage-gate",
                wp_sequence=14,
                agent_id="worker-7",
                payload="{\"state\":\"running\"}",
                timestamp="2026-08-12T00:00:00Z",
            )
        )


class _UnavailableThenCleanStreamingStub:
    """Models a reconnectable generated server-streaming RPC."""

    def __init__(self, grpc) -> None:
        self._grpc = grpc
        self.calls = 0

    def StreamAgentEvents(self, _request):  # noqa: N802 - generated RPC naming
        self.calls += 1
        if self.calls == 1:
            return self._unavailable()
        return self._clean_end()

    async def _unavailable(self):
        raise self._grpc.aio.AioRpcError(
            self._grpc.StatusCode.UNAVAILABLE,
            initial_metadata=self._grpc.aio.Metadata(),
            trailing_metadata=self._grpc.aio.Metadata(),
            details="stream interrupted",
        )
        yield  # pragma: no cover - preserves async-generator type

    async def _clean_end(self):
        if False:  # pragma: no cover - preserves async-generator type
            yield None


class _DeniedStreamingStub:
    """Models a terminal permission failure from the core stream."""

    def __init__(self, grpc) -> None:
        self._grpc = grpc

    def StreamAgentEvents(self, _request):  # noqa: N802 - generated RPC naming
        return self._denied()

    async def _denied(self):
        raise self._grpc.aio.AioRpcError(
            self._grpc.StatusCode.PERMISSION_DENIED,
            initial_metadata=self._grpc.aio.Metadata(),
            trailing_metadata=self._grpc.aio.Metadata(),
            details="subscription denied",
        )
        yield  # pragma: no cover - preserves async-generator type


@pytest.mark.asyncio
async def test_stream_agent_events_unwraps_generated_response_event() -> None:
    """The generated response wraps the agent event in its ``event`` field."""
    client = AgilePlusCoreClient()
    stub = _StreamingStub()
    client._stub = stub

    received = [event async for event in client.stream_agent_events("coverage-gate")]

    assert received == [
        {
            "event_type": "agent_started",
            "feature_slug": "coverage-gate",
            "wp_sequence": 14,
            "agent_id": "worker-7",
            "payload": '{"state":"running"}',
            "timestamp": "2026-08-12T00:00:00Z",
        }
    ]
    assert stub.request.feature_slug == "coverage-gate"


@pytest.mark.asyncio
async def test_stream_agent_events_reconnects_after_unavailable(monkeypatch) -> None:
    import grpc

    client = AgilePlusCoreClient()
    stub = _UnavailableThenCleanStreamingStub(grpc)
    client._stub = stub
    client.connect = AsyncMock()
    monkeypatch.setattr("agileplus_mcp.grpc_client.asyncio.sleep", AsyncMock())

    received = [event async for event in client.stream_agent_events("coverage-gate")]

    assert received == []
    assert stub.calls == 2
    client.connect.assert_awaited_once()


@pytest.mark.asyncio
async def test_stream_agent_events_stops_when_reconnect_fails(monkeypatch) -> None:
    import grpc

    client = AgilePlusCoreClient()
    client._stub = _UnavailableThenCleanStreamingStub(grpc)
    client.connect = AsyncMock(side_effect=GrpcConnectionError("core still down"))
    monkeypatch.setattr("agileplus_mcp.grpc_client.asyncio.sleep", AsyncMock())

    received = [event async for event in client.stream_agent_events("coverage-gate")]

    assert received == []
    client.connect.assert_awaited_once()


@pytest.mark.asyncio
async def test_stream_agent_events_maps_non_transient_rpc_failure() -> None:
    """The consumer receives the stable client error rather than gRPC internals."""
    import grpc

    from agileplus_mcp.grpc_client import GrpcCallError

    client = AgilePlusCoreClient()
    client._stub = _DeniedStreamingStub(grpc)

    with pytest.raises(GrpcCallError, match="PERMISSION_DENIED") as error:
        _ = [event async for event in client.stream_agent_events("coverage-gate")]

    assert error.value.code == grpc.StatusCode.PERMISSION_DENIED
