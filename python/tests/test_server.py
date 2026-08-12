"""Direct behavior tests for MCP server resources, sampling, and lifecycle boundaries."""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock

import pytest

from agileplus_mcp import server
from agileplus_mcp.grpc_client import GrpcConnectionError


@pytest.fixture(autouse=True)
def _restore_server_state():
    original_client = server._client
    original_sampling = server._sampling
    try:
        yield
    finally:
        server._client = original_client
        server._sampling = original_sampling


def _client() -> MagicMock:
    client = MagicMock()
    client.list_features = AsyncMock(
        return_value=[{"slug": "feature-one"}, {"slug": "invalid slug"}]
    )
    client.get_feature = AsyncMock(return_value={"slug": "feature-one", "state": "planned"})
    client.get_feature_state = AsyncMock(
        return_value={"state": "planned", "next_command": "implement"}
    )
    client.close = AsyncMock()
    return client


@pytest.mark.asyncio
async def test_workspace_roots_include_only_valid_feature_boundaries(monkeypatch) -> None:
    client = _client()
    monkeypatch.setattr(server, "_client", client)

    roots = await server.get_workspace_roots()

    assert roots["roots"][:2] == [
        {"uri": "file:///", "name": "project-root"},
        {"uri": "file://.agileplus/", "name": "agileplus-data"},
    ]
    assert roots["roots"][2:] == [
        {"uri": "file://kitty-specs/feature-one/", "name": "feature-spec-feature-one"},
        {"uri": "file://.worktrees/feature-one/", "name": "feature-worktree-feature-one"},
    ]


@pytest.mark.asyncio
async def test_elicit_and_sampling_handlers_validate_and_delegate(monkeypatch) -> None:
    client = _client()
    sampling = MagicMock()
    sampling.auto_triage = AsyncMock(return_value={"severity": "warning"})
    sampling.governance_pre_check = AsyncMock(return_value={"ready": False, "blockers": ["review"]})
    sampling.generate_retrospective = AsyncMock(return_value={"audit_valid": True})
    monkeypatch.setattr(server, "_client", client)
    monkeypatch.setattr(server, "_sampling", sampling)

    elicitation = await server.elicit_feature("Coverage gate", target_branch="release")
    assert elicitation["feature_name"] == "Coverage gate"
    assert len(elicitation["questions"]) == 5
    assert (await server.elicit_clarify("feature-one"))["current_state"] == "planned"
    assert await server.sample_triage("feature-one", "warning: cache cold") == {
        "severity": "warning"
    }
    assert await server.sample_governance_check("feature-one", "planned->validated") == {
        "ready": False,
        "blockers": ["review"],
    }
    assert await server.sample_retrospective("feature-one") == {"audit_valid": True}

    with pytest.raises(ValueError, match="feature_slug"):
        await server.sample_triage("Feature One", "output")
    with pytest.raises(ValueError, match="agent_output"):
        await server.sample_triage("feature-one", "x" * 1_000_001)


@pytest.mark.asyncio
async def test_sampling_tools_fail_before_initialisation(monkeypatch) -> None:
    monkeypatch.setattr(server, "_sampling", None)

    with pytest.raises(RuntimeError, match="Sampling handler not initialised"):
        await server.sample_triage("feature-one", "output")
    with pytest.raises(RuntimeError, match="Sampling handler not initialised"):
        await server.sample_governance_check("feature-one", "planned->validated")
    with pytest.raises(RuntimeError, match="Sampling handler not initialised"):
        await server.sample_retrospective("feature-one")


@pytest.mark.asyncio
async def test_startup_keeps_client_available_after_grpc_connection_error(monkeypatch) -> None:
    client = _client()
    client.connect = AsyncMock(side_effect=GrpcConnectionError("unavailable"))
    constructor = MagicMock(return_value=client)
    registrations = [
        monkeypatch.setattr(module, "register_tools", MagicMock())
        for module in (server.features_module, server.governance_module, server.status_module)
    ]
    assert registrations == [None, None, None]
    monkeypatch.setattr(server, "AgilePlusCoreClient", constructor)

    await server.startup("127.0.0.1:1")

    assert server._get_client() is client
    client.connect.assert_awaited_once()
    assert isinstance(server._sampling, server.SamplingHandler)


@pytest.mark.asyncio
async def test_shutdown_closes_client_and_clears_global_state(monkeypatch) -> None:
    client = _client()
    monkeypatch.setattr(server, "_client", client)

    await server.shutdown()

    client.close.assert_awaited_once()
    assert server._client is None
