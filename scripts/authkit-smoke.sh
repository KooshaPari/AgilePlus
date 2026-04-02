#!/usr/bin/env bash
set -euo pipefail

AUTHKIT_DOMAIN="${AUTHKIT_DOMAIN:-}"
MCP_BASE_URL="${AGILEPLUS_MCP_BASE_URL:-}"
MCP_PATH="${AGILEPLUS_MCP_PATH:-/mcp}"

if [[ -z "$AUTHKIT_DOMAIN" ]]; then
  echo "AUTHKIT_DOMAIN is required" >&2
  exit 1
fi

if [[ ! "$MCP_PATH" =~ ^/ ]]; then
  MCP_PATH="/$MCP_PATH"
fi

check_json_url() {
  local name="$1"
  local url="$2"
  local body

  body="$(curl -fsS --max-time 15 "$url")"
  printf '%s\n' "$body" | python3 -m json.tool >/dev/null
  echo "PASS  $name  $url"
}

check_json_url "authkit-authorization-server" \
  "${AUTHKIT_DOMAIN%/}/.well-known/oauth-authorization-server"

if [[ -n "$MCP_BASE_URL" ]]; then
  check_json_url "local-authorization-server" \
    "${MCP_BASE_URL%/}/.well-known/oauth-authorization-server"
  check_json_url "local-protected-resource" \
    "${MCP_BASE_URL%/}/.well-known/oauth-protected-resource${MCP_PATH}"
fi
