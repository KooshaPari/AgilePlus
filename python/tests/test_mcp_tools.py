"""Direct behavior tests for the registered MCP tool handlers."""

from __future__ import annotations

from collections.abc import AsyncIterator
from unittest.mock import AsyncMock, MagicMock

import pytest
from fastmcp import FastMCP

from agileplus_mcp.tools import features, governance, queue, status


def _client() -> MagicMock:
    client = MagicMock()
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

    client.stream_agent_events = events
    return client


async def _tool(mcp: FastMCP, name: str):
    return (await mcp.get_tool(name)).fn


@pytest.mark.asyncio
async def test_feature_and_governance_tools_invoke_validated_client_operations() -> None:
    mcp = FastMCP("feature-tools")
    client = _client()
    features.register_tools(mcp, client)
    governance.register_tools(mcp, client)

    assert (await (await _tool(mcp, "agileplus_specify"))("feature-one"))["status"] == "success"
    await (await _tool(mcp, "agileplus_specify"))(
        "feature-one", from_file="kitty-specs/feature-one/spec.md", target_branch="release"
    )
    await (await _tool(mcp, "agileplus_research"))("feature-one")
    await (await _tool(mcp, "agileplus_plan"))("feature-one")
    await (await _tool(mcp, "agileplus_implement"))("feature-one", wp_id="WP01")
    await (await _tool(mcp, "agileplus_validate"))("feature-one", skip_policies=True)
    assert (
        await (await _tool(mcp, "agileplus_check_governance_gate"))(
            "feature-one", "planned->implementing"
        )
    )["passed"]
    audit = await (await _tool(mcp, "agileplus_get_audit_trail"))(
        "feature-one", verify=True, after_id=1
    )
    assert audit["verification"] == {"valid": True}
    assert (await (await _tool(mcp, "agileplus_verify_audit_chain"))("feature-one"))["valid"]
    client.run_command.assert_any_await("implement", feature_slug="feature-one", wp="WP01")


@pytest.mark.asyncio
async def test_feature_and_governance_tools_preserve_core_failures_and_validate_boundaries(
) -> None:
    """Handler failures remain explicit instead of being reported as success."""
    mcp = FastMCP("feature-tool-failures")
    client = _client()
    client.run_command.return_value = {
        "success": False,
        "message": "transition blocked",
        "outputs": {},
    }
    client.check_governance_gate.return_value = {
        "passed": False,
        "violations": [{"message": "evidence required"}],
    }
    client.verify_audit_chain.return_value = {"valid": False, "first_invalid_id": 7}
    features.register_tools(mcp, client)
    governance.register_tools(mcp, client)

    assert (await (await _tool(mcp, "agileplus_plan"))("feature-one"))["status"] == "error"
    assert not (
        await (await _tool(mcp, "agileplus_check_governance_gate"))(
            "feature-one", "planned->implementing"
        )
    )["passed"]
    assert not (await (await _tool(mcp, "agileplus_verify_audit_chain"))("feature-one"))["valid"]

    specify = await _tool(mcp, "agileplus_specify")
    with pytest.raises(ValueError, match="must be under"):
        await specify("feature-one", from_file="outside/spec.md")
    with pytest.raises(ValueError, match="feature_slug"):
        await (await _tool(mcp, "agileplus_plan"))("Feature One")
    with pytest.raises(ValueError, match="transition"):
        await (await _tool(mcp, "agileplus_check_governance_gate"))(
            "feature-one", "planned-implementing"
        )


@pytest.mark.asyncio
async def test_status_tools_cover_feature_work_package_and_streaming_paths() -> None:
    mcp = FastMCP("status-tools")
    client = _client()
    status.register_tools(mcp, client)

    assert (await (await _tool(mcp, "agileplus_status"))())["features"] == [{"slug": "feature-one"}]
    detailed = await (await _tool(mcp, "agileplus_status"))("feature-one")
    assert detailed["work_packages"] == [{"sequence": 1}]
    assert (await (await _tool(mcp, "agileplus_status"))("feature-one", wp_sequence=1))[
        "work_package"
    ] == {"sequence": 1}
    assert (await (await _tool(mcp, "agileplus_ship"))("feature-one", "release"))[
        "status"
    ] == "success"
    assert (await (await _tool(mcp, "agileplus_retrospective"))("feature-one"))["message"] == "done"
    events = [event async for event in (await _tool(mcp, "agileplus_stream_status"))("feature-one")]
    assert events == [{"event_type": "updated"}]


@pytest.mark.asyncio
async def test_status_tools_preserve_failed_ship_and_reject_invalid_feature_scope() -> None:
    mcp = FastMCP("status-tool-failures")
    client = _client()
    client.run_command.return_value = {
        "success": False,
        "message": "review required",
        "outputs": {},
    }
    status.register_tools(mcp, client)

    assert (await (await _tool(mcp, "agileplus_ship"))("feature-one"))["status"] == "error"
    with pytest.raises(ValueError, match="feature_slug"):
        await (await _tool(mcp, "agileplus_status"))("Feature One")
    with pytest.raises(ValueError, match="feature_slug"):
        await (await _tool(mcp, "agileplus_stream_status"))("Feature One").__anext__()


@pytest.mark.asyncio
async def test_queue_tools_cover_success_empty_and_not_found_paths() -> None:
    mcp = FastMCP("queue-tools")
    client = _client()
    queue.register_tools(mcp, client)

    added = await (await _tool(mcp, "agileplus_queue_add"))(
        "Queue item", feature_slug="feature-one", tags=["coverage"]
    )
    assert added["item"]["id"] == 2
    assert (await (await _tool(mcp, "agileplus_queue_list"))(item_type="task", limit=1))[
        "items"
    ] == [{"id": 2}]
    assert (await (await _tool(mcp, "agileplus_queue_show"))(2))["status"] == "success"
    client.get_backlog_item.return_value = None
    assert (await (await _tool(mcp, "agileplus_queue_show"))(99))["status"] == "not_found"
    assert (await (await _tool(mcp, "agileplus_queue_pop"))(count=1))["status"] == "success"
    client.pop_backlog_items.return_value = []
    assert (await (await _tool(mcp, "agileplus_queue_pop"))())["status"] == "empty"
    assert (await (await _tool(mcp, "agileplus_queue_import"))([{"title": "Imported"}]))[
        "status"
    ] == "success"
    import_tool = await _tool(mcp, "agileplus_queue_import")
    with pytest.raises(ValueError, match="requires a title"):
        await import_tool([{}])


@pytest.mark.asyncio
async def test_queue_tools_reject_invalid_input_before_mutating_backlog() -> None:
    mcp = FastMCP("queue-tool-validation")
    client = _client()
    queue.register_tools(mcp, client)

    add = await _tool(mcp, "agileplus_queue_add")
    with pytest.raises(ValueError, match="invalid item_type"):
        await add("Queue item", item_type="incident")
    with pytest.raises(ValueError, match="feature_slug"):
        await add("Queue item", feature_slug="Feature One")
    client.create_backlog_item.assert_not_awaited()

    import_tool = await _tool(mcp, "agileplus_queue_import")
    with pytest.raises(ValueError, match="batch size"):
        await import_tool([{"title": "item"}] * 101)
    client.import_backlog_items.assert_not_awaited()
