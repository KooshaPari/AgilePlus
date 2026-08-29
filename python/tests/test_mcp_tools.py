"""Direct behavior tests for the registered MCP tool handlers."""

from __future__ import annotations

from collections.abc import AsyncIterator
from unittest.mock import AsyncMock, MagicMock

import pytest
from fastmcp import FastMCP

from agileplus_mcp import server
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
    client.promote_backlog_item = AsyncMock(return_value={"success": True, "message": "promoted"})

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
async def test_queue_tools_use_the_canonical_create_list_and_promote_contract() -> None:
    mcp = FastMCP("queue-tools")
    client = _client()
    queue.register_tools(mcp, client)

    added = await (await _tool(mcp, "agileplus_queue_add"))(
        "Queue item", body="Canonical body", feature_id="feature-one", wp_id="WP14"
    )
    assert added["item"]["id"] == 2
    assert (await (await _tool(mcp, "agileplus_queue_list"))(item_type="task", state="triaged"))[
        "items"
    ] == [{"id": 2}]
    assert (await (await _tool(mcp, "agileplus_queue_promote"))(2, "feature"))["success"]
    client.create_backlog_item.assert_awaited_once_with(
        item_type="task",
        title="Queue item",
        body="Canonical body",
        priority="",
        feature_id="feature-one",
        wp_id="WP14",
        triaged_by="mcp",
    )
    client.list_backlog.assert_awaited_once_with(
        type_filter="task", state_filter="triaged", feature_slug=None
    )


@pytest.mark.asyncio
async def test_canonical_compatibility_tools_round_trip_to_the_grpc_client() -> None:
    mcp = FastMCP("compatibility-tools")
    client = _client()
    client.list_features = AsyncMock(
        return_value=[
            {"slug": "feature-one", "state": "planned"},
            {"slug": "feature-two", "state": "planned"},
            {"slug": "feature-three"},
        ]
    )
    client.list_work_packages = AsyncMock(return_value=[{"sequence": 1, "state": "doing"}])
    client.get_audit_trail = AsyncMock(
        return_value=[{"id": 1, "timestamp": "2026-08-29T01:00:00Z"}]
    )
    server.register_compatibility_tools(mcp, client)

    assert (await (await _tool(mcp, "health_check"))())["grpc_core"] == "ok"
    assert len(await (await _tool(mcp, "list_features"))("planned")) == 3
    assert (await (await _tool(mcp, "get_feature"))("feature-one"))["slug"] == "feature-one"
    assert await (await _tool(mcp, "get_work_packages"))("feature-one") == [
        {"sequence": 1, "state": "doing"}
    ]
    assert (await (await _tool(mcp, "get_work_package"))("feature-one", "WP01"))["sequence"] == 1
    assert (await (await _tool(mcp, "get_tasks"))("feature-one"))["error"] == "not_implemented"
    assert (await (await _tool(mcp, "get_metrics"))())["capability"] == "metrics"
    assert (await (await _tool(mcp, "get_governance_rules"))())["capability"] == (
        "governance_rules"
    )
    assert (await (await _tool(mcp, "check_governance"))("feature-one"))["passed"]
    assert (
        await (await _tool(mcp, "check_governance"))(
            "feature-one", transition="planned->implementing"
        )
    )["passed"]
    assert client.check_governance_gate.await_args_list[-2].args == ("feature-one", "")
    assert client.check_governance_gate.await_args_list[-1].args == (
        "feature-one",
        "planned->implementing",
    )
    assert await (await _tool(mcp, "get_audit_trail"))("feature-one", 1) == [
        {"id": 1, "timestamp": "2026-08-29T01:00:00Z"}
    ]
    assert (await (await _tool(mcp, "verify_audit_chain"))("feature-one"))["valid"]
    dashboard = await (await _tool(mcp, "get_dashboard"))()
    assert dashboard["feature_counts"] == {"planned": 2, "unknown": 1}
    assert len(dashboard["active_work_packages"]) == 3
    assert len(dashboard["recent_audit_entries"]) == 3


@pytest.mark.asyncio
async def test_canonical_health_reports_grpc_failures() -> None:
    mcp = FastMCP("unhealthy-compatibility-tools")
    client = _client()
    client.list_features = AsyncMock(side_effect=RuntimeError("core offline"))
    server.register_compatibility_tools(mcp, client)

    health = await (await _tool(mcp, "health_check"))()

    assert health["status"] == "unhealthy"
    assert health["grpc_core"] == "unreachable"
    assert health["error"] == "core offline"


@pytest.mark.asyncio
@pytest.mark.parametrize("wp_id", ["1", "WP", "WP0", "wp01", "WP-1"])
async def test_canonical_get_work_package_rejects_invalid_ids(wp_id: str) -> None:
    mcp = FastMCP("invalid-work-package-id")
    client = _client()
    server.register_compatibility_tools(mcp, client)

    with pytest.raises(ValueError, match="wp_id"):
        await (await _tool(mcp, "get_work_package"))("feature-one", wp_id)

    client.get_work_package_status.assert_not_awaited()


def test_http_transport_requires_loopback(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("AGILEPLUS_MCP_HOST", "0.0.0.0")  # noqa: S104 - rejection fixture
    with pytest.raises(ValueError, match="loopback"):
        server._transport_kwargs("http")

    monkeypatch.setenv("AGILEPLUS_MCP_HOST", "127.0.0.1")
    monkeypatch.setenv("AGILEPLUS_MCP_PORT", "9876")
    assert server._transport_kwargs("http") == {
        "host": "127.0.0.1",
        "port": 9876,
        "path": "/mcp",
    }
    assert server._transport_kwargs("stdio") == {}


@pytest.mark.asyncio
async def test_canonical_governance_rejects_ambiguous_transition() -> None:
    mcp = FastMCP("governance-transition-validation")
    client = _client()
    server.register_compatibility_tools(mcp, client)

    with pytest.raises(ValueError, match="from->to"):
        await (await _tool(mcp, "check_governance"))("feature-one", transition="implementing")

    client.check_governance_gate.assert_not_awaited()


@pytest.mark.asyncio
async def test_repeated_startup_replaces_client_used_by_registered_tools(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    app = FastMCP("repeated-startup")
    first = _client()
    second = _client()
    clients = iter((first, second))

    monkeypatch.setattr(server, "mcp", app)
    monkeypatch.setattr(server, "AgilePlusCoreClient", lambda _address: next(clients))
    first.connect = AsyncMock()
    first.close = AsyncMock()
    second.connect = AsyncMock()
    second.close = AsyncMock()

    await server.startup("127.0.0.1:50051")
    await server.startup("127.0.0.1:50051")
    await (await _tool(app, "get_feature"))("feature-one")

    first.close.assert_awaited_once()
    first.get_feature.assert_not_awaited()
    second.get_feature.assert_awaited_once_with("feature-one")
    await server.shutdown()
