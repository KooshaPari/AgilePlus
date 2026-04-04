#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RUNTIME_DIR="${PROJECT_DIR}/.agileplus/runtime"
PORTS_FILE="${RUNTIME_DIR}/local-ports.env"

mkdir -p "$RUNTIME_DIR"

port_in_use() {
  local port="$1"
  lsof -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
}

pick_port() {
  local env_name="$1"
  local default_port="$2"
  local value="${!env_name:-}"

  if [[ -n "$value" ]]; then
    printf '%s' "$value"
    return 0
  fi

  local seconds_now
  seconds_now=$((10#$(date +%S)))
  local start=$((default_port + ((seconds_now + $$) % 25)))
  local candidate="$start"

  while port_in_use "$candidate"; do
    candidate=$((candidate + 1))
  done

  printf '%s' "$candidate"
}

export AGILEPLUS_POSTGRES_PORT="$(pick_port AGILEPLUS_POSTGRES_PORT 5440)"
export AGILEPLUS_REDIS_PORT="$(pick_port AGILEPLUS_REDIS_PORT 6388)"
export AGILEPLUS_NATS_PORT="$(pick_port AGILEPLUS_NATS_PORT 4231)"
export AGILEPLUS_NATS_HTTP_PORT="$(pick_port AGILEPLUS_NATS_HTTP_PORT 8232)"
export AGILEPLUS_NEO4J_PORT="$(pick_port AGILEPLUS_NEO4J_PORT 7697)"
export AGILEPLUS_NEO4J_HTTP_PORT="$(pick_port AGILEPLUS_NEO4J_HTTP_PORT 7485)"
export AGILEPLUS_MINIO_PORT="$(pick_port AGILEPLUS_MINIO_PORT 9012)"
export AGILEPLUS_MINIO_CONSOLE_PORT="$(pick_port AGILEPLUS_MINIO_CONSOLE_PORT 9013)"
export AGILEPLUS_PLANE_API_PORT="$(pick_port AGILEPLUS_PLANE_API_PORT 8013)"
export AGILEPLUS_PLANE_WEB_PORT="$(pick_port AGILEPLUS_PLANE_WEB_PORT 3113)"
export AGILEPLUS_API_PORT="$(pick_port AGILEPLUS_API_PORT 3014)"

cat >"$PORTS_FILE" <<EOF
AGILEPLUS_POSTGRES_PORT=${AGILEPLUS_POSTGRES_PORT}
AGILEPLUS_REDIS_PORT=${AGILEPLUS_REDIS_PORT}
AGILEPLUS_NATS_PORT=${AGILEPLUS_NATS_PORT}
AGILEPLUS_NATS_HTTP_PORT=${AGILEPLUS_NATS_HTTP_PORT}
AGILEPLUS_NEO4J_PORT=${AGILEPLUS_NEO4J_PORT}
AGILEPLUS_NEO4J_HTTP_PORT=${AGILEPLUS_NEO4J_HTTP_PORT}
AGILEPLUS_MINIO_PORT=${AGILEPLUS_MINIO_PORT}
AGILEPLUS_MINIO_CONSOLE_PORT=${AGILEPLUS_MINIO_CONSOLE_PORT}
AGILEPLUS_PLANE_API_PORT=${AGILEPLUS_PLANE_API_PORT}
AGILEPLUS_PLANE_WEB_PORT=${AGILEPLUS_PLANE_WEB_PORT}
AGILEPLUS_API_PORT=${AGILEPLUS_API_PORT}
EOF
