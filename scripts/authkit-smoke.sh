#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
PORTS_FILE="$PROJ_DIR/.agileplus/runtime/local-ports.env"
AUTH_FILE="$PROJ_DIR/.agileplus/authkit.env"

if [[ -f "$AUTH_FILE" ]]; then
  source "$AUTH_FILE"
fi

if [[ -f "$PORTS_FILE" ]]; then
  source "$PORTS_FILE"
fi

: "${AUTHKIT_DOMAIN:?Set AUTHKIT_DOMAIN or create .agileplus/authkit.env first.}"

AUTHKIT_DOMAIN="${AUTHKIT_DOMAIN%/}"
MCP_BASE_URL="${AGILEPLUS_MCP_BASE_URL:-}"

check_url() {
  local name="$1"
  local url="$2"

  if curl -fsS --max-time 10 "$url" >/dev/null; then
    echo "PASS  $name  $url"
    return 0
  fi

  echo "FAIL  $name  $url" >&2
  return 1
}

check_url "authkit-oauth-metadata" \
  "${AUTHKIT_DOMAIN}/.well-known/oauth-authorization-server"
check_url "authkit-oidc-metadata" \
  "${AUTHKIT_DOMAIN}/.well-known/openid-configuration"

if [[ -n "$MCP_BASE_URL" ]]; then
  MCP_BASE_URL="${MCP_BASE_URL%/}"
  check_url "mcp-protected-resource" \
    "${MCP_BASE_URL}/.well-known/oauth-protected-resource"
  check_url "mcp-authorization-server" \
    "${MCP_BASE_URL}/.well-known/oauth-authorization-server"
else
  echo "SKIP  local-mcp-auth-endpoints  Set AGILEPLUS_MCP_BASE_URL to verify local MCP auth metadata."
fi
