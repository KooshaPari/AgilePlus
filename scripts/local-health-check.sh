#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
PORTS_FILE="$PROJ_DIR/.agileplus/runtime/local-ports.env"

if [[ ! -f "$PORTS_FILE" ]]; then
  echo "Missing $PORTS_FILE. Run scripts/dev-up.sh first." >&2
  exit 1
fi

source "$PORTS_FILE"

check_url() {
  local name="$1"
  local url="$2"

  if curl -fsS --max-time 5 "$url" >/dev/null; then
    echo "PASS  $name  $url"
    return 0
  fi

  echo "FAIL  $name  $url" >&2
  return 1
}

check_url "nats-http" "http://localhost:${AGILEPLUS_NATS_HTTP_PORT}/healthz"
check_url "minio" "http://localhost:${AGILEPLUS_MINIO_PORT}/minio/health/live"
check_url "plane-api" "http://localhost:${AGILEPLUS_PLANE_API_PORT}/api/health"
check_url "agileplus-api" "http://localhost:${AGILEPLUS_API_PORT}/health"
