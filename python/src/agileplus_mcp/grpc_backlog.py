"""Backlog RPC helpers for the AgilePlus gRPC client."""

from __future__ import annotations

import ipaddress
import os
from collections.abc import Awaitable, Callable
from typing import Any, Protocol, cast
from urllib.parse import urlsplit


class _BacklogGrpcHost(Protocol):
    """Host operations supplied by ``AgilePlusCoreClient``."""

    _channel_target: str | None

    def _require_integrations_stub(self) -> Any: ...

    def _project_scope(self) -> Any: ...

    async def _call_with_retry(self, coro_factory: Callable[[], Awaitable[Any]]) -> Any: ...


class AgilePlusBacklogGrpcMixin:
    """Backlog helpers that match the generated IntegrationsService contract."""

    async def create_backlog_item(
        self,
        item_type: str,
        title: str,
        body: str = "",
        priority: str = "",
        feature_id: str = "",
        wp_id: str = "",
        triaged_by: str = "mcp",
    ) -> dict[str, Any]:
        """Create a backlog item via the integrations service."""
        import grpc

        from agileplus_proto.gen.agileplus.v1 import integrations_pb2  # type: ignore[import]

        host = cast(_BacklogGrpcHost, self)
        stub = host._require_integrations_stub()
        request = integrations_pb2.CreateBacklogItemRequest(
            type=item_type,
            title=title,
            body=body,
            priority=priority,
            feature_id=feature_id,
            wp_id=wp_id,
            triaged_by=triaged_by,
            project_scope=host._project_scope(),
        )
        # Create has no idempotency key. Retrying after an ambiguous transport
        # failure could duplicate an item that the server already committed.
        try:
            response = await stub.CreateBacklogItem(
                request, metadata=_queue_metadata(host._channel_target)
            )
        except grpc.aio.AioRpcError as exc:
            # Import lazily to avoid a module cycle while grpc_client defines
            # the public exceptions after importing this mixin.
            from agileplus_mcp.grpc_client import GrpcCallError, GrpcConnectionError

            if exc.code() in (grpc.StatusCode.UNAVAILABLE, grpc.StatusCode.DEADLINE_EXCEEDED):
                raise GrpcConnectionError(
                    f"CreateBacklogItem outcome is ambiguous: {exc.details()}"
                ) from exc
            raise GrpcCallError(exc.code(), exc.details()) from exc
        return self._backlog_item_to_dict(response.item)

    async def list_backlog(
        self,
        type_filter: str | None = None,
        state_filter: str | None = None,
        feature_slug: str | None = None,
    ) -> list[dict[str, Any]]:
        """List backlog items via the integrations service."""
        from agileplus_proto.gen.agileplus.v1 import integrations_pb2  # type: ignore[import]

        host = cast(_BacklogGrpcHost, self)
        stub = host._require_integrations_stub()
        request = integrations_pb2.ListBacklogRequest(
            type_filter=type_filter or "",
            state_filter=state_filter or "",
            feature_slug=feature_slug or "",
            project_scope=host._project_scope(),
        )
        response = await host._call_with_retry(
            lambda: stub.ListBacklog(request, metadata=_queue_metadata(host._channel_target))
        )
        return [self._backlog_item_to_dict(item) for item in response.items]

    async def promote_backlog_item(self, backlog_item_id: int, target_type: str) -> dict[str, Any]:
        """Promote one triaged item using the canonical integrations RPC."""
        from agileplus_proto.gen.agileplus.v1 import integrations_pb2  # type: ignore[import]

        host = cast(_BacklogGrpcHost, self)
        stub = host._require_integrations_stub()
        request = integrations_pb2.PromoteBacklogItemRequest(
            backlog_item_id=backlog_item_id, target_type=target_type
        )
        response = await host._call_with_retry(lambda: stub.PromoteBacklogItem(request))
        return {
            "success": response.success,
            "created_entity_id": response.created_entity_id,
            "message": response.message,
        }

    @staticmethod
    def _backlog_item_to_dict(item: Any) -> dict[str, Any]:
        return {
            "id": item.id,
            "type": item.type,
            "title": item.title,
            "body": item.body,
            "priority": item.priority,
            "state": item.state,
            "external_ref": item.external_ref,
        }


def _queue_metadata(channel_target: str | None) -> tuple[tuple[str, str], ...]:
    token = os.environ.get("AGILEPLUS_API_KEY", "")
    if not token:
        return ()
    try:
        if channel_target is None or any(c in channel_target for c in "/?#@%"):
            raise ValueError("invalid target")
        parsed = urlsplit("//" + channel_target)
        if parsed.hostname is None or parsed.port is None or not 1 <= parsed.port <= 65535:
            raise ValueError("invalid target")
        if not ipaddress.ip_address(parsed.hostname).is_loopback:
            raise ValueError("non-loopback target")
    except ValueError:
        from agileplus_mcp.grpc_client import GrpcConnectionError

        raise GrpcConnectionError(
            "Queue credentials require a numeric loopback gRPC target"
        ) from None
    return (("authorization", f"Bearer {token}"),)
