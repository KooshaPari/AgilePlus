#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
PLANE_API_DIR="$PROJ_DIR/.agileplus/plane/apps/api"

source "$SCRIPT_DIR/resolve-local-ports-codex-local.sh"
source "$PROJ_DIR/.agileplus/runtime/local-ports.env"

cd "$PLANE_API_DIR"

export DATABASE_URL="postgresql://agileplus:${PLANE_POSTGRES_PASSWORD:-agileplus-dev}@127.0.0.1:${AGILEPLUS_POSTGRES_PORT}/plane"
export REDIS_URL="redis://127.0.0.1:${AGILEPLUS_REDIS_PORT}"
export SECRET_KEY="${PLANE_SECRET_KEY:-agileplus-dev-secret-key}"
export WEB_URL="http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}"
export CORS_ALLOWED_ORIGINS="http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT},http://127.0.0.1:${AGILEPLUS_API_PORT}"
export APP_BASE_URL="http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}"
export LIVE_BASE_URL="http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}"
export LIVE_BASE_PATH="/live"
export AWS_S3_ENDPOINT_URL="http://127.0.0.1:${AGILEPLUS_MINIO_PORT}"
export AWS_ACCESS_KEY_ID="agileplus"
export AWS_SECRET_ACCESS_KEY="agileplus-dev"
export AWS_S3_BUCKET_NAME="uploads"
export USE_MINIO="1"
export AMQP_URL="${AMQP_URL:-memory://}"
export PYTHONUNBUFFERED="1"

machine_signature="$(
  .venv/bin/python - <<'PY'
import hashlib
import socket

print(hashlib.sha256(socket.gethostname().encode()).hexdigest())
PY
)"

prepare_plane_api() {
  .venv/bin/python manage.py wait_for_db --settings=plane.settings.local
  .venv/bin/python manage.py migrate --settings=plane.settings.local --noinput
  .venv/bin/python manage.py register_instance "$machine_signature" --settings=plane.settings.local
  .venv/bin/python manage.py configure_instance --settings=plane.settings.local
  .venv/bin/python manage.py create_bucket --settings=plane.settings.local
  .venv/bin/python manage.py clear_cache --settings=plane.settings.local
}

if [[ "${1:-}" == "--prepare-only" ]]; then
  prepare_plane_api
  exit 0
fi

if [[ "${1:-}" != "--serve-only" ]]; then
  prepare_plane_api
fi

exec .venv/bin/python manage.py runserver "0.0.0.0:${AGILEPLUS_PLANE_API_PORT}" --settings=plane.settings.local
