from __future__ import annotations

import logging
import os
import signal
from collections.abc import Callable
from typing import Any

import httpx
from mcp.server.fastmcp import FastMCP

mcp = FastMCP("dispatch-mcp")
logger = logging.getLogger("dispatch_mcp")

MAX_MESSAGE_LENGTH = 4096  # bytes — prevents unbounded payload to OmniRoute

# Allowlist of valid dispatch tiers — dispatch_custom must use one of these.
VALID_TIERS = frozenset(
    {
        "worker",
        "main",
        "codeman",
        "freetier",
        "kimi",
        "kimi_thinking",
        "minimax",
        "opus",
        "haiku",
        "gemini",
    }
)


def _call_omniroute(route: str, payload: dict[str, Any]) -> dict[str, Any]:
    base = os.environ.get("OMNIROUTE_URL")
    if not base:
        raise ValueError(
            "OMNIROUTE_URL environment variable is not set. "
            "Set it to the base URL of the dispatch backend before starting the server."
        )
    with httpx.Client(timeout=10, follow_redirects=False) as client:
        try:
            response = client.post(
                f"{base.rstrip('/')}/{route.lstrip('/')}", json=payload
            )
            response.raise_for_status()
            return response.json()  # type: ignore[no-any-return]
        except httpx.TimeoutException as e:
            logger.error("OmniRoute timeout for route %s: %s", route, e)
            raise
        except httpx.HTTPStatusError as e:
            logger.error(
                "OmniRoute HTTP error %s for route %s: %s",
                e.response.status_code,
                route,
                e,
            )
            raise
        except httpx.RequestError as e:
            logger.error("OmniRoute request error for route %s: %s", route, e)
            raise


def _make_dispatch(tier: str) -> Callable[[], Callable[..., Any]]:
    @mcp.tool(name=f"dispatch_{tier}")
    def dispatch(message: str) -> dict[str, Any]:
        if len(message.encode()) > MAX_MESSAGE_LENGTH:
            raise ValueError(
                f"message exceeds maximum length of {MAX_MESSAGE_LENGTH} bytes"
            )
        return _call_omniroute("dispatch", {"tier": tier, "message": message})

    return dispatch


dispatch_worker = _make_dispatch("worker")
dispatch_main = _make_dispatch("main")
dispatch_codeman = _make_dispatch("codeman")
dispatch_freetier = _make_dispatch("freetier")
dispatch_kimi = _make_dispatch("kimi")
dispatch_kimi_thinking = _make_dispatch("kimi_thinking")
dispatch_minimax = _make_dispatch("minimax")
dispatch_opus = _make_dispatch("opus")
dispatch_haiku = _make_dispatch("haiku")
dispatch_gemini = _make_dispatch("gemini")


@mcp.tool()
def dispatch_custom(tier: str, message: str) -> dict[str, Any]:
    if tier not in VALID_TIERS:
        raise ValueError(
            f"Invalid tier '{tier}'. Must be one of: {', '.join(sorted(VALID_TIERS))}"
        )
    if len(message.encode()) > MAX_MESSAGE_LENGTH:
        raise ValueError(
            f"message exceeds maximum length of {MAX_MESSAGE_LENGTH} bytes"
        )
    return _call_omniroute("dispatch", {"tier": tier, "message": message})


@mcp.tool()
def dispatch_health() -> dict[str, Any]:
    """Check OmniRoute backend health. Requires OMNIROUTE_URL."""
    return _call_omniroute("health", {})


@mcp.tool()
def dispatch_liveness() -> dict[str, Any]:
    """Return server liveness status. Does not require OmniRoute."""
    return {"status": "alive", "server": "dispatch-mcp"}


def main() -> None:
    shutdown_requested = False

    def _handle_signal(signum: int, frame: object) -> None:
        nonlocal shutdown_requested
        sig_name = signal.Signals(signum).name
        logger.warning("Received %s, initiating graceful shutdown", sig_name)
        shutdown_requested = True

    signal.signal(signal.SIGTERM, _handle_signal)
    signal.signal(signal.SIGINT, _handle_signal)
    mcp.run()


if __name__ == "__main__":
    main()
