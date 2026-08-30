"""Lifecycle and error-path tests for the in-process FastMCP server."""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock

import pytest

from agileplus_mcp import server


@pytest.fixture(autouse=True)
def reset_server_state(monkeypatch: pytest.MonkeyPatch) -> None:
    """Isolate module globals without creating a network connection."""
    monkeypatch.setattr(server, "_client", None)
    monkeypatch.setattr(server, "_sampling", None)


def test_get_client_reports_startup_requirement_when_connection_is_unavailable() -> None:
    with pytest.raises(RuntimeError, match="gRPC client not initialised"):
        server._get_client()


@pytest.mark.asyncio
async def test_elicit_clarify_returns_feature_state_and_questions(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client = MagicMock()
    feature = {"slug": "valid-feature", "friendly_name": "Valid Feature"}
    client.get_feature = AsyncMock(return_value=feature)
    client.get_feature_state = AsyncMock(
        return_value={"state": "specified", "next_command": "plan"}
    )
    monkeypatch.setattr(server, "_client", client)

    result = await server.elicit_clarify("valid-feature")

    assert result["feature"] == feature
    assert result["current_state"] == "specified"
    assert result["questions"] == [
        {
            "id": "blockers",
            "question": "Are there any blockers preventing moving from specified to plan?",
            "type": "text",
            "required": False,
        },
        {
            "id": "dependencies",
            "question": "Does this feature depend on any other features or external systems?",
            "type": "text",
            "required": False,
        },
        {
            "id": "timeline",
            "question": "Is there a target completion date?",
            "type": "text",
            "required": False,
        },
    ]
    client.get_feature.assert_awaited_once_with("valid-feature")
    client.get_feature_state.assert_awaited_once_with("valid-feature")


@pytest.mark.asyncio
async def test_shutdown_closes_active_client_and_clears_global(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client = MagicMock()
    client.close = AsyncMock()
    monkeypatch.setattr(server, "_client", client)

    await server.shutdown()

    client.close.assert_awaited_once_with()
    assert server._client is None


@pytest.mark.asyncio
async def test_shutdown_without_client_is_a_noop() -> None:
    await server.shutdown()
    assert server._client is None
