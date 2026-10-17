"""Canonical status-tool aliases for MCP consumers."""

from __future__ import annotations

from typing import Any

from fastmcp import FastMCP

from agileplus_mcp.grpc_client import AgilePlusCoreClient


def register_compatibility_tools(app: FastMCP, client: AgilePlusCoreClient) -> None:
    """Register the canonical Codex-facing AgilePlus tool names."""

    @app.tool(name="health_check")
    async def health_check() -> dict[str, Any]:
        try:
            await client.list_features()
        except Exception as exc:
            return {
                "status": "unhealthy",
                "live": True,
                "ready": False,
                "mcp_server": "ok",
                "grpc_core": "unreachable",
                "version": "0.1.0",
                "error": str(exc),
            }
        return {
            "status": "healthy",
            "mcp_server": "ok",
            "grpc_core": "ok",
            "version": "0.1.0",
            "live": True,
            "ready": True,
        }

    @app.tool(name="list_features")
    async def list_features(state: str | None = None) -> list[dict[str, Any]]:
        return await client.list_features(state)

    @app.tool(name="get_feature")
    async def get_feature(slug: str) -> dict[str, Any]:
        return await client.get_feature(slug)

    @app.tool(name="get_work_packages")
    async def get_work_packages(feature_slug: str) -> list[dict[str, Any]]:
        return await client.list_work_packages(feature_slug)

    @app.tool(name="get_work_package")
    async def get_work_package(feature_slug: str, wp_id: str) -> dict[str, Any]:
        if not wp_id.startswith("WP") or not wp_id[2:].isdigit():
            raise ValueError("wp_id must use the canonical WP<positive integer> form")
        sequence = int(wp_id[2:])
        if sequence < 1:
            raise ValueError("wp_id sequence must be positive")
        return await client.get_work_package_status(feature_slug, sequence)

    @app.tool(name="get_tasks")
    async def get_tasks(feature_slug: str, wp_id: str | None = None) -> dict[str, str]:
        del feature_slug, wp_id
        return {"error": "not_implemented", "capability": "tasks"}

    @app.tool(name="get_metrics")
    async def get_metrics(feature_slug: str | None = None) -> dict[str, str]:
        del feature_slug
        return {"error": "not_implemented", "capability": "metrics"}

    @app.tool(name="get_governance_rules")
    async def get_governance_rules(feature_slug: str | None = None) -> dict[str, str]:
        del feature_slug
        return {"error": "not_implemented", "capability": "governance_rules"}

    @app.tool(name="check_governance")
    async def check_governance(feature_slug: str, transition: str | None = None) -> dict[str, Any]:
        """Check governance rules for one transition.

        When ``transition`` is omitted, the core evaluates only rules that
        apply globally (rules whose transition is empty). An explicit value
        is forwarded verbatim and must contain a ``source -> destination``
        transition, including planner forms such as ``WP01: Doing -> Review``.
        """
        if transition is not None:
            source, separator, destination = transition.partition("->")
            if (
                transition.count("->") != 1
                or not separator
                or not source.strip()
                or not destination.strip()
                or "\r" in transition
                or "\n" in transition
            ):
                raise ValueError("transition must contain source -> destination")
        return await client.check_governance_gate(feature_slug, transition or "")

    @app.tool(name="get_audit_trail")
    async def get_audit_trail(feature_slug: str, limit: int = 50) -> list[dict[str, Any]]:
        if limit < 0:
            raise ValueError("limit must be non-negative")
        return (await client.get_audit_trail(feature_slug))[:limit]

    @app.tool(name="verify_audit_chain")
    async def verify_audit_chain(feature_slug: str) -> dict[str, Any]:
        return await client.verify_audit_chain(feature_slug)

    @app.tool(name="get_dashboard")
    async def get_dashboard() -> dict[str, Any]:
        features = await client.list_features()
        counts: dict[str, int] = {}
        active_work_packages: list[dict[str, Any]] = []
        recent_audit_entries: list[dict[str, Any]] = []
        for feature in features:
            state = str(feature.get("state", "unknown"))
            counts[state] = counts.get(state, 0) + 1
            slug = str(feature.get("slug", ""))
            work_packages = await client.list_work_packages(slug)
            active_work_packages.extend(
                work_package
                for work_package in work_packages
                if str(work_package.get("state", "")).lower() in {"doing", "in_progress", "blocked"}
            )
            recent_audit_entries.extend(await client.get_audit_trail(slug))
        recent_audit_entries.sort(key=lambda entry: str(entry.get("timestamp", "")), reverse=True)
        return {
            "feature_counts": counts,
            "active_work_packages": active_work_packages,
            "recent_audit_entries": recent_audit_entries[:10],
            "health": "healthy",
        }
