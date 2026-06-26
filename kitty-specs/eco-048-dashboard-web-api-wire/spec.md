# eco-048: Dashboard Web API Wire

## Goal
Wire the Vite+React dashboard at `crates/agileplus-dashboard/web/` to agileplus-api
using a typed client and live dashboard endpoints.

## Acceptance Criteria
- Typed API client uses `VITE_API_BASE` (default `http://localhost:3000`)
- Main dashboard fetch uses `GET /api/dashboard/epics-stories.json`
- Work packages use `GET /api/dashboard/work-packages.json`
- `npm install --legacy-peer-deps`, `npm run dev`, and `npm run build` succeed
- Linked PR references `spec: eco-048-dashboard-web-api-wire`
