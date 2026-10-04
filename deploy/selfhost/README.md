# AgilePlus desktop-hosted API

This is the zero-paid-backend deployment target for the next AgilePlus release.

Canonical network doctrine:
`docs/architecture/NETWORK_DEPLOYMENT_DOCTRINE.md`

## Default private/operator topology

- **Frontend:** Vercel Hobby static/dashboard deployment.
- **Private network:** Tailscale tailnet.
- **Service identity:** prefer Tailscale Services/stable service naming over a
  physical desktop hostname where practical.
- **Backend:** `agileplus-api` on the owner's desktop with persistent SQLite.
- **Host edge:** one host-level Caddy.
- **DNS:** owned product names, resolved through split DNS/private resolver for
  tailnet-only services.
- **Cost target:** $0 incremental hosting spend.

Cloudflare Tunnel is **not required** for the private/operator release. It is an
optional later public-ingress profile when arbitrary non-tailnet users must
reach the backend.

The old CLI-only self-host description is obsolete: AgilePlus now has a native
Rust HTTP API. The CLI remains supported but is not the deployment boundary.

## Required runtime secret

Create an uncommitted `.env.selfhost` for the API bootstrap:

```env
HOSTNAME=api.agileplus.pheno.studio
AGILEPLUS_API_KEY=
```

For the optional public-ingress profile only:

```env
CF_TUNNEL_TOKEN=
```

Never commit credentials.

## Bootstrap with Compose

```sh
docker compose -f deploy/selfhost/docker-compose.selfhost.yml \
  --env-file .env.selfhost up -d
```

The API stores SQLite at `/data/agileplus.db` in the `agileplus-data`
volume.

Compose is a bootstrap/test packaging path. The intended long-term desktop host
uses native process supervision (for example systemd) for tailscaled, Caddy,
AgilePlus and other justified services.

## Tailnet integration

The private release should route owned product service names over the tailnet:

```
api.agileplus.pheno.studio
  -> split DNS
  -> Tailscale service/tailnet address
  -> host-level Caddy
  -> agileplus-api
```

Do not expose a physical `.ts.net` device name as the product contract.

## Browser authentication warning

`AGILEPLUS_API_KEY` must never be embedded in a Vite/public browser bundle.
Before the SaaS-style dashboard release, close an identity-aware browser path
using tailnet identity, a server-side BFF/proxy, or proper application user
authentication.

## Optional future public ingress

When a user who is not in the tailnet must connect:

```sh
docker compose -f deploy/selfhost/docker-compose.selfhost.yml \
  --env-file .env.selfhost --profile public-ingress up -d
```

The public-ingress profile may use Cloudflare Tunnel, but the backend topology
and product naming must remain unchanged.

## Release gates

1. canonical Rust acceptance tests execute and pass;
2. real-SQLite HTTP acceptance tests execute and pass;
3. tailnet API naming/routing resolves from an authorized remote device;
4. Caddy routes the product hostname to the real API;
5. browser frontend contains no operator secret;
6. production browser auth/API-routing contract is closed;
7. dashboard reads real backend state;
8. SQLite backup + restore is tested;
9. desktop reboot recovery is tested;
10. public/stub fallback is never mistaken for real product state.
