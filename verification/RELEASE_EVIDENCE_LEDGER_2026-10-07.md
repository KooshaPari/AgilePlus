# AgilePlus Release Evidence Ledger — 2026-10-07

Scope: mature-first AgilePlus branch `spec/mature-product-contract-v2`.
Use this ledger with `AGENTS.md`, `spec/product/AUTHORITY_INDEX.md`,
`verification/FORWARD_WBS_2026-10-04.md`, and
`docs/architecture/NETWORK_DEPLOYMENT_DOCTRINE.md`.

## Engineering status is not deployment status

**Implementation/source:**

- Canonical SQLite atomic acceptance and command/receipt architecture are
  implemented.
- HTTP and authenticated gRPC acceptance witnesses have executed on recovery
  candidates.
- The shared promotion preflight and CLI shipping tests are under active repair.
- Private desktop bootstrap is API-only, loopback-bound, and designed for one
  host-level Caddy.
- Rust HTTP CORS is fail-closed by default; explicit trusted frontend origins
  must be configured where cross-origin requests are actually required.
- A private single-operator Caddy site definition exists for the owned product
  hostname, preserving `/api/*` on the same origin while proxying static
  frontend assets from a distinct Vercel deployment hostname.
- A WAL-consistent SQLite backup script and disposable offline restore smoke
  tests exist.

**Native CI evidence as of this snapshot:**

- Recovery Acceptance Witnesses run `37708831067` (commit `6b368c3`)
  passed SQLite atomic acceptance, application unit tests, migrations, mounted
  HTTP, and authenticated gRPC, but failed CLI shipping tests (7 pass / 7 fail).
- Later commits `78dd0ca` and `e63755c` changed promotion error reporting
  and legacy ship test expectations. Their correctness is not inferred from
  the prior red run.
- Recovery Acceptance Witnesses run `37709760488` on commit `d225f376`
  is running at snapshot time; don't claim it green before final results.
- Private Desktop Deploy Contract run `37710071238` passed on commit
  `1a60b127`, including static Compose/Caddy boundary checks and offline
  WAL backup/restore tests.
- gitleaks run `37710077217` passed on `1a60b127`.
- Private Desktop Deploy Contract run `37710277023` (commit `38e6e8c4`)
  passed real Caddy 2.8 syntax validation for the operator-alpha vhost using
  disposable credentials, resolved-Compose static security checks, and
  SQLite WAL backup/restore smoke tests. This does not prove a deployed host
  vhost, real DNS-01 TLS certificate, live browser authentication, or tailnet
  reachability.
- A passing deploy *contract* does not mean live DNS, TLS, Tailscale, Vercel,
  or a running desktop backend was tested.

## Release modes

### R0 — private operator/API alpha (not yet demonstrated)

One authorized device on the tailnet can reach the actual desktop-backed
AgilePlus API under a trusted owned product hostname. Requires host runtime,
split DNS, Tailscale ACL/service identity, Caddy certificate, database
persistence, API-key provisioning and live health/restart evidence.

### R1 — single-operator browser alpha (not yet demonstrated)

The browser uses `https://agileplus.pheno.studio` on private split DNS.
The shared Caddy edge enforces operator Basic authentication and routes:

- `/api/*` -> localhost AgilePlus backend, with server-side API key injection
  and browser Authorization header stripping;
- all other paths -> separately identified Vercel `*.vercel.app` frontend
  origin with credentials/cookies stripped.

This is not multi-user application authentication. A protected operator-only
web alpha must also demonstrate CSRF/origin protection, real product data,
acceptance receipt continuity and frontend supply-chain integrity.

### R2 — multi-user/public SaaS (not yet demonstrated)

Requires independent user identities/authorization, CSRF protection, public
ingress only when needed, abuse controls, audit/recovery and terms-compatible
hosting. Do not classify R1 as R2.

## Hard limitations / external setup

- A Vercel project named `agileplus` is visible through the connected Vercel
  account, but detailed deployment inspection returned 403 for team scope
  `koosha-paridehpours-projects`. Do not invent a `*.vercel.app` upstream
  hostname until actual authorized deployment metadata is available.
- No remote desktop login, DNS mutation, tailnet ACL change, host-level Caddy
  reload, production certificate issuance or live remote E2E occurred while
  authoring the latest deployment configuration.
- The home desktop's sleep/power behavior and OS-specific supervisor are
  release dependencies, not yet verified.
- No paid backend/VPS is authorized. Private transport defaults to Tailscale;
  Cloudflare Tunnel is optional later public ingress.

## Next executable sequence

1. Consume the latest recovery CI to determine whether the ship suite and CORS
   changes are truly native green on a concrete SHA.
2. Resolve Vercel project/deployment authorization and obtain the real frontend
   origin; otherwise keep it configurable, not guessed.
3. Validate host Caddy config with operator credentials from a local secret
   source (never repo/frontend).
4. Configure exact product split DNS and tailnet ACLs; validate TLS for the
   owned domain from an authorized remote device.
5. Run the desktop API with persistent SQLite and prove local health.
6. Test R0 remotely.
7. Test R1 same-origin frontend/API routing, authentication rejection, CSRF
   protections and acceptance persistence through reload/reboot.
8. Backup and restore actual host state; prove rollback and restart recovery.
9. Only mark a release READY when every deployed journey has execution evidence.

**Estimate, not a promise:** after CI passes and the desktop + DNS credentials
are available, R0 may need roughly 1–2 focused workdays; R1 roughly 2–5 further
focused workdays depending on Caddy/TLS/DNS/browser integration. Multi-user
SaaS is outside this operator-alpha gate.

## Work-mode continuation (2026-10-07, local execution)

Inspected source base: `38e6e8c48e44f4cec5a1f617219755c259681250`.
Recovery branch subsequently advanced to `4e9f454babcf602540bb10d37ac17d6a73b368fd`
with an evidence-only update; that update is preserved by this continuation.

- SQLite atomic acceptance: 10/10 executed and passed on Rust 1.99 stable,
  also 10/10 on the repository-pinned nightly.
- Application library: 165/165 executed and passed on stable.
- SQLite backup/restore witnesses: 3/3 executed and passed.
- Operator gateway: 4/4 executed and passed through real local Caddy 2.11.7
  with disposable loopback upstreams. Includes twenty blocked mutations across
  absent, null, foreign, suffix-spoofed and alternate-port origins; authorized
  same-origin mutations; 401 rejection; and upstream credential stripping.
- Operator gateway now rejects unsafe API requests without the exact owned
  product Origin. It strips browser-supplied API keys from frontend requests.
- Native Linux systemd template added. Its syntax was checked with a disposable
  executable substitution; actual service startup, account permissions and
  reboot recovery still require the target host.
- Expanded mounted HTTP acceptance: 7/7 passed, including missing Feature and
  stale expected governance.
- Authenticated gRPC acceptance: 6/6 passed.
- CLI implementation: 10/10 passed; shipping: 14/14 passed; validation: 12/12
  passed after repairing diagnostics and the duplicate-WP-sequence fixture.
- Migrations: 15/15 passed; API CORS policy: 3/3 passed.
- Workspace formatting and `git diff --check` passed.
- Selected-crate Clippy exposed a pre-existing denied approximate-PI literal
  in a CLI decimal-formatting test. Replacing that arbitrary value with 2.5
  made CLI all-targets Clippy pass; existing warnings remain. This is not a
  full desktop/all-workspace quality-gate claim.
- Recovery acceptance CI now also runs on PRs targeting the recovery branch.

**Confirmed dashboard blocker:** `web/index.html` loads `src/main.tsx`, whose
requests use `/api/dashboard/work-packages.json` and
`/api/dashboard/epics-stories.json`. The canonical Rust router exposes
`/api/v1/...` and does not mount those dashboard endpoints. Separately,
`src/App.tsx` substitutes seed data on API errors/empty results. Resolve the
active entrypoint and canonical API mapping, then demonstrate live data and
visible offline/error states before claiming R1. Proxy configuration alone
does not repair the dashboard contract.

Promotion saga, live desktop access, tailnet/DNS/TLS configuration, Vercel
origin authorization, deployed browser acceptance and reboot/restore drills
remain open. No live infrastructure configuration was performed here.

## Reporting rule

Every future status report must state the exact branch SHA, CI run, deployed
environment, which journeys were *executed*, and time-to-next-release. Never
count a static design, a URL, or a mock response as a usable release.
