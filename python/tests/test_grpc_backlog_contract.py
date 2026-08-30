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
async def test_backlog_promote_propagates_unimplemented_without_false_success() -> None:
    client, integrations = _client_with_integrations_stub()
    integrations.PromoteBacklogItem = AsyncMock(
        side_effect=grpc.aio.AioRpcError(
            grpc.StatusCode.UNIMPLEMENTED,
            initial_metadata=grpc.aio.Metadata(),
            trailing_metadata=grpc.aio.Metadata(),
            details="promotion requires atomic mutation",
        )
    )

    with pytest.raises(GrpcCallError, match="atomic mutation") as caught:
        await client.promote_backlog_item(3, target_type="feature")

    assert caught.value.code == grpc.StatusCode.UNIMPLEMENTED
    integrations.PromoteBacklogItem.assert_awaited_once()


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
    promote_request = integrations.PromoteBacklogItem.call_args.args[0]
    assert (list_request.type_filter, list_request.state_filter, list_request.feature_slug) == (
        "bug",
        "triaged",
        "001",
    )
    assert (promote_request.backlog_item_id, promote_request.target_type) == (3, "feature")
