"""FastMCP 3.0 server for AgilePlus."""

from __future__ import annotations

import os
from dataclasses import dataclass

from fastmcp import FastMCP
from fastmcp.server.auth.providers.workos import AuthKitProvider, WorkOSProvider
from fastmcp.server.http import StarletteWithLifespan

from agileplus_mcp.tools import features, governance, status

DEFAULT_MCP_HOST = "127.0.0.1"
DEFAULT_MCP_PATH = "/mcp"
DEFAULT_MCP_PORT = 8000
DEFAULT_SCOPES = ["openid", "profile", "email"]


def _parse_scopes(value: str | None) -> list[str] | None:
    if value is None:
        return None
    scopes = [scope.strip() for scope in value.split(",") if scope.strip()]
    return scopes or None


@dataclass(frozen=True)
class ServerSettings:
    """Environment-backed server configuration."""

    authkit_domain: str | None = None
    base_url: str | None = None
    auth_mode: str = "authkit"
    client_id: str | None = None
    client_secret: str | None = None
    host: str = DEFAULT_MCP_HOST
    port: int = DEFAULT_MCP_PORT
    mcp_path: str = DEFAULT_MCP_PATH
    required_scopes: list[str] | None = None

    @classmethod
    def from_env(cls) -> ServerSettings:
        mcp_path = os.getenv("AGILEPLUS_MCP_PATH", DEFAULT_MCP_PATH).strip() or DEFAULT_MCP_PATH
        if not mcp_path.startswith("/"):
            mcp_path = f"/{mcp_path}"

        return cls(
            authkit_domain=os.getenv("AUTHKIT_DOMAIN"),
            base_url=os.getenv("AGILEPLUS_MCP_BASE_URL"),
            auth_mode=os.getenv("AGILEPLUS_MCP_AUTH_MODE", "authkit").strip().lower(),
            client_id=os.getenv("WORKOS_CLIENT_ID"),
            client_secret=os.getenv("WORKOS_CLIENT_SECRET"),
            host=os.getenv("AGILEPLUS_MCP_HOST", DEFAULT_MCP_HOST),
            port=int(os.getenv("AGILEPLUS_MCP_PORT", str(DEFAULT_MCP_PORT))),
            mcp_path=mcp_path,
            required_scopes=_parse_scopes(os.getenv("AGILEPLUS_MCP_REQUIRED_SCOPES")),
        )

    @property
    def auth_enabled(self) -> bool:
        return bool(self.authkit_domain and self.base_url)

    @property
    def scopes(self) -> list[str]:
        return self.required_scopes or DEFAULT_SCOPES


def build_auth_provider(
    settings: ServerSettings,
) -> AuthKitProvider | WorkOSProvider | None:
    """Create the configured auth provider, if any."""
    if not settings.auth_enabled:
        return None

    if settings.auth_mode == "workos":
        if not settings.client_id or not settings.client_secret:
            raise ValueError(
                "WORKOS_CLIENT_ID and WORKOS_CLIENT_SECRET are required when "
                "AGILEPLUS_MCP_AUTH_MODE=workos"
            )
        return WorkOSProvider(
            client_id=settings.client_id,
            client_secret=settings.client_secret,
            authkit_domain=settings.authkit_domain,
            base_url=settings.base_url,
            required_scopes=settings.scopes,
        )

    if settings.auth_mode != "authkit":
        raise ValueError(
            "AGILEPLUS_MCP_AUTH_MODE must be either 'authkit' or 'workos'"
        )

    return AuthKitProvider(
        authkit_domain=settings.authkit_domain,
        base_url=settings.base_url,
        client_id=settings.client_id,
        required_scopes=settings.scopes,
    )


def create_mcp(settings: ServerSettings) -> FastMCP:
    """Build the FastMCP server with the configured auth surface."""
    mcp = FastMCP(
        name="agileplus",
        instructions="Spec-driven development engine with governance",
        auth=build_auth_provider(settings),
    )

    features.register(mcp)
    governance.register(mcp)
    status.register(mcp)
    return mcp


def create_http_app(settings: ServerSettings) -> StarletteWithLifespan:
    """Build an HTTP app that exposes MCP and OAuth metadata routes."""
    return create_mcp(settings).http_app(path=settings.mcp_path, transport="http")


settings = ServerSettings.from_env()
mcp = create_mcp(settings)
app = create_http_app(settings)


def main() -> None:
    """Start the MCP server."""
    if settings.auth_enabled:
        mcp.run(
            transport="http",
            host=settings.host,
            port=settings.port,
            path=settings.mcp_path,
        )
        return

    mcp.run()
