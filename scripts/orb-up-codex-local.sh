#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/resolve-local-ports-codex-local.sh"

DRAGONFLY_NAME="agileplus-dragonfly"
POSTGRES_NAME="agileplus-postgres"
POSTGRES_USER="agileplus"
POSTGRES_PASSWORD="${PLANE_POSTGRES_PASSWORD:-agileplus-dev}"
POSTGRES_DB="plane"

start_container() {
  local name="$1"
  local image="$2"
  local host_port="$3"
  local container_port="$4"
  shift 4

  local docker_args=()
  local command_args=()
  local parsing_command_args=false

  for arg in "$@"; do
    if [[ "$arg" == "--" ]]; then
      parsing_command_args=true
      continue
    fi
    if [[ "$parsing_command_args" == true ]]; then
      command_args+=("$arg")
    else
      docker_args+=("$arg")
    fi
  done

  current_port() {
    docker inspect \
      --format "{{(index (index .HostConfig.PortBindings \"${container_port}/tcp\") 0).HostPort}}" \
      "$name" 2>/dev/null || true
  }

  if docker ps --format '{{.Names}}' | grep -q "^${name}$"; then
    if [[ "$(current_port)" == "${host_port}" ]]; then
      echo "${name} is already running on host port ${host_port}"
      return 0
    fi
    docker stop "${name}" >/dev/null 2>&1 || true
    docker rm "${name}" >/dev/null 2>&1 || true
  elif docker ps -a --format '{{.Names}}' | grep -q "^${name}$"; then
    docker rm "${name}" >/dev/null 2>&1 || true
  fi

  if lsof -iTCP:"${host_port}" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "Host port ${host_port} is already in use." >&2
    return 1
  fi

  local docker_run_args=(-d --name "${name}" -p "${host_port}:${container_port}")
  if [[ ${#docker_args[@]} -gt 0 ]]; then
    docker_run_args+=("${docker_args[@]}")
  fi
  docker_run_args+=("${image}")
  if [[ ${#command_args[@]} -gt 0 ]]; then
    docker_run_args+=("${command_args[@]}")
  fi
  docker run "${docker_run_args[@]}" >/dev/null
}

echo "--- Starting Dragonfly (Redis-compatible cache) ---"
start_container \
  "${DRAGONFLY_NAME}" \
  "docker.dragonflydb.io/dragonflydb/dragonfly:latest" \
  "${AGILEPLUS_REDIS_PORT}" \
  6379 \
  -- \
  --maxmemory=4gb --bind 0.0.0.0

echo "--- Starting PostgreSQL 16 ---"
start_container \
  "${POSTGRES_NAME}" \
  "postgres:16-alpine" \
  "${AGILEPLUS_POSTGRES_PORT}" \
  5432 \
  -e "POSTGRES_USER=${POSTGRES_USER}" \
  -e "POSTGRES_PASSWORD=${POSTGRES_PASSWORD}" \
  -e "POSTGRES_DB=${POSTGRES_DB}"

echo "Waiting for containers to become ready..."
for i in $(seq 1 30); do
  local_pg_ok=false
  local_df_ok=false

  if redis-cli -h localhost -p "${AGILEPLUS_REDIS_PORT}" ping 2>/dev/null | grep -q PONG; then
    local_df_ok=true
  fi

  if pg_isready -h localhost -p "${AGILEPLUS_POSTGRES_PORT}" >/dev/null 2>&1; then
    local_pg_ok=true
  fi

  if [[ "$local_pg_ok" == true && "$local_df_ok" == true ]]; then
    echo "All OrbStack containers are ready."
    exit 0
  fi

  echo "  Waiting... (${i}/30)"
  sleep 2
done

echo "ERROR: Containers did not become ready in time" >&2
exit 1
