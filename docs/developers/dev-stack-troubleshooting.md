# Dev Stack Troubleshooting

The AgilePlus dev stack is a `process-compose` orchestration of:

- OrbStack containers: Postgres 16, Dragonfly (Redis-compatible).
- Native services: NATS (`:4222`/`:8222`), Neo4j (`:7687`/`:7474`), MinIO (`:9000`/`:9001`).
- Plane.so fork: Django API (`:8000`), React-Router 7 web (`:3100`), Celery beat + worker.
- AgilePlus Rust API (`:3000`).

Start / stop:

```
task dev:up     # mkdir logs + bash scripts/orb-up.sh + process-compose up -D
task dev:down   # process-compose down + bash scripts/orb-down.sh
task dev:status # orb list + process-compose process list
```

Logs live under `.agileplus/logs/<service>.log`. `process-compose.log` is the
orchestrator log.

## Expected healthy HTTP

| Endpoint                              | Status  | Notes                               |
|---------------------------------------|---------|-------------------------------------|
| `http://localhost:8222/healthz`       | 200     | NATS monitoring port                |
| `http://localhost:9000/minio/health/live` | 200 | MinIO                               |
| `http://localhost:8000/api/`          | 200/302 | Plane Django API                    |
| `http://localhost:3100/`              | 200     | Plane web (react-router build)      |
| `http://localhost:3000/health`        | 200     | AgilePlus Rust API                  |

## Common failures and fixes

### 1. `orb-containers` exits immediately with `Permission denied`

**Symptom**: `.agileplus/logs/orb-containers.log` shows:
```
sh: scripts/orb-up.sh: Permission denied
```

**Fix**: `chmod +x scripts/orb-up.sh scripts/orb-down.sh scripts/resolve-local-ports.sh`.
These scripts are tracked; ensure git preserves the exec bit (`git update-index --chmod=+x scripts/*.sh`).

### 2. `neo4j` fails with `Neo4j is already running (pid:NNN)`

**Symptom**: `.agileplus/logs/neo4j.log` reports an existing pid but `lsof -i :7687`
returns nothing. This is a stale pidfile left by a previous crash.

**Fix**:
```
rm -f /opt/homebrew/var/neo4j/run/neo4j.pid /opt/homebrew/var/neo4j/neo4j.pid
```
Also remove Homebrew log directory permission issues by running `neo4j console` once
manually so any log files under `/opt/homebrew/var/log/neo4j/` are owned by the current user.

### 3. `plane-api` fails with `ModuleNotFoundError: No module named 'django'`

**Symptom**: process-compose had `cd .agileplus/plane/apiserver`, but the Plane fork
structure is `.agileplus/plane/apps/api/`. The venv never resolves.

**Fix** (already applied to `process-compose.yml`): path is now
`.agileplus/plane/apps/api` and settings module is forced to
`plane.settings.local` for dev.

Also ensure the venv is populated:

```
cd .agileplus/plane/apps/api
python3 -m venv .venv
.venv/bin/pip install -r requirements/local.txt
.venv/bin/python manage.py migrate
```

### 4. `plane-web` serves 404 on `/`

**Symptom**: Plane web is running but every route returns 404.

**Cause**: The Plane fork migrated from Next.js to React-Router 7. Any
`npx next start` command is wrong.

**Fix** (already applied to `process-compose.yml`): command is now
```
cd .agileplus/plane/apps/web \
  && { [ -d build/client ] || pnpm exec react-router build; } \
  && pnpm exec serve -s build/client -l 3100
```
If `build/client` is stale after a code change, delete it and restart the service:

```
rm -rf .agileplus/plane/apps/web/build
process-compose process restart plane-web
```

### 5. `agileplus-api` never starts (cargo compile error)

**Symptom**: `cargo run --release -p agileplus-api` fails with a compile error in
`agileplus-fixtures` (`summary` field missing from `WorkPackage`).

**Fix**: open a separate spec to fix `crates/agileplus-fixtures/src/builders.rs`.
As a stopgap while that is in flight, comment out the fixtures member in the
workspace `Cargo.toml` or gate the offending builder behind `#[cfg(feature = "fixtures")]`.

### 6. Port collision with other local dev stacks

**Symptom**: `orb-up.sh` reports `Host port 5432 is already in use` or
`Host port 6379 is already in use`, or MinIO fails to bind `:9000`.

**Cause**: Other Docker stacks (for example `dev-postgres-1`, `dev-redis-1`,
`sonarqube`, Firecrawl) hold the canonical ports.

**Fix options**:

- Stop the colliding stack: `docker stop dev-postgres-1 dev-redis-1 sonarqube`.
- Use `scripts/resolve-local-ports.sh` and export the randomized ports into your
  shell *before* `task dev:up`. Note: `process-compose.yml` currently hardcodes
  canonical ports; this requires rewriting the compose file to consume
  `${AGILEPLUS_*_PORT}` env vars. Tracked as a follow-up.

## Known structural gaps

These are not fixed by the current troubleshooting doc; they require dedicated
specs:

1. **`process-compose.yml` ports are hardcoded** (5432, 6379, 9000, 8000, 3100,
   3000, 7687). `scripts/resolve-local-ports.sh` randomizes ports and writes
   `.agileplus/runtime/local-ports.env`, but nothing consumes that file. Plane
   `apps/api/.env` and `apps/web/.env` capture whatever ports were resolved on a
   past run, which drift from both the compose file and the current resolver
   output.
2. **`agileplus-fixtures` compile error** blocks `cargo run -p agileplus-api`.
3. **No `.env` at repo root** — `process-compose.yml` references
   `${PLANE_POSTGRES_PASSWORD}` etc., but relies on default-literal fallbacks.
   A tracked `.env.example` -> `.env` step would make substitutions explicit.

## Quick verification

```
curl -sSf http://localhost:8222/healthz && echo ' nats ok'
curl -sSf http://localhost:9000/minio/health/live && echo ' minio ok'
curl -sSf http://localhost:8000/api/ | head -c 200 && echo
curl -sSf http://localhost:3100/ | head -c 200 && echo
curl -sSf http://localhost:3000/health && echo ' agileplus-api ok'
```

Any non-zero exit here is a real failure; tail the corresponding log under
`.agileplus/logs/` and cross-reference the table above.
