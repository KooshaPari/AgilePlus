"""AgilePlus MCP server — FastMCP entry point.

Registers all MCP tools, implements the Roots and Elicitation primitives,
and connects to the Rust gRPC backend.

Traceability: FR-010, FR-049 / WP14-T082, T084b, T084c, T084d

Usage::

    uv run python -m agileplus_mcp
    # or after installation:
    agileplus-mcp
"""

from __future__ import annotations

import logging
import os
import socket
import subprocess
from ipaddress import ip_address
from pathlib import Path
from typing import Any
from urllib.parse import unquote, urlparse

from fastmcp import Context, FastMCP
from fastmcp.server.middleware import Middleware
from mcp.types import Root

from agileplus_mcp.grpc_client import (
    AgilePlusCoreClient,
    GrpcConnectionError,
    bind_session_project_root,
)
from agileplus_mcp.sampling import SamplingHandler
from agileplus_mcp.tools import features as features_module
from agileplus_mcp.tools import governance as governance_module
from agileplus_mcp.tools import queue as queue_module
from agileplus_mcp.tools import status as status_module
from agileplus_mcp.validation import validate_slug, validate_text

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Global state: one client and one FastMCP app shared across the process.
# ---------------------------------------------------------------------------

GRPC_ADDRESS = os.environ.get("AGILEPLUS_GRPC_ADDRESS", "localhost:50051")

mcp: FastMCP = FastMCP("AgilePlus")
_client: AgilePlusCoreClient | None = None
_sampling: SamplingHandler | None = None
_registered_app: FastMCP | None = None
_runtime_tool_names: set[str] = set()
_session_project_root: Path | None = None


def _get_client() -> AgilePlusCoreClient:
    if _client is None:
        raise RuntimeError("gRPC client not initialised — call startup() first")
    return _client


async def resolve_client_project_root(roots: list[Root]) -> Path:
    """Validate one absolute client-advertised Git worktree URI.

    Uses Git discovery so linked worktrees (``.git`` is a file) and main
    worktrees (``.git`` is a directory) are both accepted, while any
    directory containing a fake ``.git`` is rejected.
    """
    if len(roots) != 1:
        raise ValueError("exactly one file root is required for an AgilePlus session")
    parsed = urlparse(str(roots[0].uri))
    if parsed.scheme != "file" or parsed.netloc:
        raise ValueError("the project root must be an absolute file URI")
    path = Path(unquote(parsed.path))
    if not path.is_absolute():
        raise ValueError("the project root must be an absolute file URI")
    try:
        root = path.resolve(strict=True)
    except FileNotFoundError as exc:
        raise ValueError("the project root does not exist") from exc
    if not root.is_dir():
        raise ValueError("the project root must be a Git worktree")
    try:
        completed = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "--show-toplevel", "--git-common-dir"],
            check=False,
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise ValueError("the project root must be a Git worktree") from exc
    if completed.returncode != 0:
        raise ValueError("the project root must be a Git worktree")
    lines = [line.strip() for line in completed.stdout.splitlines() if line.strip()]
    if len(lines) != 2:
        raise ValueError("the project root must be a Git worktree")
    toplevel = Path(lines[0]).resolve()
    if toplevel != root:
        raise ValueError(
            f"the advertised project root {root} resolves to git worktree {toplevel}"
        )
    return root


async def _ensure_session_project_root(fastmcp_context: Any) -> str:
    """Return the session-bound project root, resolving from Roots if missing.

    Resource requests and tool requests both go through this helper so
    that ``on_read_resource`` gets the same binding as ``on_call_tool``.
    """
    project_root = await fastmcp_context.get_state("agileplus.project_root")
    if project_root is None:
        root = await resolve_client_project_root(await fastmcp_context.list_roots())
        project_root = str(root)
        await fastmcp_context.set_state("agileplus.project_root", project_root)
    return project_root


class ProjectRootMiddleware(Middleware):
    """Bind each tool/resource request to one client-approved repository root."""

    async def on_call_tool(self, context: Any, call_next: Any) -> Any:
        fastmcp_context = context.fastmcp_context
        if fastmcp_context is None:
            raise RuntimeError("MCP session context is unavailable")
        project_root = await _ensure_session_project_root(fastmcp_context)
        with bind_session_project_root(project_root):
            return await call_next(context)

    async def on_read_resource(self, context: Any, call_next: Any) -> Any:
        fastmcp_context = context.fastmcp_context
        if fastmcp_context is None:
            raise RuntimeError("MCP session context is unavailable")
        project_root = await _ensure_session_project_root(fastmcp_context)
        with bind_session_project_root(project_root):
            return await call_next(context)


mcp.add_middleware(ProjectRootMiddleware())


# ---------------------------------------------------------------------------
# T084c: MCP Roots primitive — declare workspace boundaries.
# ---------------------------------------------------------------------------


@mcp.resource("roots://workspace")
async def get_workspace_roots(ctx: Context | None = None) -> dict[str, Any]:
    """Declare workspace roots for the MCP client.

    Returns a list of filesystem roots the server works within, allowing
    the MCP client to scope file operations correctly.

    Roots update dynamically as features are created.

    Note: the ``on_read_resource`` middleware is responsible for binding
    the session project root before this resource handler runs, so the
    ``list_features`` call below already carries a valid ``ProjectScope``.
    The local ``_ensure_session_project_root`` call here covers callers
    that bypass middleware (e.g. direct in-process unit tests).
    """
    if ctx is not None:
        project_root = await _ensure_session_project_root(ctx)
    elif _session_project_root is None:
        raise RuntimeError("project root is not bound to this MCP session")
    else:
        project_root = str(_session_project_root)
    with bind_session_project_root(project_root):
        client = _get_client()
        features = await client.list_features()

    root = Path(project_root).resolve()
    roots = [
        {"uri": root.as_uri(), "name": "project-root"},
        {"uri": (root / ".agileplus").as_uri(), "name": "agileplus-data"},
    ]

    for feature in features:
        slug = feature["slug"]
        try:
            validate_slug(slug, "feature slug")
        except ValueError:
            logger.warning("Skipping feature with invalid slug: %r", slug)
            continue
        roots.append(
            {
                "uri": (root / "docs" / "agileplus" / slug).as_uri(),
                "name": f"feature-spec-{slug}",
            }
        )

    return {"roots": roots}


# ---------------------------------------------------------------------------
# T084d: MCP Elicitation primitive — structured discovery interviews.
# ---------------------------------------------------------------------------


@mcp.tool(name="agileplus_elicit_feature")
async def elicit_feature(
    feature_name: str,
    target_branch: str = "main",
) -> dict[str, Any]:
    """Begin an elicitation interview to specify a new feature.

    Sends structured questions to the MCP client and gathers answers to
    build a complete feature specification.

    Args:
        feature_name: Human-readable feature name (used to derive the slug).
        target_branch: Target branch for the eventual merge.

    Returns:
        dict with ``questions`` for the caller to answer, plus a ``session_id``
        to pass back with answers.
    """
    import hashlib
    import time

    validate_text(feature_name, "feature_name", max_length=256)
    validate_text(target_branch, "target_branch", max_length=256)

    session_id = hashlib.sha256(f"{feature_name}{time.time()}".encode()).hexdigest()[:8]

    return {
        "session_id": session_id,
        "feature_name": feature_name,
        "target_branch": target_branch,
        "questions": [
            {
                "id": "problem_statement",
                "question": "What problem does this feature solve?",
                "type": "text",
                "required": True,
            },
            {
                "id": "acceptance_criteria",
                "question": "What are the acceptance criteria? (one per line)",
                "type": "multiline",
                "required": True,
            },
            {
                "id": "scope",
                "question": "Which files or modules are in scope? (comma-separated paths)",
                "type": "text",
                "required": False,
            },
            {
                "id": "out_of_scope",
                "question": "What is explicitly out of scope?",
                "type": "text",
                "required": False,
            },
            {
                "id": "risks",
                "question": "What are the main risks or open questions?",
                "type": "text",
                "required": False,
            },
        ],
    }


@mcp.tool(name="agileplus_elicit_clarify")
async def elicit_clarify(feature_slug: str) -> dict[str, Any]:
    """Generate clarifying questions for an existing feature spec.

    Analyses the current spec and returns targeted questions to resolve
    ambiguities before planning.

    Args:
        feature_slug: Kebab-case feature identifier.

    Returns:
        dict with ``questions`` and current ``feature`` snapshot.
    """
    validate_slug(feature_slug, "feature_slug")
    client = _get_client()
    feature = await client.get_feature(feature_slug)
    state = await client.get_feature_state(feature_slug)

    return {
        "feature": feature,
        "current_state": state["state"],
        "questions": [
            {
                "id": "blockers",
                "question": (
                    f"Are there any blockers preventing moving from {state['state']}"
                    f" to {state.get('next_command', 'next state')}?"
                ),
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
        ],
    }


# ---------------------------------------------------------------------------
# T084b: Sampling tool — server-initiated analysis
# ---------------------------------------------------------------------------


@mcp.tool(name="agileplus_sample_triage")
async def sample_triage(feature_slug: str, agent_output: str) -> dict[str, Any]:
    """Server-initiated triage of agent output.

    Classifies errors/warnings in agent output and suggests remediation.

    Args:
        feature_slug: Feature the agent was working on.
        agent_output: Raw output from the agent run.

    Returns:
        Triage result with ``severity``, ``category``, and ``remediation``.
    """
    validate_slug(feature_slug, "feature_slug")
    validate_text(agent_output, "agent_output", max_length=1_000_000)
    sampling = _sampling
    if sampling is None:
        raise RuntimeError("Sampling handler not initialised")
    return await sampling.auto_triage(feature_slug, agent_output)


@mcp.tool(name="agileplus_sample_governance_check")
async def sample_governance_check(feature_slug: str, planned_transition: str) -> dict[str, Any]:
    """Server-initiated governance pre-check before a state transition.

    Args:
        feature_slug: Feature about to transition.
        planned_transition: Transition string (e.g. implementing->validated).

    Returns:
        dict with ``ready`` bool and ``blockers`` list.
    """
    validate_slug(feature_slug, "feature_slug")
    validate_text(planned_transition, "planned_transition", max_length=256)
    sampling = _sampling
    if sampling is None:
        raise RuntimeError("Sampling handler not initialised")
    return await sampling.governance_pre_check(feature_slug, planned_transition)


@mcp.tool(name="agileplus_sample_retrospective")
async def sample_retrospective(feature_slug: str) -> dict[str, Any]:
    """Server-initiated retrospective analysis of a shipped feature.

    Args:
        feature_slug: Shipped feature to analyse.

    Returns:
        Retrospective summary with highlights, issues, and metrics.
    """
    validate_slug(feature_slug, "feature_slug")
    sampling = _sampling
    if sampling is None:
        raise RuntimeError("Sampling handler not initialised")
    return await sampling.generate_retrospective(feature_slug)


# ---------------------------------------------------------------------------
# Startup / shutdown lifecycle
# ---------------------------------------------------------------------------


def register_compatibility_tools(app: FastMCP, client: AgilePlusCoreClient) -> None:
    """Register the canonical Codex-facing AgilePlus tool names."""

    @app.tool(name="health_check")
    async def health_check() -> dict[str, str]:
        try:
            await client.list_features()
        except Exception as exc:
            return {
                "status": "unhealthy",
                "mcp_server": "ok",
                "grpc_core": "unreachable",
                "version": "0.1.0",
                "error": str(exc),
            }
        return {"status": "healthy", "mcp_server": "ok", "grpc_core": "ok", "version": "0.1.0"}

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


async def startup(grpc_address: str = GRPC_ADDRESS) -> None:
    """Initialise the gRPC client and register all tools."""
    global _client, _registered_app, _runtime_tool_names, _sampling

    if _client is not None:
        await _client.close()

    if _registered_app is mcp:
        for tool_name in _runtime_tool_names:
            mcp.local_provider.remove_tool(tool_name)
    else:
        _runtime_tool_names = set()

    existing_tool_names = {tool.name for tool in await mcp.list_tools()}

    client = AgilePlusCoreClient(grpc_address)
    try:
        await client.connect()
    except GrpcConnectionError as exc:
        logger.warning(
            "Could not connect to gRPC server at %s: %s — tools will fail until server is up",
            grpc_address,
            exc,
        )

    _client = client
    _sampling = SamplingHandler(client)

    features_module.register_tools(mcp, client)
    governance_module.register_tools(mcp, client)
    queue_module.register_tools(mcp, client)
    status_module.register_tools(mcp, client)
    register_compatibility_tools(mcp, client)
    _runtime_tool_names = {
        tool.name for tool in await mcp.list_tools() if tool.name not in existing_tool_names
    }
    _registered_app = mcp

    logger.info("AgilePlus MCP server ready (gRPC: %s)", grpc_address)


async def shutdown() -> None:
    """Close the gRPC connection."""
    global _client, _sampling
    if _client is not None:
        await _client.close()
        _client = None
    _sampling = None


def main() -> None:
    """Entry point for `agileplus-mcp` command."""
    import asyncio

    logging.basicConfig(level=logging.INFO)

    async def _run() -> None:
        await startup()
        try:
            transport = os.environ.get("AGILEPLUS_MCP_TRANSPORT", "stdio")
            kwargs = _transport_kwargs(transport)
            await mcp.run_async(transport=transport, show_banner=False, **kwargs)
        finally:
            await shutdown()

    asyncio.run(_run())


def _transport_kwargs(transport: str) -> dict[str, Any]:
    if transport != "http":
        return {}
    host = os.environ.get("AGILEPLUS_MCP_HOST", "127.0.0.1")
    if host == "localhost":
        try:
            resolved_addresses = {
                ip_address(address[4][0])
                for address in socket.getaddrinfo(host, None, type=socket.SOCK_STREAM)
            }
        except (OSError, ValueError) as exc:
            raise ValueError("plaintext AgilePlus MCP host must resolve to loopback") from exc
        is_loopback = bool(resolved_addresses) and all(
            address.is_loopback for address in resolved_addresses
        )
    else:
        try:
            is_loopback = ip_address(host).is_loopback
        except ValueError:
            is_loopback = False
    if not is_loopback:
        raise ValueError("plaintext AgilePlus MCP must bind to a loopback address")
    return {
        "host": host,
        "port": int(os.environ.get("AGILEPLUS_MCP_PORT", "8765")),
        "path": os.environ.get("AGILEPLUS_MCP_PATH", "/mcp"),
    }


if __name__ == "__main__":
    main()
