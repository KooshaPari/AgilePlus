#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJ_DIR"

source "$SCRIPT_DIR/resolve-local-ports-codex-local.sh"
source "$PROJ_DIR/.agileplus/runtime/local-ports.env"

DRAGONFLY_NAME="agileplus-dragonfly"
POSTGRES_NAME="agileplus-postgres"
POSTGRES_USER="agileplus"
POSTGRES_PASSWORD="${PLANE_POSTGRES_PASSWORD:-agileplus-dev}"
POSTGRES_DB="plane"

select_postgres_image() {
  if [[ -n "${AGILEPLUS_POSTGRES_IMAGE:-}" ]]; then
    printf '%s\n' "${AGILEPLUS_POSTGRES_IMAGE}"
    return 0
  fi

  local cached_candidates=(
    "postgres:15.7-alpine"
    "postgres:16-alpine"
  )
  local image

  for image in "${cached_candidates[@]}"; do
    if docker image inspect "${image}" >/dev/null 2>&1; then
      printf '%s\n' "${image}"
      return 0
    fi
  done

  echo "No cached Postgres image available. Set AGILEPLUS_POSTGRES_IMAGE to an explicit local image." >&2
  exit 1
}

POSTGRES_IMAGE="$(select_postgres_image)"

recreate_if_needed() {
  local name="$1"
  local host_port="$2"
  local container_port="$3"

  current_port() {
    docker inspect \
      --format "{{(index (index .HostConfig.PortBindings \"${container_port}/tcp\") 0).HostPort}}" \
      "$name" 2>/dev/null || true
  }

  if docker ps --format '{{.Names}}' | grep -q "^${name}$"; then
    if [[ "$(current_port)" == "${host_port}" ]]; then
      echo "${name} is already running on host port ${host_port}"
      return 1
    fi
    docker stop "${name}" >/dev/null 2>&1 || true
    docker rm "${name}" >/dev/null 2>&1 || true
  elif docker ps -a --format '{{.Names}}' | grep -q "^${name}$"; then
    docker rm "${name}" >/dev/null 2>&1 || true
  fi

  if lsof -iTCP:"${host_port}" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "Host port ${host_port} is already in use." >&2
    exit 1
  fi
  return 0
}

if recreate_if_needed "${DRAGONFLY_NAME}" "${AGILEPLUS_REDIS_PORT}" 6379; then
  docker run -d \
    --name "${DRAGONFLY_NAME}" \
    -p "${AGILEPLUS_REDIS_PORT}:6379" \
    docker.dragonflydb.io/dragonflydb/dragonfly:latest \
    --maxmemory=4gb \
    --bind 0.0.0.0 >/dev/null
fi

if recreate_if_needed "${POSTGRES_NAME}" "${AGILEPLUS_POSTGRES_PORT}" 5432; then
  docker run -d \
    --name "${POSTGRES_NAME}" \
    -p "${AGILEPLUS_POSTGRES_PORT}:5432" \
    -e "POSTGRES_USER=${POSTGRES_USER}" \
    -e "POSTGRES_PASSWORD=${POSTGRES_PASSWORD}" \
    -e "POSTGRES_DB=${POSTGRES_DB}" \
    "${POSTGRES_IMAGE}" >/dev/null
fi

for _ in $(seq 1 90); do
  if redis-cli -h localhost -p "${AGILEPLUS_REDIS_PORT}" ping 2>/dev/null | grep -q PONG \
    && pg_isready -h localhost -p "${AGILEPLUS_POSTGRES_PORT}" >/dev/null 2>&1; then
    exit 0
  fi
  sleep 2
done

echo "OrbStack containers did not become ready in time." >&2
exit 1
