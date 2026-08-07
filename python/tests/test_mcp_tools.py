"""Protocol-level tests for the registered MCP tools."""

from __future__ import annotations

from collections.abc import AsyncIterator
from unittest.mock import AsyncMock, MagicMock

import pytest
from fastmcp import Client, FastMCP
from fastmcp.exceptions import ToolError

from agileplus_mcp.grpc_client import AgilePlusCoreClient
from agileplus_mcp.tools import features, governance, queue, status


def _client() -> MagicMock:
    client = MagicMock(spec=AgilePlusCoreClient)
    client.run_command = AsyncMock(return_value={"success": True, "message": "done", "outputs": {}})
    client.get_feature = AsyncMock(return_value={"slug": "feature-one", "state": "planned"})
    client.list_features = AsyncMock(return_value=[{"slug": "feature-one"}])
    client.get_feature_state = AsyncMock(
        return_value={"state": "planned", "next_command": "implement"}
    )
    client.list_work_packages = AsyncMock(return_value=[{"sequence": 1}])
    client.get_work_package_status = AsyncMock(return_value={"sequence": 1})
    client.check_governance_gate = AsyncMock(return_value={"passed": True, "violations": []})
    client.get_audit_trail = AsyncMock(return_value=[{"id": 1}])
    client.verify_audit_chain = AsyncMock(return_value={"valid": True})
    client.create_backlog_item = AsyncMock(return_value={"id": 2, "title": "Queue item"})
    client.list_backlog = AsyncMock(return_value=[{"id": 2}])
    client.get_backlog_item = AsyncMock(return_value={"id": 2})
    client.pop_backlog_items = AsyncMock(return_value=[{"id": 2}])
    client.import_backlog_items = AsyncMock(return_value=[{"id": 2}])

    async def events(_: str) -> AsyncIterator[dict[str, str]]:
        yield {"event_type": "updated"}

    client.stream_agent_events = MagicMock(side_effect=events)
    return client


async def _call(client: Client, name: str, arguments: dict | None = None) -> dict:
    result = await client.call_tool(name, arguments or {})
    return result.data


@pytest.mark.asyncio
async def test_feature_and_governance_tools_invoke_validated_client_operations() -> None:
    mcp = FastMCP("feature-tools")
    core = _client()
    features.register_tools(mcp, core)
    governance.register_tools(mcp, core)

    async with Client(mcp) as protocol:
        assert (
            await _call(protocol, "agileplus_specify", {"feature_slug": "feature-one"})
        )["status"] == "success"
        await _call(
            protocol,
            "agileplus_specify",
            {
                "feature_slug": "feature-one",
                "from_file": "kitty-specs/feature-one/spec.md",
                "target_branch": "release",
            },
        )
        await _call(protocol, "agileplus_research", {"feature_slug": "feature-one"})
        await _call(protocol, "agileplus_plan", {"feature_slug": "feature-one"})
        await _call(
            protocol,
            "agileplus_implement",
            {"feature_slug": "feature-one", "wp_id": "WP01"},
        )
        await _call(
            protocol,
            "agileplus_validate",
            {"feature_slug": "feature-one", "skip_policies": True},
        )
        assert (
            await _call(
                protocol,
                "agileplus_check_governance_gate",
                {"feature_slug": "feature-one", "transition": "planned->implementing"},
            )
        )["passed"]
        audit = await _call(
            protocol,
            "agileplus_get_audit_trail",
            {"feature_slug": "feature-one", "verify": True, "after_id": 1},
        )
        assert audit["verification"] == {"valid": True}
        assert (
            await _call(protocol, "agileplus_verify_audit_chain", {"feature_slug": "feature-one"})
        )["valid"]

    core.run_command.assert_any_await("specify", feature_slug="feature-one", target_branch="main")
    core.run_command.assert_any_await(
        "specify",
        feature_slug="feature-one",
        from_file="kitty-specs/feature-one/spec.md",
        target_branch="release",
    )
    core.run_command.assert_any_await("validate", feature_slug="feature-one", skip_policies="true")
    core.run_command.assert_any_await("implement", feature_slug="feature-one", wp="WP01")
    core.get_audit_trail.assert_awaited_once_with("feature-one", after_id=1)
    core.verify_audit_chain.assert_any_await("feature-one")


@pytest.mark.asyncio
async def test_status_tools_cover_feature_work_package_and_streaming_paths() -> None:
    mcp = FastMCP("status-tools")
    core = _client()
    status.register_tools(mcp, core)

    async with Client(mcp) as protocol:
        assert (await _call(protocol, "agileplus_status"))["features"] == [{"slug": "feature-one"}]
        detailed = await _call(protocol, "agileplus_status", {"feature_slug": "feature-one"})
        assert detailed["work_packages"] == [{"sequence": 1}]
        assert (
            await _call(
                protocol,
                "agileplus_status",
                {"feature_slug": "feature-one", "wp_sequence": 1},
            )
        )["work_package"] == {"sequence": 1}
        assert (
            await _call(
                protocol,
                "agileplus_ship",
                {"feature_slug": "feature-one", "target_branch": "release"},
            )
        )["status"] == "success"
        assert (
            await _call(protocol, "agileplus_retrospective", {"feature_slug": "feature-one"})
        )["message"] == "done"
        await _call(protocol, "agileplus_stream_status", {"feature_slug": "feature-one"})

    stream_tool = (await mcp.get_tool("agileplus_stream_status")).fn
    events = [event async for event in stream_tool("feature-one")]
    assert events == [{"event_type": "updated"}]

    core.list_features.assert_awaited_once_with()
    core.get_feature.assert_awaited_once_with("feature-one")
    core.list_work_packages.assert_awaited_once_with("feature-one")
    core.get_work_package_status.assert_awaited_once_with("feature-one", 1)
    core.run_command.assert_any_await("ship", feature_slug="feature-one", target_branch="release")
    core.run_command.assert_any_await("retrospective", feature_slug="feature-one")
    assert core.stream_agent_events.call_count == 2
    core.stream_agent_events.assert_called_with("feature-one")


@pytest.mark.asyncio
async def test_queue_tools_cover_success_empty_and_not_found_paths() -> None:
    mcp = FastMCP("queue-tools")
    core = _client()
    queue.register_tools(mcp, core)

    async with Client(mcp) as protocol:
        added = await _call(
            protocol,
            "agileplus_queue_add",
            {"title": "Queue item", "feature_slug": "feature-one", "tags": ["coverage"]},
        )
        assert added["item"]["id"] == 2
        assert (
            await _call(protocol, "agileplus_queue_list", {"item_type": "task", "limit": 1})
        )["items"] == [{"id": 2}]
        assert (
            await _call(protocol, "agileplus_queue_show", {"item_id": 2})
        )["status"] == "success"
        core.get_backlog_item.return_value = None
        assert (
            await _call(protocol, "agileplus_queue_show", {"item_id": 99})
        )["status"] == "not_found"
        assert (await _call(protocol, "agileplus_queue_pop", {"count": 1}))["status"] == "success"
        core.pop_backlog_items.return_value = []
        assert (await _call(protocol, "agileplus_queue_pop"))["status"] == "empty"
        assert (
            await _call(protocol, "agileplus_queue_import", {"items": [{"title": "Imported"}]})
        )["status"] == "success"
        with pytest.raises(ToolError, match="requires a title"):
            await _call(protocol, "agileplus_queue_import", {"items": [{}]})

    core.create_backlog_item.assert_awaited_once_with(
        item_type="task",
        title="Queue item",
        description="",
        priority="",
        source="mcp",
        feature_slug="feature-one",
        tags=["coverage"],
    )
    core.list_backlog.assert_awaited_once_with(
        type_filter="task",
        status_filter=None,
        priority_filter=None,
        feature_slug=None,
        source_filter=None,
        sort="priority",
        limit=1,
    )
    assert core.get_backlog_item.await_args_list[0].args == (2,)
    assert core.get_backlog_item.await_args_list[1].args == (99,)
    assert core.pop_backlog_items.await_count == 2
    core.pop_backlog_items.assert_awaited_with(count=1)
    core.import_backlog_items.assert_awaited_once_with([{"title": "Imported"}])


@pytest.mark.asyncio
async def test_server_startup_registers_queue_tools(monkeypatch: pytest.MonkeyPatch) -> None:
    from agileplus_mcp import server

    class FakeCoreClient:
        def __init__(self, _: str) -> None:
            pass

        async def connect(self) -> None:
            return None

        async def close(self) -> None:
            return None

    monkeypatch.setattr(server, "AgilePlusCoreClient", FakeCoreClient)
    await server.startup("localhost:50051")
    try:
        assert await server.mcp.get_tool("agileplus_queue_add")
        assert await server.mcp.get_tool("agileplus_queue_list")
        assert await server.mcp.get_tool("agileplus_queue_show")
        assert await server.mcp.get_tool("agileplus_queue_pop")
        assert await server.mcp.get_tool("agileplus_queue_import")
    finally:
        await server.shutdown()
