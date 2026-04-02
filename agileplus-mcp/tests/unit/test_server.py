"""Smoke tests for MCP server initialization."""

from __future__ import annotations

import pytest
from fastmcp.server.auth.providers.workos import AuthKitProvider, WorkOSProvider


def test_server_creates() -> None:
    """Verify the FastMCP server can be instantiated with the expected name."""
    from agileplus_mcp.server import mcp

    assert mcp.name == "agileplus"


@pytest.mark.asyncio
async def test_tools_registered() -> None:
    """Verify all required MCP tools are registered with the server.

    This test acts as a contract: if someone removes a tool, this test fails.
    """
    from agileplus_mcp.server import mcp

    tool_names = [t.name for t in await mcp.list_tools()]
    # Feature tools
    assert "get_feature" in tool_names
    assert "list_features" in tool_names
    assert "get_work_packages" in tool_names
    # Governance tools
    assert "check_governance" in tool_names
    assert "get_audit_trail" in tool_names
    assert "verify_audit_chain" in tool_names
    # Status tools
    assert "get_dashboard" in tool_names
    assert "health_check" in tool_names


def test_grpc_client_target() -> None:
    """Verify gRPC client can be instantiated with the correct default target."""
    from agileplus_mcp.grpc_client import AgilePlusCoreClient

    client = AgilePlusCoreClient()
    assert client.target == "localhost:50051"


def test_grpc_client_custom_target() -> None:
    """Verify gRPC client accepts custom host and port."""
    from agileplus_mcp.grpc_client import AgilePlusCoreClient

    client = AgilePlusCoreClient(host="grpc-core", port=9090)
    assert client.target == "grpc-core:9090"


@pytest.mark.asyncio
async def test_grpc_client_stubs_raise_not_implemented() -> None:
    """Verify stub methods raise NotImplementedError as expected."""
    from agileplus_mcp.grpc_client import AgilePlusCoreClient

    client = AgilePlusCoreClient()
    with pytest.raises(NotImplementedError):
        await client.get_feature("test-slug")


def test_auth_disabled_without_provider_env() -> None:
    """Verify auth stays off when no provider env is configured."""
    from agileplus_mcp.server import ServerSettings, build_auth_provider

    settings = ServerSettings()
    assert build_auth_provider(settings) is None


def test_authkit_provider_selected_from_settings() -> None:
    """Verify AuthKit is the default provider when auth env is present."""
    from agileplus_mcp.server import ServerSettings, build_auth_provider

    settings = ServerSettings(
        authkit_domain="https://example.authkit.app",
        base_url="http://127.0.0.1:8000",
        client_id="client_123",
    )

    provider = build_auth_provider(settings)
    assert isinstance(provider, AuthKitProvider)


def test_workos_provider_selected_when_requested() -> None:
    """Verify the proxy provider is used when explicitly requested."""
    from agileplus_mcp.server import ServerSettings, build_auth_provider

    settings = ServerSettings(
        authkit_domain="https://example.authkit.app",
        base_url="http://127.0.0.1:8000",
        auth_mode="workos",
        client_id="client_123",
        client_secret="secret_123",  # noqa: S106
    )

    provider = build_auth_provider(settings)
    assert isinstance(provider, WorkOSProvider)


def test_workos_provider_requires_credentials() -> None:
    """Verify WorkOS mode fails closed without client credentials."""
    from agileplus_mcp.server import ServerSettings, build_auth_provider

    settings = ServerSettings(
        authkit_domain="https://example.authkit.app",
        base_url="http://127.0.0.1:8000",
        auth_mode="workos",
    )

    with pytest.raises(ValueError, match="WORKOS_CLIENT_ID"):
        build_auth_provider(settings)


def test_http_app_exposes_oauth_well_known_routes() -> None:
    """Verify the auth-enabled HTTP app exposes the discovery routes."""
    from agileplus_mcp.server import ServerSettings, create_http_app

    settings = ServerSettings(
        authkit_domain="https://example.authkit.app",
        base_url="http://127.0.0.1:8000",
        client_id="client_123",
    )
    app = create_http_app(settings)
    route_paths = {route.path for route in app.routes}

    assert "/mcp" in route_paths
    assert "/.well-known/oauth-authorization-server" in route_paths
    assert "/.well-known/oauth-protected-resource/mcp" in route_paths
