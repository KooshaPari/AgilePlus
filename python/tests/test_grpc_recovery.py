"""Wire-level recovery and failure controls for the canonical client."""

from __future__ import annotations

import asyncio
import socket
from typing import override
from unittest.mock import AsyncMock

import grpc
import pytest
from fastmcp import Client, FastMCP

from agileplus_mcp.compatibility import register_compatibility_tools
from agileplus_mcp.grpc_client import AgilePlusCoreClient, GrpcCallError, GrpcConnectionError
from agileplus_proto.gen.agileplus.v1 import core_pb2, core_pb2_grpc


def unused_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


class ReadableCore(core_pb2_grpc.AgilePlusCoreServiceServicer):
    @override
    async def ListFeatures(self, request, context):
        return core_pb2.ListFeaturesResponse()


@pytest.mark.asyncio
async def test_late_core_and_restart_recover_same_channel_and_health() -> None:
    address = f"127.0.0.1:{unused_port()}"
    client = AgilePlusCoreClient(address, readiness_timeout=0.03, rpc_timeout=0.2)
    client._retry_delay = 0.01
    server = grpc.aio.server()
    core_pb2_grpc.add_AgilePlusCoreServiceServicer_to_server(ReadableCore(), server)
    assert server.add_insecure_port(address)
    app = FastMCP("recovery-control")
    register_compatibility_tools(app, client)
    try:
        with pytest.raises(GrpcConnectionError):
            await client.connect()
        channel = client._channel
        assert client._stub is not None
        async with Client(app) as mcp:
            failed = (await mcp.call_tool("health_check")).data
            assert failed["live"] is True and failed["ready"] is False
            assert failed["mcp_server"] == "ok" and failed["grpc_core"] == "unreachable"
            await server.start()
            await client.connect()
            assert client._channel is channel
            recovered = (await mcp.call_tool("health_check")).data
            assert recovered["ready"] is True and recovered["status"] == "healthy"
            assert (await mcp.call_tool("list_features")).data == []
    finally:
        await client.close()
        await server.stop(0)


@pytest.mark.asyncio
async def test_rpc_deadline_cancels_hung_call() -> None:
    cancelled = asyncio.Event()

    async def hangs():
        try:
            await asyncio.Event().wait()
        finally:
            cancelled.set()

    client = AgilePlusCoreClient(rpc_timeout=0.03)
    with pytest.raises(GrpcConnectionError, match="deadline"):
        await asyncio.wait_for(client._call_with_retry(hangs), timeout=0.5)
    assert cancelled.is_set()


@pytest.mark.asyncio
async def test_unavailable_retry_budget_and_no_final_backoff() -> None:
    error = grpc.aio.AioRpcError(
        grpc.StatusCode.UNAVAILABLE, grpc.aio.Metadata(), grpc.aio.Metadata(), "unavailable"
    )
    call = AsyncMock(side_effect=error)
    client = AgilePlusCoreClient(rpc_timeout=0.04)
    client._retry_delay = 1
    with pytest.raises(GrpcConnectionError, match="deadline"):
        await asyncio.wait_for(client._call_with_retry(call), timeout=0.5)
    assert call.await_count == 1


@pytest.mark.asyncio
async def test_malformed_wire_response_cannot_report_ready() -> None:
    # A deliberately invalid protobuf payload, not a Python object mock.
    server = grpc.aio.server()
    handler = grpc.method_handlers_generic_handler(
        "agileplus.v1.AgilePlusCoreService",
        {"ListFeatures": grpc.unary_unary_rpc_method_handler(lambda request, context: b"\xff")},
    )
    server.add_generic_rpc_handlers((handler,))
    port = server.add_insecure_port("127.0.0.1:0")
    client = AgilePlusCoreClient(f"127.0.0.1:{port}", rpc_timeout=0.2)
    app = FastMCP("malformed-control")
    register_compatibility_tools(app, client)
    await server.start()
    try:
        await client.connect()
        with pytest.raises(GrpcCallError):
            await client.list_features()
        async with Client(app) as mcp:
            failed = (await mcp.call_tool("health_check")).data
            assert failed["ready"] is False
            assert failed["status"] == "unhealthy"
    finally:
        await client.close()
        await server.stop(0)


@pytest.mark.asyncio
async def test_missing_feature_message_is_not_a_success() -> None:
    client = AgilePlusCoreClient()
    from unittest.mock import MagicMock

    client._stub = MagicMock()
    client._stub.GetFeature = AsyncMock(return_value=core_pb2.GetFeatureResponse())
    with pytest.raises(GrpcCallError, match="omitted feature"):
        await client.get_feature("missing")


@pytest.mark.asyncio
async def test_audit_stream_deadline_cancels_and_classifies() -> None:
    from unittest.mock import MagicMock

    cancelled = asyncio.Event()

    async def hanging_entries():
        try:
            await asyncio.Event().wait()
            yield core_pb2.GetAuditTrailResponse()
        finally:
            cancelled.set()

    client = AgilePlusCoreClient(rpc_timeout=0.03)
    client._stub = MagicMock()
    client._stub.GetAuditTrail.side_effect = lambda *args, **kwargs: hanging_entries()
    with pytest.raises(GrpcConnectionError, match="deadline"):
        await asyncio.wait_for(client.get_audit_trail("engine"), timeout=0.5)
    assert cancelled.is_set()


@pytest.mark.asyncio
async def test_context_manager_closes_after_readiness_failure(monkeypatch) -> None:
    from agileplus_mcp.grpc_client import connect_client

    close = AsyncMock()
    monkeypatch.setattr(AgilePlusCoreClient, "close", close)
    monkeypatch.setattr(
        AgilePlusCoreClient, "connect", AsyncMock(side_effect=GrpcConnectionError())
    )
    with pytest.raises(GrpcConnectionError):
        async with connect_client():
            pytest.fail("unready context should not yield")
    close.assert_awaited_once()
