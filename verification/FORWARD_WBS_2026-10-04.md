# AgilePlus forward work breakdown

Status: canonical recovery WBS as of 2026-10-04.

This document is the operational continuation point for future ChatGPT chats,
ChatGPT Work sessions, coding agents and human contributors. It supplements the
semantic authority chain; it does not override the accepted mature product
contract.

## Release target

The next meaningful release is a remotely usable, zero-paid-service,
SaaS-style private/operator alpha.

Target topology:

```
Vercel Hobby dashboard
  -> product domain
  -> tailnet-aware/private API naming
  -> Tailscale organizational network
  -> stable service identity where practical
  -> one host-level Caddy
  -> owner's desktop
  -> agileplus-api
  -> persistent SQLite
```

Cloudflare Tunnel is optional public ingress later, not the private network
foundation.

### Current engineering ETA

Assuming focused execution and no newly discovered major dashboard regression:

- native/operator API alpha: 1–2 focused working days;
- first usable authenticated browser alpha: 4–7 focused working days;
- stronger alpha with acceptance UI and worker/recovery visibility: 8–15 focused working days.

These are estimates, not completion percentages.

## P0 — make canonical atomic acceptance natively green

Do not broaden product scope until this gate has authoritative execution.

1. Run Rust formatting, Clippy, compile and workspace tests on the canonical head.
2. Execute canonical SQLite atomic acceptance witnesses:
   - complete commit;
   - exact replay;
   - conflicting request reuse;
   - invalid existing audit chain;
   - forced late receipt failure rollback.
3. Execute real-SQLite HTTP acceptance witnesses:
   - auth required;
   - commit;
   - exact replay;
   - conflict;
   - validation/precondition failure is 422;
   - failure leaves terminal state unchanged.
4. Preserve the corrected CI truth rule:
   test not executed != test passed.
5. Separate unrelated repository debt from this gate:
   Python lint, Cargo Deny, coverage, security and release failures remain
   tracked but cannot be used either to hide or fabricate acceptance evidence.

Exit criterion: one concrete commit has native evidence that Rust tests
actually executed and the atomic acceptance suite passed.

## P1 — finish HTTP acceptance as a production contract

1. Add OpenAPI contract for the dedicated acceptance endpoint.
2. Pin request/response/idempotency semantics.
3. Prove:
   - missing feature -> 404;
   - stale governance version -> 409;
   - bad correctness/governance -> 422;
   - storage failure -> sanitized 500;
   - exact replay returns the original receipt;
   - generic feature transition cannot award Validated or Shipped.
4. Ensure acceptance receipt identity can be retrieved after restart.
5. Add restart-backed database integration witness.

Exit criterion: HTTP is a projection of the application acceptance use case,
not a parallel lifecycle implementation.

## P2 — gRPC validation parity

Generic gRPC lifecycle dispatch currently fails closed. Replace only validate
with the canonical path:

```
RPC
  -> resolve feature
  -> durable request identity
  -> AcceptFeatureCommand
  -> accept_feature
  -> AtomicAcceptancePort
  -> AcceptanceOutcome
```

Rules:

- no gRPC-local governance evaluator;
- no gRPC-local candidate evaluator;
- no direct state mutation;
- request identity must be stable/idempotent;
- if the protobuf cannot represent the required command, version the API.

Do not implement ship through the same shortcut.

## P3 — worker replacement and restart recovery

Prove the durable work model rather than merely the happy path.

Required journey:

```
Assignment A
 -> Attempt A / worker A
 -> lease expiry or process death
 -> Attempt A retained as Expired/Failed
 -> worker B claims same Assignment
 -> Attempt B
 -> exact Candidate B
 -> independent Evaluation B
 -> atomic acceptance
```

Failure injection/restart points:

- after Assignment creation;
- after claim acquisition;
- during Attempt;
- after candidate creation;
- after Evaluation;
- before acceptance;
- during acceptance;
- after committed acceptance but before caller sees response.

After restart the system must distinguish safe retry, durable history and work
that must not be repeated.

## P4 — promotion/ship application use case

Extract terminal promotion from CLI into a canonical application operation.

Required checks:

1. Feature is Validated.
2. Durable acceptance receipt exists.
3. Accepted candidates still map to exact source commits.
4. Re-read source immediately before merge.
5. Any branch drift fails closed.
6. Merge the accepted commit, not merely the current branch tip.
7. Record resulting merge/release identity.
8. Persist promotion audit/event/receipt.
9. Clean worktrees/resources only after committed promotion.

CLI/gRPC/HTTP can later project this same operation.

## P5 — tailnet-first deployment foundation

Canonical doctrine: docs/architecture/NETWORK_DEPLOYMENT_DOCTRINE.md

### Organizational network

- Porkbun remains registrar.
- Cloudflare remains public authoritative DNS.
- Tailscale is the private organizational network.
- Use split DNS/custom internal resolver for owned service names.
- Prefer stable Tailscale Service identity over physical host identity.
- Product URLs must not expose machine names as their contract.

### Host edge

Move toward one host-level edge process:

```
tailscaled
caddy
internal DNS resolver if required
agileplus-api
workers
stateful dependencies only when justified
```

Do not create project-Caddy -> global-Caddy -> Traefik chains without a
demonstrated requirement.

### Private naming

Candidate service names:

- agileplus.pheno.studio — browser/product identity;
- api.agileplus.pheno.studio — HTTP API;
- grpc.agileplus.pheno.studio — gRPC;
- mcp.agileplus.pheno.studio — MCP;
- infra.pheno.studio subtree — organizational infrastructure.

Exact names are configurable; the abstraction boundary is mandatory.

### TLS

Owned custom hostnames require certificates for those hostnames. For
tailnet-only targets use DNS-01 where required rather than leaking `.ts.net`
names into product UX.

## P6 — browser authentication and dashboard production path

This is the main uncertainty between API alpha and SaaS-style alpha.

Never embed `AGILEPLUS_API_KEY` in Vite/public JavaScript.

Preferred order of investigation:

1. tailnet identity-aware path for private/operator users;
2. server-side BFF/proxy holding backend credential;
3. proper application user auth for broader users.

Tailscale Serve can attach identity headers for tailnet traffic, but Serve's
HTTPS names are restricted to the tailnet domain. If custom product domains
terminate directly at Caddy, use another identity-aware mechanism rather than
pretending DNS itself authenticates the user.

Dashboard production work:

- explicit production API-origin contract;
- no dev-only Vite proxy assumptions;
- browser smoke against real API;
- acceptance UI using canonical endpoint;
- durable receipt presentation;
- error states for 409/422/500;
- no secret in bundle;
- reconnect/offline behavior.

## P7 — zero-cost desktop operations

The desktop is the stateful host, not a development-only accident.

Required operational gates:

- native/process-supervisor deployment path;
- automatic startup on reboot;
- Tailscale startup and route/service advertisement;
- Caddy startup/reload;
- API health/readiness;
- persistent SQLite location;
- periodic backup;
- restore drill;
- disk-space monitoring;
- log rotation;
- update/rollback procedure;
- desktop sleep/power-loss expectations explicitly documented.

Compose remains useful as bootstrap/test packaging but should not be required
for the long-term host.

## P8 — first deployed private/operator alpha

A deployment is not merely a reachable URL.

Minimum journey:

1. open dashboard through product domain;
2. reach real desktop-hosted API through tailnet architecture;
3. list/read real project state;
4. inspect one Feature and work package;
5. perform or observe one canonical acceptance operation;
6. reload/reconnect and see durable receipt/state;
7. restart backend and observe persistence;
8. prove no stub/fallback data is presented as real state.

Exit criterion: remotely usable from an authorized tailnet device with zero
incremental paid hosting.

## P9 — public ingress later

Only when a non-tailnet user must connect.

Add exactly one public ingress layer in front of the same host edge. A
Cloudflare Tunnel is an acceptable option then. Do not restructure the backend
or product naming merely to add public reachability.

Public access requires a real user-auth model and substantially stronger abuse,
rate-limit and availability controls than the private alpha.

## P10 — Tracera federation

After AgilePlus's execution kernel is trusted:

```
AcceptedWork receipt
 -> Tracera Observation
 -> applicability assessment
 -> CurrentValid / Suspect / Stale / Invalid / Unknown
```

Hard invariant:

AgilePlus AcceptedWork != Tracera Satisfied.

First federation trial should use one concrete product obligation and one exact
candidate, then invalidate a dependency and prove Tracera changes applicability
without rewriting AgilePlus history.

## Completion discipline

No single percentage is authoritative. Future status reports must separate:

- mature semantic coverage;
- implementation realization;
- native evidence;
- user journey closure;
- deployment readiness;
- operational readiness;
- external validation;
- uncertainty/regression state.

Every status report must include time-to-next-deployed/installable release.
