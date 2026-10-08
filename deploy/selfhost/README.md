# AgilePlus: desktop-backed private release

**Source of truth:** [network doctrine](../../docs/architecture/NETWORK_DEPLOYMENT_DOCTRINE.md)
and [forward WBS](../../verification/FORWARD_WBS_2026-10-04.md).

This directory is an **optional API-only Compose bootstrap**. It is not a
complete SaaS deployment and does not launch Caddy, Tailscale, CoreDNS or a
Cloudflare Tunnel. Those are shared, host/organization-level infrastructure.

## Topology and zero-paid-service rule

```text
Authorized tailnet browser
  → api.agileplus.pheno.studio (owned product domain)
  → Tailscale split DNS / service identity
  → ONE host-level Caddy (:443, real custom-domain certificate)
  → 127.0.0.1:3000 (desktop)
  → agileplus-api
  → persistent SQLite
```

- Porkbun: domain registrar.
- Cloudflare: public authoritative DNS and DNS-01 certificate validation.
- Tailscale: private transport, access control and optional stable Service identity.
- Desktop: stateful backend; no paid VPS/Render required.
- Vercel Hobby: eventual browser/frontend static hosting where plan terms permit.

**A public Vercel frontend does not make a private API publicly reachable.**
The authorized user's device must itself reach the tailnet API, unless a
separate authenticated public ingress/BFF has been explicitly introduced.
A Vercel serverless function cannot directly reach a tailnet-only address by
DNS aliasing alone.

## Bootstrap API only

The checked-in Compose file runs the `agileplus-api` binary against SQLite
and publishes only `127.0.0.1:3000` on the desktop. It uses a Rust image
compatible with the workspace's declared MSRV.

Set secrets in an untracked environment file, never in frontend builds:

```env
AGILEPLUS_API_KEY=<strong-operator-secret>
# Optional: explicit trusted browser origins, never wildcard
AGILEPLUS_ALLOWED_ORIGINS=https://agileplus.pheno.studio
AGILEPLUS_HOST_PORT=3000
```

Start from the repository root:

```sh
docker compose -f deploy/selfhost/docker-compose.selfhost.yml \
  --env-file .env.selfhost up -d agileplus-api
```

Then verify the actual local API rather than only container/process existence:

```sh
curl --fail http://127.0.0.1:3000/health
```

Check the API's configured port in `AppConfig` if it differs from 3000.
The durable database resides in the `agileplus-data` volume. It is not
recreated on ordinary container restart.

For the long-term native deployment, use the host's process supervisor
(systemd on Linux; a suitable native service manager on Windows) and a
persistent, backed-up data directory rather than compiling on every boot.

## Shared Caddy, not one proxy per project

`deploy/selfhost/Caddyfile` is an **importable host-level site definition**.
Its upstream is `127.0.0.1:3000`, not a Docker service DNS name.

The global Caddy configuration should import AgilePlus and Tracera's
independent site fragments. Do not launch an AgilePlus-specific Caddy or
Traefik instance. Restrict the host firewall/edge listener to the tailnet.

Because `api.agileplus.pheno.studio` is private, standard public HTTP-01 TLS
validation is not a safe assumption. Use DNS-01 with a Caddy build that has
the Cloudflare DNS plugin, or provision a trusted certificate separately.
A stock Caddy binary does not automatically include every DNS provider module.

## Browser-origin boundary

The Rust HTTP API disables cross-origin browser access by default. For a
cross-origin browser frontend, configure `AGILEPLUS_ALLOWED_ORIGINS` with
explicit comma-separated origins (for example,
`https://agileplus.pheno.studio`). Wildcards, arbitrary public HTTP origins,
and URL paths are rejected. An unset variable leaves same-origin and
nonbrowser clients unaffected but does not enable a cross-origin dashboard.

**CORS is not authentication.** An allowed origin must still use a safe
identity-aware session/BFF; never transmit the operator API key in browser
JavaScript. Avoid a direct Vercel serverless-to-tailnet route: Vercel's cloud
runtime is not a tailnet member merely because the frontend uses a familiar
domain.

## Browser credential boundary

`AGILEPLUS_API_KEY` is an operator/server credential. Never expose it as a
`VITE_*` value, browser header constant, static asset or frontend environment
variable. Tailnet reachability does **not** by itself constitute application
authorization. Before a dashboard acceptance mutation is enabled, implement
an identity-aware browser gateway/session layer with least-privilege access.

The Vite dev server's `/api → localhost:3000` proxy does not configure a
production Vercel API route.

## Public ingress is a separate future capability

When non-tailnet users must access the backend, deploy exactly one
authenticated public ingress in front of the **same global Caddy**.
Cloudflare Tunnel is one possible mechanism, but is not started by this
repository and should not be made a hard dependency of private deployments.
A public ingress also requires abuse protections, identity, authorization
and a reviewed backup/incident path.

## Deployment release gates

No "deployed" claim until all are executed:

1. `cargo test --locked` acceptance, API, CLI and gRPC recovery witnesses pass
   on the exact commit;
2. API starts and health responds locally on the desktop;
3. tailnet service/DNS resolves from a remote authorized device;
4. TLS cert matches the **owned product hostname**;
5. global Caddy forwards to the actual API with no accidental public exposure;
6. browser auth works without exposing the operator API key;
7. dashboard reads real state and handles offline/409/422/500 visibly;
8. acceptance produces a durable receipt and survives restart;
9. SQLite backup + restore is demonstrated;
10. desktop reboot and tailnet/Caddy/API recovery are demonstrated.

**No remote tunnel or host configuration has been performed by editing these
files.** That operational evidence must be collected separately.
