#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"

if [[ -z "${AUTHKIT_DOMAIN:-}" ]]; then
  echo "AUTHKIT_DOMAIN is required. Export it before running this script."
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required for metadata discovery."
  exit 1
fi

METADATA_URL="${AUTHKIT_DOMAIN%/}/.well-known/openid-configuration"

echo "Fetching AuthKit provider metadata from ${METADATA_URL} ..."
if ! metadata=$(curl -fsSL "${METADATA_URL}"); then
  echo "✗ Failed to download metadata; check AUTHKIT_DOMAIN and network."
  exit 1
fi

echo "✓ Metadata retrieved (showing issuer line for brevity):"
echo "${metadata}" | grep -o '"issuer"\s*:\s*"[^"]*"' || echo "issuer field not found"

if [[ -n "${AGILEPLUS_MCP_BASE_URL:-}" ]]; then
  MCP_URL="${AGILEPLUS_MCP_BASE_URL%/}/auth/metadata"
  echo "Probing local MCP metadata at ${MCP_URL} ..."
  if ! curl -fsSL "${MCP_URL}" >/dev/null; then
    echo "⚠ Unable to reach local MCP metadata endpoint."
  else
    echo "✓ Local MCP metadata endpoint responded."
  fi
else
  echo "AGILEPLUS_MCP_BASE_URL not set; skip MCP metadata check."
fi

echo "AuthKit smoke test completed. Compare the output above to the published provider metadata to confirm connectivity."
