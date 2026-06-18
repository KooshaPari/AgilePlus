from __future__ import annotations

import logging
import os
from typing import Any

import httpx
from mcp.server.fastmcp import FastMCP

mcp = FastMCP("dispatch-mcp")
logger = logging.getLogger("dispatch_mcp")

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
    with httpx.Client(timeout=10) as client:
        try:
            response = client.post(
                f"{base.rstrip('/')}/{route.lstrip('/')}", json=payload
            )
            response.raise_for_status()
            return response.json()
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


def _make_dispatch(tier: str):
    @mcp.tool(name=f"dispatch_{tier}")
    def dispatch(message: str) -> dict[str, Any]:
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
    return _call_omniroute("dispatch", {"tier": tier, "message": message})


@mcp.tool()
def dispatch_health() -> dict[str, Any]:
    return _call_omniroute("health", {})


def main() -> None:
    """Start the MCP server. Registers SIGTERM/SIGINT handlers that log intent;
    the event loop (mcp.run) controls its own lifecycle and does not
    guarantee immediate interruption on signal receipt."""
    def _handle_signal(signum: int, frame: object) -> None:
        sig_name = signal.Signals(signum).name
        logger.warning("Received %s, initiating graceful shutdown", sig_name)

    signal.signal(signal.SIGTERM, _handle_signal)
    signal.signal(signal.SIGINT, _handle_signal)
    mcp.run()


if __name__ == "__main__":
    main()
