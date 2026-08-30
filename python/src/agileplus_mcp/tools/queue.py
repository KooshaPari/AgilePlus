"""Backlog / queue MCP tools."""

from __future__ import annotations

from typing import Any

from fastmcp import FastMCP

from agileplus_mcp.grpc_client import AgilePlusCoreClient
from agileplus_mcp.validation import (
    validate_item_type,
    validate_slug,
    validate_text,
)


def register_tools(mcp: FastMCP, client: AgilePlusCoreClient) -> None:
    """Register backlog / queue tools onto *mcp*."""

    @mcp.tool(name="agileplus_queue_add")
    async def queue_add(
        title: str,
        body: str = "",
        item_type: str = "task",
        priority: str = "",
        feature_id: str = "",
        wp_id: str = "",
        triaged_by: str = "mcp",
    ) -> dict[str, Any]:
        validate_text(title, "title", max_length=512)
        validate_text(body, "body")
        validate_item_type(item_type)
        if feature_id:
            validate_slug(feature_id, "feature_id")
        item = await client.create_backlog_item(
            item_type=item_type,
            title=title,
            body=body,
            priority=priority,
            feature_id=feature_id,
            wp_id=wp_id,
            triaged_by=triaged_by,
        )
        return {"status": "success", "item": item}

    @mcp.tool(name="agileplus_queue_list")
    async def queue_list(
        item_type: str = "",
        state: str = "",
        feature_slug: str = "",
    ) -> dict[str, Any]:
        items = await client.list_backlog(
            type_filter=item_type or None,
            state_filter=state or None,
            feature_slug=feature_slug or None,
        )
        return {"status": "success", "items": items}

    @mcp.tool(name="agileplus_queue_promote")
    async def queue_promote(item_id: int, target_type: str) -> dict[str, Any]:
        """Promote a triaged backlog item through the canonical RPC."""
        validate_text(target_type, "target_type", max_length=256)
        return await client.promote_backlog_item(item_id, target_type)
