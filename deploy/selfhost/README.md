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

`agileplus-api.service` is the native Linux supervisor template. Provision an
`agileplus` service account with `/var/lib/agileplus` as its home, an actual Git repository at
`/srv/agileplus/repository`, and a verified release binary at
`/opt/agileplus/bin/agileplus-api`. Supply `AGILEPLUS_API_KEY` through
`/etc/agileplus/api.env` (root-owned, mode 0600). For a headless host, also supply
`AGILEPLUS_CREDENTIAL_KEY` for the encrypted credential-store fallback; retain
that key with the protected recovery secrets. The service account home must
be writable so its `.agileplus/credentials.enc` can be created. Install the unit into
`/etc/systemd/system/` after reviewing host-specific paths. The unit keeps the
API on loopback, stores SQLite under `/var/lib/agileplus`, and restarts failed
processes. It does not start or configure the shared Caddy/tailnet services.
Use `systemd-analyze verify deploy/selfhost/agileplus-api.service` on the target
host, then prove health, kill/restart, reboot, and restore behavior there.
The unit's existence is not evidence of an installed or recovered service.

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

## Optional single-origin private operator alpha

The fastest **browser-accessible private alpha** can use one extra site fragment
in the **same shared host-level Caddy process**:

`deploy/selfhost/Caddy.operator-alpha.caddy`

```text
authorized tailnet browser
  -> https://agileplus.pheno.studio (private split-DNS answer)
  -> shared Caddy: HTTP Basic operator authentication
  -> /api/*: local AgilePlus API with host-injected X-API-Key
  -> other paths: Vercel static frontend via its distinct *.vercel.app origin
```

This avoids a second per-project proxy, exposes no backend key in JavaScript,
and avoids assuming Vercel cloud functions are on the tailnet. Keep the public
Cloudflare DNS entry for `agileplus.pheno.studio` pointing to Vercel if desired;
the **authorized tailnet's private DNS** should override that exact hostname
to the internal edge. Do not point the Caddy frontend upstream at the same
product hostname or it may recurse through split DNS.

The host Caddy needs these **host-only** values:

- `AGILEPLUS_OPERATOR_USERNAME`
- `AGILEPLUS_OPERATOR_PASSWORD_HASH` (generated with `caddy hash-password`)
- `AGILEPLUS_API_KEY` (same protected API key used by the backend)
- `AGILEPLUS_FRONTEND_UPSTREAM_HOST` (actual Vercel deployment hostname,
  without `https://`, never the custom product hostname)

Generate a strong secret and hash it using the installed Caddy binary; do not
check either plaintext or hash into source. The host must have TLS certificates
for the owned product domain, typically via Cloudflare DNS-01, and must bind
or firewall this vhost to authorized tailnet access. A private DNS name by
itself is **not** an access-control mechanism.

Before marking operator alpha usable:

1. validate Caddy with the actual host-only environment set;
2. unauthenticated browser request receives 401;
3. authenticated browser loads the real Vercel frontend via the shared Caddy;
4. same-origin `/api/v1/features` reaches desktop SQLite, not a stub;
5. acceptance returns a real immutable receipt and survives refresh/restart;
6. backend key and Basic credential are absent from browser assets and upstream
   frontend requests;
7. a non-tailnet device cannot reach the private edge;
8. test CSRF/origin protections on all mutating browser actions.

The operator site rejects non-GET/HEAD/OPTIONS API requests unless `Origin`
is exactly `https://agileplus.pheno.studio`. Missing or `null` origins are
rejected too. Nonbrowser clients should use the separate authenticated API
hostname. This policy does not prevent a compromised same-origin script from
acting as the operator. The local gateway witness exercises real Caddy with
disposable upstreams; run it with `CADDY_BIN=/path/to/caddy python3 -m unittest
discover -s tests/ops -p 'test_operator_gateway.py' -v`.

**Security limit:** Basic authentication plus a shared backend operator
credential is acceptable only for a tightly controlled single-operator alpha.
It is not per-user authorization or a production SaaS session model. A
compromised frontend script could still act with that operator's authority.
Do not expand access to external collaborators/public users without a
first-class identity and authorization layer and tested CSRF protections.

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

## Persistent SQLite backup / restore

The desktop's writable SQLite database needs snapshots that include committed
WAL transactions. **Do not copy the live `.db` file directly**: a file-only
copy can omit WAL data or be inconsistent.

When running AgilePlus as a native host service with a filesystem database:

```sh
python3 scripts/backup-agileplus-sqlite.py \\
  --database /srv/agileplus/data/agileplus.db \\
  --output-dir /srv/agileplus/backups
```

The script uses the SQLite online backup API, verifies `PRAGMA integrity_check`,
and atomically renames the verified snapshot. It deliberately performs no
automatic pruning. Configure scheduled execution and retention **separately**
after testing restore and disk-space policy.

For a restoration drill, stop the application writer, copy a verified snapshot
to a *new* database path, start a test instance against that restored path,
and verify Feature, WorkPackage, audit, event, and acceptance-receipt identity.
Never overwrite the production file while the API still has it open. A
Compose-managed named volume is not the same as a host-visible SQLite path;
run the backup tool in a maintenance environment with that volume mounted or
adopt a native host-path deployment. Do not assume the native path example
above works for the named Compose volume.

The CI backup smoke tests validate WAL consistency and offline restoration on
a disposable fixture. They do **not** establish that a real desktop backup
schedule, encrypted off-host copy, or disaster recovery procedure exists.

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
