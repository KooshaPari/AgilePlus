# Network and deployment doctrine

Status: canonical for the AgilePlus recovery branch as of 2026-10-04.

## Core rule

The tailnet is the private organizational network fabric. Product-facing names
remain first-class domains; machine names and `.ts.net` addresses are plumbing.

Default private/operator path:

```
product domain
  -> split DNS
  -> Tailscale service / tailnet route
  -> host edge
  -> one host-level Caddy
  -> localhost application process
```

Cloudflare Tunnel is **not** the default private transport. It is an optional
future public-ingress layer when arbitrary non-tailnet users must reach the
desktop-hosted backend.

## DNS ownership

- Registrar: Porkbun.
- Public authoritative DNS: Cloudflare.
- Private service resolution: Tailscale split DNS to an internal resolver.
- Do not hijack all public product DNS internally unless the internal resolver
  mirrors the whole public zone. Prefer exact/private namespaces or explicitly
  managed product API names.

Example logical names:

- `agileplus.pheno.studio` — public/frontend identity.
- `api.agileplus.pheno.studio` — private/operator API identity initially.
- `grpc.agileplus.pheno.studio` — private gRPC identity when exposed.
- `mcp.agileplus.pheno.studio` — private MCP identity when exposed.
- `*.infra.pheno.studio` — organizational infrastructure.

The exact domain inventory may evolve; the rule is that product identity does
not encode the physical host.

## Tailscale

Prefer Tailscale Services where practical so clients address a stable service,
not a desktop node. Service identity must survive moving the backend to a new
host.

Use ACLs/grants for network authorization. For browser/operator services,
Tailscale Serve identity headers or another identity-aware edge can eliminate
the need to expose a static operator API key to frontend JavaScript.

Important boundary: a tailnet-only service is not public SaaS. Public SaaS
requires a public ingress layer later.

## Reverse proxy

Run one Caddy instance per physical host, not one proxy stack per project.

Project-specific routing belongs in imported Caddy config fragments:

```
Caddy
  api.agileplus.pheno.studio -> 127.0.0.1:3000
  grpc.agileplus.pheno.studio -> local gRPC listener
  ...
```

Do not stack Caddy + Traefik + project Caddy without a concrete capability gap.
Traefik becomes relevant only if dynamic container/Kubernetes service discovery
actually justifies it.

## TLS

For owned custom domains on private services, prefer certificates for the real
product hostname, using DNS-01 against Cloudflare when necessary. Do not expose
`.ts.net` names as the product contract merely because Tailscale Serve can
issue certificates for them.

## Desktop as zero-cost infrastructure host

The owner's desktop is the default stateful compute host when a free managed
service is unavailable or unsuitable.

Long-term host shape:

```
systemd / process supervisor
  tailscaled
  caddy
  internal DNS resolver if needed
  agileplus-api
  workers
  optional stateful services
```

Compose can remain a bootstrap/test packaging option, but should not become a
mandatory architectural layer.

## Frontend

Vercel Hobby remains the default zero-cost frontend/static deployment target
when plan limits and usage terms fit the release.

A browser bundle must never contain `AGILEPLUS_API_KEY` or another operator
secret. A SaaS-style browser release therefore needs one of:

1. tailnet identity / identity-aware private edge;
2. a server-side BFF/proxy that owns the backend credential;
3. proper application user authentication.

For the private/operator alpha, prefer tailnet identity over adding a public
tunnel only to solve authentication.

## Public release later

When arbitrary users who are not tailnet members must connect, add one public
ingress layer in front of the same host edge. Cloudflare Tunnel is an option at
that stage; it is not foundational to the internal architecture.

The backend process, service identity, routing rules and application semantics
must not need to change merely because public ingress is added.
