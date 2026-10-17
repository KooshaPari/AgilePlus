# planify-shim

A thin REST adapter that exposes **Plane's HTTP API shape** on top of the
**AgilePlus Axum backend**, so the **Planify** frontend (`apps/web`, a Plane
fork) can render a populated board against real AgilePlus data — without any
changes to AgilePlus or to Planify's frontend code.

This makes Planify viable as **frontend candidate #1** for AgilePlus.

## Why a shim

| Planify (`apps/web`) expects | AgilePlus provides |
| --- | --- |
| Plane REST at `VITE_API_BASE_URL` (default `:8000`) | Rust Axum API on `:4000` |
| `/api/workspaces/`, `/api/.../projects/`, `/api/.../issues/`, CSRF+session cookie auth | `/api/v1/features`, `/api/v1/features/:slug/work-packages`, `X-API-Key`/Bearer auth |
| Plane object model (workspace / project / issue / state) | AgilePlus model (feature / work-package / state-machine) |

The shim sits on `:8000`, speaks Plane's dialect to the browser, and translates
to AgilePlus's REST + auth on the backend.

## Mapping

| Plane concept | AgilePlus source |
| --- | --- |
| Workspace (single, slug `agileplus`) | the AgilePlus root |
| Project | `Feature` (`GET /api/v1/features`) |
| Project states (board columns) | eight explicit columns: Created, Planned, Implementing, In Review, Blocked, Validated Feature, Work Package Done, Shipped Feature. WorkPackage Done is **not** Feature Shipped. |
| Issue | `WorkPackage` (`GET /api/v1/features/:slug/work-packages`) |
| Issue state group | `WorkPackage.state` snapped onto the nearest column group |
| `me` user / members / labels | local stubs (read-only review) |

UUIDs required by Plane are derived deterministically (SHA-256 of a namespaced
key) so the same feature/work-package always maps to the same id.

## Endpoints implemented (read path for a board)

- `GET /auth/get-csrf-token/` — stub CSRF token + cookie
- `GET /api/instances/` — instance config (setup done, signup off)
- `GET /api/users/me/` (+ `/profile/`, `/settings/`, `/workspaces/`)
- `GET /api/workspaces/:slug/` (+ `/members/`)
- `GET /api/workspaces/:slug/projects/` (+ `/details/`, `/:id/`)
- `GET /api/workspaces/:slug/projects/:id/project-members/me/`, `/members/`
- `GET /api/workspaces/:slug/projects/:id/states/`
- `GET /api/workspaces/:slug/projects/:id/issue-labels/`
- `GET /api/workspaces/:slug/projects/:id/issues/` → Plane `TIssuesResponse`
- `GET /shim/health` — shim + AgilePlus reachability check

Auth is a **local-review stub**: a fixed CSRF token and a synthetic "me" user
are returned so the SPA treats the session as authenticated. The shim itself
authenticates to AgilePlus with `X-API-Key`. **This is never an authentication
gateway and must not be exposed beyond loopback.** It binds to `127.0.0.1`
only, requires an explicit `AGILEPLUS_API_KEY`, times out upstream calls,
rejects every non-read request with HTTP 405, and returns HTTP 404 for
unimplemented reads instead of disguising missing functionality as empty data.
Malformed upstream features or work packages result in HTTP 502. Production
identity and authorization integration remains an unsatisfied adoption gate.

Local smoke/contract tests: `npm install --no-audit --no-fund`, then `npm test`.
Tests use a synthetic AgilePlus HTTP fixture and never write to real AgilePlus,
Plane, Planify or desk databases.

## Run sequence + ports

```bash
# 1. AgilePlus API on :4000 (from the AgilePlus repo root)
AGILEPLUS_API_KEY=dev-api-key cargo run -p agileplus-api      # serves :4000

# 2. The shim on :8000 (this dir)
npm install
AGILEPLUS_API_URL=http://localhost:4000 \
AGILEPLUS_API_KEY=dev-api-key \
PORT=8000 \
WEB_ORIGIN=http://localhost:4400 \
npm start                                                     # serves :8000

# 3. Planify frontend on :4400 (from C:/Users/koosh/Dev/Planify)
#    apps/web/.env already has VITE_API_BASE_URL="http://localhost:8000"
pnpm --filter web dev                                         # serves :4400
```

Open `http://localhost:4400`. The board renders the AgilePlus workspace with
one project per Feature and one card per Work-Package, laid out across the
state columns.

## Config (env)

| Var | Default | Meaning |
| --- | --- | --- |
| `PORT` | `8000` | shim listen port (must match `VITE_API_BASE_URL`) |
| `AGILEPLUS_API_URL` | `http://localhost:4000` | AgilePlus Axum base URL |
| `AGILEPLUS_API_KEY` | **required** | local test key sent as `X-API-Key`; no implicit credential |
| `WEB_ORIGIN` | `http://localhost:4400` | allowed credentialed CORS origin |

## What renders / what doesn't

**Renders:** workspace, project list, board columns (states), issues/cards
grouped by state, basic project + issue detail. Verified end-to-end against a
fixtured AgilePlus (features → projects, work-packages → issues, correct
Plane state groups).

**Gaps (intentional, read-only candidate review):**
- Writes (create/update/transition) are not mapped — board is read-only.
- Labels, cycles, modules, members beyond `me`, comments, attachments return
  empty stubs.
- Auth is a stub (no real login/session); **loopback-only; not for shared/prod use**.
- Unimplemented endpoints return 404, not synthetic success; frontend gaps stay visible.
- One synthetic workspace; AgilePlus modules/projects hierarchy is flattened
  to Feature=Project.

## State authority distinction

The board is a **projection**: an AgilePlus `WorkPackage.state=done` appears
in **Work Package Done**, never **Shipped Feature**. The latter is a separate
product/feature state and cannot be inferred from a work package finishing.
Blocked and In Review remain explicit columns. Native AgilePlus acceptance
and Tracera product verification remain their respective governed boundaries.
This shim does not implement product-acceptance writes or prove verified
usability. Its custom `_agileplus_*` fields carry local source provenance for
reconciliation; these fields are not native Plane API guarantees.
