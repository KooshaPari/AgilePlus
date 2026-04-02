#!/usr/bin/env bash
set -euo pipefail

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

check_url "nats-http" "http://localhost:8222/healthz"
check_url "minio" "http://localhost:9000/minio/health/live"
check_url "plane-api" "http://localhost:8000/"
check_url "plane-web" "http://localhost:3100/"
check_url "agileplus-api" "http://localhost:3000/health"
