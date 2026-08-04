"""Hermetic tests for the MCP server's public runtime behavior."""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock

import pytest

from agileplus_mcp import server
from agileplus_mcp.grpc_client import GrpcConnectionError


def _client() -> MagicMock:
    client = MagicMock()
    client.list_features = AsyncMock(
        return_value=[{"slug": "feature-one"}, {"slug": "Invalid Slug"}]
    )
    client.get_feature = AsyncMock(return_value={"slug": "feature-one", "state": "planned"})
    client.get_feature_state = AsyncMock(
        return_value={"state": "planned", "next_command": "implement"}
    )
    client.close = AsyncMock()
    return client


@pytest.mark.asyncio
async def test_workspace_roots_and_elicitation_use_the_configured_client(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client = _client()
    monkeypatch.setattr(server, "_client", client)

    roots = await server.get_workspace_roots()
    assert roots["roots"] == [
        {"uri": "file:///", "name": "project-root"},
        {"uri": "file://.agileplus/", "name": "agileplus-data"},
        {"uri": "file://kitty-specs/feature-one/", "name": "feature-spec-feature-one"},
        {"uri": "file://.worktrees/feature-one/", "name": "feature-worktree-feature-one"},
    ]
    elicitation = await server.elicit_feature("A coverage feature", target_branch="release")
    assert elicitation["feature_name"] == "A coverage feature"
    assert len(elicitation["questions"]) == 5
    clarification = await server.elicit_clarify("feature-one")
    assert clarification["current_state"] == "planned"


@pytest.mark.asyncio
async def test_sampling_entrypoints_delegate_to_initialized_handler(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sampling = MagicMock()
    sampling.auto_triage = AsyncMock(return_value={"severity": "info"})
    sampling.governance_pre_check = AsyncMock(return_value={"ready": True})
    sampling.generate_retrospective = AsyncMock(return_value={"total_transitions": 2})
    monkeypatch.setattr(server, "_sampling", sampling)

    assert await server.sample_triage("feature-one", "all clear") == {"severity": "info"}
    assert await server.sample_governance_check("feature-one", "planned->implementing") == {
        "ready": True
    }
    assert await server.sample_retrospective("feature-one") == {"total_transitions": 2}


@pytest.mark.asyncio
async def test_sampling_entrypoints_fail_closed_without_startup(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(server, "_sampling", None)
    with pytest.raises(RuntimeError, match="not initialised"):
        await server.sample_triage("feature-one", "all clear")


@pytest.mark.asyncio
async def test_startup_and_shutdown_register_tools_and_close_client(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client = _client()
    client.connect = AsyncMock()
    constructor = MagicMock(return_value=client)
    feature_registration = MagicMock()
    governance_registration = MagicMock()
    status_registration = MagicMock()
    monkeypatch.setattr(server, "AgilePlusCoreClient", constructor)
    monkeypatch.setattr(server.features_module, "register_tools", feature_registration)
    monkeypatch.setattr(server.governance_module, "register_tools", governance_registration)
    monkeypatch.setattr(server.status_module, "register_tools", status_registration)
    monkeypatch.setattr(server, "_client", None)
    monkeypatch.setattr(server, "_sampling", None)

    await server.startup("127.0.0.1:50051")
    constructor.assert_called_once_with("127.0.0.1:50051")
    feature_registration.assert_called_once_with(server.mcp, client)
    governance_registration.assert_called_once_with(server.mcp, client)
    status_registration.assert_called_once_with(server.mcp, client)
    await server.shutdown()
    client.close.assert_awaited_once()
    assert server._client is None


@pytest.mark.asyncio
async def test_startup_retains_client_when_initial_connection_is_unavailable(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client = _client()
    client.connect = AsyncMock(side_effect=GrpcConnectionError("not listening"))
    monkeypatch.setattr(server, "AgilePlusCoreClient", MagicMock(return_value=client))
    monkeypatch.setattr(server.features_module, "register_tools", MagicMock())
    monkeypatch.setattr(server.governance_module, "register_tools", MagicMock())
    monkeypatch.setattr(server.status_module, "register_tools", MagicMock())
    monkeypatch.setattr(server, "_client", None)
    monkeypatch.setattr(server, "_sampling", None)

    await server.startup("127.0.0.1:50051")
    assert server._client is client
    await server.shutdown()
