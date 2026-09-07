"""In-process unit tests for the FastMCP server entry point."""

from __future__ import annotations

import re
import subprocess
from unittest.mock import AsyncMock, MagicMock

import pytest
from mcp.types import Root

from agileplus_mcp import server
from agileplus_mcp.grpc_client import AgilePlusCoreClient
from agileplus_mcp.validation import InputValidationError


def _make_git_repo(path) -> None:
    """Initialise a real (non-bare) git worktree at ``path``.

    The middleware uses ``git rev-parse --show-toplevel`` to validate
    project roots, so tests must construct real repositories rather than
    fake ``.git`` directories.
    """
    path.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["git", "init", "--quiet", str(path)],
        check=True,
        capture_output=True,
    )


@pytest.fixture(autouse=True)
def reset_server_state(monkeypatch: pytest.MonkeyPatch) -> None:
    """Keep module globals isolated without connecting to a real service."""
    monkeypatch.setattr(server, "_client", None)
    monkeypatch.setattr(server, "_sampling", None)


@pytest.mark.asyncio
async def test_workspace_roots_include_valid_feature_scopes_and_skip_invalid_slugs(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path,
) -> None:
    repo = tmp_path / "project"
    _make_git_repo(repo)
    client = MagicMock()
    client.list_features = AsyncMock(
        return_value=[{"slug": "valid-feature"}, {"slug": "Invalid_Feature"}]
    )
    monkeypatch.setattr(server, "_client", client)
    monkeypatch.setattr(server, "_session_project_root", repo)

    result = await server.get_workspace_roots()

    assert result == {
        "roots": [
            {"uri": repo.as_uri(), "name": "project-root"},
            {"uri": (repo / ".agileplus").as_uri(), "name": "agileplus-data"},
            {
                "uri": (repo / "docs" / "agileplus" / "valid-feature").as_uri(),
                "name": "feature-spec-valid-feature",
            },
        ]
    }
    client.list_features.assert_awaited_once_with()


@pytest.mark.asyncio
async def test_workspace_roots_use_the_session_project_root(
    monkeypatch: pytest.MonkeyPatch, tmp_path
) -> None:
    repo = tmp_path / "project"
    _make_git_repo(repo)
    client = MagicMock()
    client.list_features = AsyncMock(return_value=[])
    context = MagicMock()
    context.get_state = AsyncMock(return_value=str(repo))
    monkeypatch.setattr(server, "_client", client)

    result = await server.get_workspace_roots(context)

    assert result["roots"][:2] == [
        {"uri": repo.as_uri(), "name": "project-root"},
        {"uri": (repo / ".agileplus").as_uri(), "name": "agileplus-data"},
    ]
    context.get_state.assert_awaited_once_with("agileplus.project_root")


@pytest.mark.asyncio
async def test_client_root_scope_requires_exactly_one_git_worktree(tmp_path) -> None:
    repo = tmp_path / "project"
    _make_git_repo(repo)

    assert await server.resolve_client_project_root(
        [Root(uri=repo.as_uri(), name="project")]
    ) == repo.resolve()

    with pytest.raises(ValueError, match="exactly one file root"):
        await server.resolve_client_project_root([])
    with pytest.raises(ValueError, match="exactly one file root"):
        await server.resolve_client_project_root(
            [Root(uri=repo.as_uri()), Root(uri=repo.as_uri())]
        )

    non_repo = tmp_path / "not-a-repo"
    non_repo.mkdir()
    with pytest.raises(ValueError, match="Git worktree"):
        await server.resolve_client_project_root([Root(uri=non_repo.as_uri())])


@pytest.mark.asyncio
async def test_middleware_binds_validated_client_root_to_the_current_request(tmp_path) -> None:
    repo = tmp_path / "project"
    _make_git_repo(repo)
    fastmcp_context = MagicMock()
    fastmcp_context.get_state = AsyncMock(return_value=None)
    fastmcp_context.list_roots = AsyncMock(return_value=[Root(uri=repo.as_uri())])
    fastmcp_context.set_state = AsyncMock()
    context = MagicMock(fastmcp_context=fastmcp_context)

    async def call_next(_context):
        return AgilePlusCoreClient()._project_scope().canonical_repo_root

    result = await server.ProjectRootMiddleware().on_call_tool(context, call_next)

    assert result == str(repo.resolve())
    fastmcp_context.set_state.assert_awaited_once_with(
        "agileplus.project_root", str(repo.resolve())
    )


@pytest.mark.asyncio
async def test_elicitation_validates_input_and_returns_structured_questions() -> None:
    result = await server.elicit_feature("Server coverage", target_branch="release")

    assert re.fullmatch(r"[0-9a-f]{8}", result["session_id"])
    assert result["feature_name"] == "Server coverage"
    assert result["target_branch"] == "release"
    assert [question["id"] for question in result["questions"]] == [
        "problem_statement",
        "acceptance_criteria",
        "scope",
        "out_of_scope",
        "risks",
    ]
    assert result["questions"][0] == {
        "id": "problem_statement",
        "question": "What problem does this feature solve?",
        "type": "text",
        "required": True,
    }

    with pytest.raises(InputValidationError, match="feature_name exceeds maximum length"):
        await server.elicit_feature("x" * 257)
    with pytest.raises(InputValidationError, match="target_branch exceeds maximum length"):
        await server.elicit_feature("Feature", target_branch="x" * 257)


@pytest.mark.asyncio
async def test_sampling_tools_require_initialised_handler() -> None:
    with pytest.raises(RuntimeError, match="Sampling handler not initialised"):
        await server.sample_triage("valid-feature", "error: failed")
    with pytest.raises(RuntimeError, match="Sampling handler not initialised"):
        await server.sample_governance_check("valid-feature", "planned->implementing")
    with pytest.raises(RuntimeError, match="Sampling handler not initialised"):
        await server.sample_retrospective("valid-feature")


@pytest.mark.asyncio
async def test_sampling_tools_delegate_to_initialised_handler(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sampling = MagicMock()
    sampling.auto_triage = AsyncMock(return_value={"severity": "error"})
    sampling.governance_pre_check = AsyncMock(return_value={"ready": True})
    sampling.generate_retrospective = AsyncMock(return_value={"audit_valid": True})
    monkeypatch.setattr(server, "_sampling", sampling)

    assert await server.sample_triage("valid-feature", "error: failed") == {"severity": "error"}
    assert await server.sample_governance_check("valid-feature", "planned->implementing") == {
        "ready": True
    }
    assert await server.sample_retrospective("valid-feature") == {"audit_valid": True}
    sampling.auto_triage.assert_awaited_once_with("valid-feature", "error: failed")
    sampling.governance_pre_check.assert_awaited_once_with("valid-feature", "planned->implementing")
    sampling.generate_retrospective.assert_awaited_once_with("valid-feature")


@pytest.mark.asyncio
async def test_startup_registers_tools_after_grpc_connection_failure(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client = MagicMock()
    client.connect = AsyncMock(side_effect=server.GrpcConnectionError("offline"))
    client_factory = MagicMock(return_value=client)
    sampling = MagicMock()
    sampling_factory = MagicMock(return_value=sampling)
    feature_registration = MagicMock()
    governance_registration = MagicMock()
    status_registration = MagicMock()
    monkeypatch.setattr(server, "AgilePlusCoreClient", client_factory)
    monkeypatch.setattr(server, "SamplingHandler", sampling_factory)
    monkeypatch.setattr(server.features_module, "register_tools", feature_registration)
    monkeypatch.setattr(server.governance_module, "register_tools", governance_registration)
    monkeypatch.setattr(server.status_module, "register_tools", status_registration)

    await server.startup("grpc://unavailable")

    client_factory.assert_called_once_with("grpc://unavailable")
    client.connect.assert_awaited_once_with()
    assert server._client is client
    assert server._sampling is sampling
    sampling_factory.assert_called_once_with(client)
    feature_registration.assert_called_once_with(server.mcp, client)
    governance_registration.assert_called_once_with(server.mcp, client)
    status_registration.assert_called_once_with(server.mcp, client)


@pytest.mark.asyncio
async def test_startup_registers_tools_after_grpc_connection(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client = MagicMock()
    client.connect = AsyncMock()
    client_factory = MagicMock(return_value=client)
    sampling = MagicMock()
    sampling_factory = MagicMock(return_value=sampling)
    feature_registration = MagicMock()
    governance_registration = MagicMock()
    status_registration = MagicMock()
    monkeypatch.setattr(server, "AgilePlusCoreClient", client_factory)
    monkeypatch.setattr(server, "SamplingHandler", sampling_factory)
    monkeypatch.setattr(server.features_module, "register_tools", feature_registration)
    monkeypatch.setattr(server.governance_module, "register_tools", governance_registration)
    monkeypatch.setattr(server.status_module, "register_tools", status_registration)

    await server.startup("grpc://available")

    client.connect.assert_awaited_once_with()
    assert server._client is client
    assert server._sampling is sampling
    feature_registration.assert_called_once_with(server.mcp, client)
    governance_registration.assert_called_once_with(server.mcp, client)
    status_registration.assert_called_once_with(server.mcp, client)
