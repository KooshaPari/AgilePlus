#!/usr/bin/env bash
# Start OrbStack containers for Dragonfly and PostgreSQL.
# OrbStack v2 exposes docker CLI - this script uses docker commands.
# Idempotent -- reuses existing containers.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/resolve-local-ports.sh"

DRAGONFLY_NAME="agileplus-dragonfly"
POSTGRES_NAME="agileplus-postgres"

POSTGRES_USER="agileplus"
POSTGRES_PASSWORD="${PLANE_POSTGRES_PASSWORD:-agileplus-dev}"
POSTGRES_DB="plane"

start_container() {
    name="$1"
    image="$2"
    host_port="$3"
    container_port="$4"
    shift 4
    docker_args=()
    command_args=()
    parsing_command_args=false

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
        if [ "$(current_port)" = "${host_port}" ]; then
            echo "${name} is already running on host port ${host_port}"
            return 0
        fi
        echo "Recreating ${name} to move host port to ${host_port}"
        docker stop "${name}" >/dev/null 2>&1 || true
        docker rm "${name}" >/dev/null 2>&1 || true
    elif docker ps -a --format '{{.Names}}' | grep -q "^${name}$"; then
        echo "Removing stale container ${name} so host port ${host_port} can be applied"
        docker rm "${name}" >/dev/null 2>&1 || true
    fi

    if lsof -iTCP:"${host_port}" -sTCP:LISTEN >/dev/null 2>&1; then
        echo "Host port ${host_port} is already in use; export a different AGILEPLUS_*_PORT first." >&2
        return 1
    fi

    echo "Creating container ${name} on host port ${host_port}"
    docker_run_args=(-d --name "${name}" -p "${host_port}:${container_port}")
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
start_container "${DRAGONFLY_NAME}" \
    "docker.dragonflydb.io/dragonflydb/dragonfly:latest" \
    "${AGILEPLUS_REDIS_PORT}" \
    6379 \
    -- \
    --maxmemory=4gb --bind 0.0.0.0

echo "--- Starting PostgreSQL 16 ---"
start_container "${POSTGRES_NAME}" \
    "postgres:16-alpine" \
    "${AGILEPLUS_POSTGRES_PORT}" \
    5432 \
    -e "POSTGRES_USER=${POSTGRES_USER}" \
    -e "POSTGRES_PASSWORD=${POSTGRES_PASSWORD}" \
    -e "POSTGRES_DB=${POSTGRES_DB}"

echo "Waiting for containers to become ready..."
for i in $(seq 1 30); do
    pg_ok=false
    df_ok=false

    if redis-cli -h localhost -p "${AGILEPLUS_REDIS_PORT}" ping 2>/dev/null | grep -q PONG; then
        df_ok=true
    fi

    if pg_isready -h localhost -p "${AGILEPLUS_POSTGRES_PORT}" 2>/dev/null; then
        pg_ok=true
    fi

    if [ "$pg_ok" = true ] && [ "$df_ok" = true ]; then
        echo "All OrbStack containers are ready."
        exit 0
    fi
    echo "  Waiting... (${i}/30)"
    sleep 2
done

echo "ERROR: Containers did not become ready in time" >&2
exit 1
