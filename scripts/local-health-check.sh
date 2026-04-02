#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
source "$SCRIPT_DIR/resolve-local-ports.sh"

check_tcp() {
  local name="$1"
  local port="$2"
  if ! timeout 3 bash -c "cat < /dev/tcp/localhost/$port" >/dev/null 2>&1; then
    printf 'port_check|%s|%s|%s\n' "$name" "$port" "closed"
    return 1
  fi
  printf 'port_check|%s|%s|%s\n' "$name" "$port" "open"
}

check_http() {
  local name="$1"
  local port="$2"
  if ! curl -fsS "http://localhost:$port/" >/dev/null; then
    printf 'http_check|%s|%s|%s\n' "$name" "$port" "failed"
    return 1
  fi
  printf 'http_check|%s|%s|%s\n' "$name" "$port" "ok"
}

set +e
check_tcp "plane-api" "$AGILEPLUS_PLANE_API_PORT"
plane_ok=$?
check_http "plane-web" "$AGILEPLUS_PLANE_WEB_PORT"
web_ok=$?
check_http "api" "$AGILEPLUS_API_PORT"
api_ok=$?
set -e

if (( plane_ok + web_ok + api_ok > 0 )); then
  echo "ERROR: health check failed; see port/http diagnostics above"
  exit 1
fi

echo "AgilePlus local services healthy (plane API, plane web, API port)"
