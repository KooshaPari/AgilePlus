# AgilePlus desktop-hosted API

This is the zero-paid-backend deployment target for the next AgilePlus release.

## Topology

- **Frontend:** static dashboard on Vercel Hobby once its production API-origin contract is closed.
- **Backend:** `agileplus-api` runs on the owner's desktop with persistent SQLite.
- **Ingress:** Cloudflare Tunnel publishes the desktop API without opening inbound router ports.
- **Reverse proxy:** Caddy forwards the public hostname to `agileplus-api:3000`.
- **Cost target:** $0 incremental hosting spend.

The old CLI-only self-host description is obsolete: AgilePlus now has a native
Rust HTTP API. The CLI remains supported but is not the public deployment
boundary.

## Required secrets

Create an uncommitted `.env.selfhost`:

```env
HOSTNAME=api.agileplus.pheno.studio
CF_TUNNEL_TOKEN=
AGILEPLUS_API_KEY=
```

Never commit either token.

## Start

```sh
docker compose -f deploy/selfhost/docker-compose.selfhost.yml \
  --env-file .env.selfhost up -d
```

The API stores its SQLite database at `/data/agileplus.db` in the
`agileplus-data` volume. The Rust API requires `AGILEPLUS_API_KEY`; protected
requests use the existing API-key authentication contract.

## Public-release gates

A public hostname is not a release by itself. Before calling this deployed:

1. current Rust acceptance tests execute and pass;
2. real-SQLite HTTP acceptance tests execute and pass;
3. CORS is explicitly restricted to the production frontend origin;
4. API key is not embedded in the public browser bundle;
5. browser-facing authentication/proxy design is closed;
6. health/readiness checks run through the tunnel;
7. SQLite backup + restore is tested;
8. desktop reboot recovery is tested;
9. Cloudflare tunnel restarts automatically;
10. dashboard has a production API-origin contract and browser smoke test.

Until browser auth is closed, the public API hostname should be treated as an
operator/API endpoint, not a directly credentialed browser backend.
