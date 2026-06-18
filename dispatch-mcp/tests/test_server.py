"""Tests for dispatch_mcp.server.

Note: These tests mock httpx.Client to avoid requiring a live OmniRoute server.
"""

from __future__ import annotations

from unittest.mock import MagicMock, patch

import pytest


class TestCallOmniroute:
    """Tests for _call_omniroute via dispatch_custom and dispatch_health."""

    def test_dispatch_custom_success(self) -> None:
        with (
            patch.dict("os.environ", {"OMNIROUTE_URL": "http://localhost:8080"}),
            patch("dispatch_mcp.server.httpx.Client") as mock_client_cls,
        ):
            mock_response = MagicMock()
            mock_response.json.return_value = {
                "ok": True,
                "tier": "worker",
                "message": "hello",
            }
            mock_response.raise_for_status = MagicMock()
            mock_client = MagicMock()
            mock_client.__enter__ = MagicMock(return_value=mock_client)
            mock_client.__exit__ = MagicMock(return_value=False)
            mock_client.post.return_value = mock_response
            mock_client_cls.return_value = mock_client

            from dispatch_mcp.server import dispatch_custom

            result = dispatch_custom("worker", "hello")
            mock_client.post.assert_called_once()
            call_args = mock_client.post.call_args
            assert "dispatch" in call_args[0][0]
            assert call_args[1]["json"] == {"tier": "worker", "message": "hello"}
            assert result == {"ok": True, "tier": "worker", "message": "hello"}

    def test_dispatch_custom_rejects_oversized_response(self) -> None:
        with (
            patch.dict("os.environ", {"OMNIROUTE_URL": "http://localhost:8080"}),
            patch("dispatch_mcp.server._client") as mock_client,
        ):
            mock_response = MagicMock()
            # Response body larger than MAX_RESPONSE_LENGTH (1 MiB)
            mock_response.content = b"x" * (1024 * 1024 + 1)
            mock_response.raise_for_status = MagicMock()
            mock_client.post.return_value = mock_response

            from dispatch_mcp.server import dispatch_custom

            with pytest.raises(RuntimeError, match="exceeds maximum allowed size"):
                dispatch_custom("worker", "test")

    def test_dispatch_custom_connection_error(self) -> None:
        import httpx

        with (
            patch.dict("os.environ", {"OMNIROUTE_URL": "http://localhost:8080"}),
            patch("dispatch_mcp.server.httpx.Client") as mock_client_cls,
        ):
            mock_client = MagicMock()
            mock_client.__enter__ = MagicMock(return_value=mock_client)
            mock_client.__exit__ = MagicMock(return_value=False)
            mock_client.post.side_effect = httpx.ConnectError("Connection refused")
            mock_client_cls.return_value = mock_client

            from dispatch_mcp.server import dispatch_custom

            with pytest.raises(httpx.ConnectError):
                dispatch_custom("main", "test")

    def test_dispatch_custom_timeout(self) -> None:
        import httpx

        with (
            patch.dict("os.environ", {"OMNIROUTE_URL": "http://localhost:8080"}),
            patch("dispatch_mcp.server.httpx.Client") as mock_client_cls,
        ):
            mock_client = MagicMock()
            mock_client.__enter__ = MagicMock(return_value=mock_client)
            mock_client.__exit__ = MagicMock(return_value=False)
            mock_client.post.side_effect = httpx.TimeoutException("timed out")
            mock_client_cls.return_value = mock_client

            from dispatch_mcp.server import dispatch_custom

            with pytest.raises(httpx.TimeoutException):
                dispatch_custom("worker", "test")

    def test_dispatch_custom_http_error(self) -> None:
        import httpx

        with (
            patch.dict("os.environ", {"OMNIROUTE_URL": "http://localhost:8080"}),
            patch("dispatch_mcp.server.httpx.Client") as mock_client_cls,
        ):
            mock_response = MagicMock()
            mock_response.raise_for_status.side_effect = httpx.HTTPStatusError(
                "404 Not Found",
                request=MagicMock(),
                response=MagicMock(status_code=404),
            )
            mock_client = MagicMock()
            mock_client.__enter__ = MagicMock(return_value=mock_client)
            mock_client.__exit__ = MagicMock(return_value=False)
            mock_client.post.return_value = mock_response
            mock_client_cls.return_value = mock_client

            from dispatch_mcp.server import dispatch_custom

            with pytest.raises(httpx.HTTPStatusError):
                dispatch_custom("worker", "test")

    def test_dispatch_custom_json_decode_error(self) -> None:
        import json

        with (
            patch.dict("os.environ", {"OMNIROUTE_URL": "http://localhost:8080"}),
            patch("dispatch_mcp.server.httpx.Client") as mock_client_cls,
        ):
            mock_response = MagicMock()
            mock_response.json.side_effect = json.JSONDecodeError("invalid", "", 0)
            mock_response.raise_for_status = MagicMock()
            mock_client = MagicMock()
            mock_client.__enter__ = MagicMock(return_value=mock_client)
            mock_client.__exit__ = MagicMock(return_value=False)
            mock_client.post.return_value = mock_response
            mock_client_cls.return_value = mock_client

            from dispatch_mcp.server import dispatch_custom

            with pytest.raises(json.JSONDecodeError):
                dispatch_custom("worker", "test")

    def test_dispatch_health_success(self) -> None:
        with (
            patch.dict("os.environ", {"OMNIROUTE_URL": "http://localhost:8080"}),
            patch("dispatch_mcp.server.httpx.Client") as mock_client_cls,
        ):
            mock_response = MagicMock()
            mock_response.json.return_value = {"status": "ok"}
            mock_response.raise_for_status = MagicMock()
            mock_client = MagicMock()
            mock_client.__enter__ = MagicMock(return_value=mock_client)
            mock_client.__exit__ = MagicMock(return_value=False)
            mock_client.post.return_value = mock_response
            mock_client_cls.return_value = mock_client

            from dispatch_mcp.server import dispatch_health

            result = dispatch_health()
            mock_client.post.assert_called_once()
            call_args = mock_client.post.call_args
            assert "health" in call_args[0][0]
            assert call_args[1]["json"] == {}
            assert result == {"status": "ok"}

    def test_missing_omniroute_url_raises(self) -> None:
        with patch.dict("os.environ", {}, clear=True):
            from dispatch_mcp.server import dispatch_custom

            with pytest.raises(ValueError, match="OMNIROUTE_URL"):
                dispatch_custom("worker", "test")


class TestDispatchCustomTierValidation:
    """Tests for dispatch_custom tier validation."""

    def test_invalid_tier_raises(self) -> None:
        from dispatch_mcp.server import dispatch_custom

        with pytest.raises(ValueError, match="Invalid tier 'rogue'"):
            dispatch_custom("rogue", "test")

    def test_empty_tier_raises(self) -> None:
        from dispatch_mcp.server import dispatch_custom

        with pytest.raises(ValueError, match="Invalid tier ''"):
            dispatch_custom("", "test")


class TestTierTools:
    """Tests that tier dispatch tools are registered and callable."""

    def test_all_tier_tools_importable(self) -> None:
        from dispatch_mcp.server import (
            dispatch_codeman,
            dispatch_freetier,
            dispatch_gemini,
            dispatch_haiku,
            dispatch_kimi,
            dispatch_kimi_thinking,
            dispatch_main,
            dispatch_minimax,
            dispatch_opus,
            dispatch_worker,
        )

        tools = [
            dispatch_worker,
            dispatch_main,
            dispatch_codeman,
            dispatch_freetier,
            dispatch_kimi,
            dispatch_kimi_thinking,
            dispatch_minimax,
            dispatch_opus,
            dispatch_haiku,
            dispatch_gemini,
        ]
        for tool in tools:
            assert callable(tool)

    def test_all_tier_tools_have_unique_references(self) -> None:
        """Regression: ensure no silent tool-name collisions."""
        from dispatch_mcp.server import (
            dispatch_codeman,
            dispatch_freetier,
            dispatch_gemini,
            dispatch_haiku,
            dispatch_kimi,
            dispatch_kimi_thinking,
            dispatch_main,
            dispatch_minimax,
            dispatch_opus,
            dispatch_worker,
        )

        tools = [
            dispatch_worker,
            dispatch_main,
            dispatch_codeman,
            dispatch_freetier,
            dispatch_kimi,
            dispatch_kimi_thinking,
            dispatch_minimax,
            dispatch_opus,
            dispatch_haiku,
            dispatch_gemini,
        ]
        seen: dict[int, str] = {}
        for tool in tools:
            id_ = id(tool)
            assert id_ not in seen, f"Duplicate tool reference for {seen[id_]}"
            seen[id_] = getattr(tool, "__name__", str(tool))
