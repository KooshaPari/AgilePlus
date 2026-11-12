"""Canonical integration-service contracts for the backlog client."""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock

import grpc
import pytest

from agileplus_mcp.grpc_client import AgilePlusCoreClient, GrpcCallError, GrpcConnectionError
from agileplus_proto.gen.agileplus.v1 import integrations_pb2


def _client_with_integrations_stub() -> tuple[AgilePlusCoreClient, MagicMock]:
    client = AgilePlusCoreClient()
    integrations = MagicMock()
    client._integrations_stub = integrations
    client._channel_target = "127.0.0.1:50051"
    return client, integrations


@pytest.mark.asyncio
async def test_backlog_create_uses_the_generated_integrations_request_shape() -> None:
    client, integrations = _client_with_integrations_stub()
    integrations.CreateBacklogItem = AsyncMock(
        return_value=integrations_pb2.CreateBacklogItemResponse(
            item=integrations_pb2.BacklogItem(
                id=9,
                type="task",
                title="Repair MCP contract",
                body="Use canonical fields",
                priority="high",
                state="triaged",
                external_ref="mcp",
            )
        )
    )

    item = await client.create_backlog_item(
        item_type="task",
        title="Repair MCP contract",
        body="Use canonical fields",
        priority="high",
        feature_id="001",
        wp_id="WP14",
        triaged_by="mcp",
    )

    assert item == {
        "id": 9,
        "type": "task",
        "title": "Repair MCP contract",
        "body": "Use canonical fields",
        "priority": "high",
        "state": "triaged",
        "external_ref": "mcp",
    }
    request = integrations.CreateBacklogItem.call_args.args[0]
    assert (request.feature_id, request.wp_id, request.triaged_by) == ("001", "WP14", "mcp")
    assert request.HasField("project_scope")


@pytest.mark.asyncio
async def test_backlog_create_does_not_retry_an_ambiguous_write() -> None:
    client, integrations = _client_with_integrations_stub()
    integrations.CreateBacklogItem = AsyncMock(
        side_effect=grpc.aio.AioRpcError(
            grpc.StatusCode.UNAVAILABLE,
            initial_metadata=grpc.aio.Metadata(),
            trailing_metadata=grpc.aio.Metadata(),
            details="connection lost after ambiguous write",
        )
    )
    client._call_with_retry = AsyncMock(side_effect=AssertionError("create must not retry"))

    with pytest.raises(GrpcConnectionError, match="ambiguous write"):
        await client.create_backlog_item(item_type="task", title="One write")

    integrations.CreateBacklogItem.assert_awaited_once()
    client._call_with_retry.assert_not_awaited()


@pytest.mark.asyncio
async def test_backlog_create_maps_non_retryable_error_without_retry() -> None:
    client, integrations = _client_with_integrations_stub()
    integrations.CreateBacklogItem = AsyncMock(
        side_effect=grpc.aio.AioRpcError(
            grpc.StatusCode.INVALID_ARGUMENT,
            initial_metadata=grpc.aio.Metadata(),
            trailing_metadata=grpc.aio.Metadata(),
            details="invalid backlog item",
        )
    )
    client._call_with_retry = AsyncMock(side_effect=AssertionError("create must not retry"))

    with pytest.raises(GrpcCallError, match="invalid backlog item") as caught:
        await client.create_backlog_item(item_type="task", title="Invalid")

    assert caught.value.code == grpc.StatusCode.INVALID_ARGUMENT
    integrations.CreateBacklogItem.assert_awaited_once()
    client._call_with_retry.assert_not_awaited()


@pytest.mark.asyncio
async def test_backlog_list_and_promote_use_only_declared_rpc_fields() -> None:
    client, integrations = _client_with_integrations_stub()
    integrations.ListBacklog = AsyncMock(
        return_value=integrations_pb2.ListBacklogResponse(
            items=[integrations_pb2.BacklogItem(id=3, type="bug", title="Failure")]
        )
    )
    integrations.PromoteBacklogItem = AsyncMock(
        return_value=integrations_pb2.PromoteBacklogItemResponse(
            success=True, created_entity_id="FR-3", message="promoted"
        )
    )

    items = await client.list_backlog(type_filter="bug", state_filter="triaged", feature_slug="001")
    promotion = await client.promote_backlog_item(3, target_type="feature")

    assert items == [
        {
            "id": 3,
            "type": "bug",
            "title": "Failure",
            "body": "",
            "priority": "",
            "state": "",
            "external_ref": "",
        }
    ]
    assert promotion == {"success": True, "created_entity_id": "FR-3", "message": "promoted"}
    list_request = integrations.ListBacklog.call_args.args[0]
    assert list_request.HasField("project_scope")
    promote_request = integrations.PromoteBacklogItem.call_args.args[0]
    assert (list_request.type_filter, list_request.state_filter, list_request.feature_slug) == (
        "bug",
        "triaged",
        "001",
    )
    assert (promote_request.backlog_item_id, promote_request.target_type) == (3, "feature")


@pytest.mark.asyncio
async def test_backlog_requests_attach_configured_queue_credential(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client, integrations = _client_with_integrations_stub()
    monkeypatch.setenv("AGILEPLUS_API_KEY", "fixture-token")
    integrations.CreateBacklogItem = AsyncMock(
        return_value=integrations_pb2.CreateBacklogItemResponse()
    )
    integrations.ListBacklog = AsyncMock(return_value=integrations_pb2.ListBacklogResponse())
    await client.create_backlog_item("task", "test")
    await client.list_backlog()
    assert integrations.CreateBacklogItem.call_args.kwargs["metadata"] == (
        ("authorization", "Bearer fixture-token"),
    )
    assert integrations.ListBacklog.call_args.kwargs["metadata"] == (
        ("authorization", "Bearer fixture-token"),
    )
    monkeypatch.delenv("AGILEPLUS_API_KEY")
    await client.list_backlog()
    assert integrations.ListBacklog.call_args.kwargs["metadata"] == ()


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "target",
    [
        "10.0.0.1:50051",
        "localhost:50051",
        "localhost.evil:50051",
        "127.0.0.1.evil:50051",
        "dns:///127.0.0.1:50051",
        "127.0.0.1:50051@evil:1234",
        "[::ffff:10.0.0.1]:50051",
        "127.0.0.1",
        "127.0.0.1:99999",
    ],
)
async def test_queue_credential_refuses_non_numeric_loopback_target(
    monkeypatch: pytest.MonkeyPatch, target: str, caplog: pytest.LogCaptureFixture
) -> None:
    client, integrations = _client_with_integrations_stub()
    client._channel_target = target
    monkeypatch.setenv("AGILEPLUS_API_KEY", "sensitive-fixture-token")
    integrations.CreateBacklogItem = AsyncMock()
    integrations.ListBacklog = AsyncMock()
    with pytest.raises(GrpcConnectionError, match="numeric loopback"):
        await client.create_backlog_item("task", "test")
    with pytest.raises(GrpcConnectionError, match="numeric loopback"):
        await client.list_backlog()
    integrations.CreateBacklogItem.assert_not_awaited()
    integrations.ListBacklog.assert_not_awaited()
    assert "sensitive-fixture-token" not in caplog.text


@pytest.mark.asyncio
@pytest.mark.parametrize("target", ["127.0.0.1:50051", "[::1]:50051"])
async def test_queue_credential_accepts_numeric_loopback(
    monkeypatch: pytest.MonkeyPatch, target: str
) -> None:
    client, integrations = _client_with_integrations_stub()
    client._channel_target = target
    monkeypatch.setenv("AGILEPLUS_API_KEY", "fixture-token")
    integrations.ListBacklog = AsyncMock(return_value=integrations_pb2.ListBacklogResponse())
    assert await client.list_backlog() == []
    integrations.ListBacklog.assert_awaited_once()


@pytest.mark.asyncio
async def test_default_channel_uses_numeric_loopback_and_disables_http_proxy(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client = AgilePlusCoreClient()
    channel = MagicMock()
    channel.channel_ready = AsyncMock()
    connect = MagicMock(return_value=channel)
    monkeypatch.setattr(grpc.aio, "insecure_channel", connect)
    await client.connect()
    connect.assert_called_once_with("127.0.0.1:50051", options=(("grpc.enable_http_proxy", 0),))
