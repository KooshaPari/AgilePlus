# AgilePlus AuthKit Readiness (2026-04-02)

## Current MCP surface

- The canonical FastMCP server lives at `agileplus-mcp/src/agileplus_mcp/server.py`. It registers the standard `features`, `governance`, and `status` modules but today starts with no authentication provider wired.
- When this server runs locally it listens on whatever `AGILEPLUS_MCP_PORT` the dev stack exposes (the AgilePlus local hook uses the randomized port map, e.g., `http://localhost:8014` during the codex-local boot). That endpoint must eventually expose the FastMCP `.well-known` metadata described in FastMCP docs (see the `/.well-known/oauth-protected-resource` and `/.well-known/oauth-authorization-server` routes referenced in `references/fastmcp.txt` around the AuthKit integration chapters).

## Desired AuthKit contract

1. **Provider configuration.** Acquire and persist the WorkOS AuthKit domain (looks like `https://<project>.authkit.app`). This becomes the `AUTHKIT_DOMAIN` env variable used by the FastMCP `AuthKitProvider`. For now we plan to pin the domain to the real provider we will execute the smoke test against before writing any AuthKit client data back to the repo.
2. **FastMCP wiring.** The FastMCP server must eventually instantiate `AuthKitProvider` (see the FastMCP docs’ `AuthKitProvider(authkit_domain="…")` snippet) and pass it as the `auth` argument to `FastMCP()`. That code should consume:
   * `AUTHKIT_DOMAIN`
   * `AUTHKIT_BASE_URL` (e.g., the local MCP base URL that `scripts/dev-up` exposes)
   * A DCR-friendly client identity (the provider’s dynamic client registration output) or static `AUTHKIT_CLIENT_ID`/`AUTHKIT_CLIENT_SECRET` if DCR cannot be used.
   * `AGILEPLUS_MCP_BASE_URL` / `AGILEPLUS_MCP_PUBLIC_URL` so that the MCP metadata including `/.well-known/oauth-protected-resource` is reachable by clients.
3. **Smoke verification.** Before wiring a login flow, run the AuthKit-ready smoke path:
   * Hit `https://<authkit-domain>/.well-known/openid-configuration` to confirm the provider metadata is reachable from the agent host.
   * Hit our local discovery endpoints (e.g., `http://localhost:${AGILEPLUS_MCP_PORT}/.well-known/oauth-protected-resource`) so we can ensure metadata forwarding is working.
   * Record whichever local HTTP routes the MCP clients will use (`/.well-known/oauth-protected-resource`, `/.well-known/oauth-authorization-server`) and ensure they mirror AuthKit.

## Non-runtime documentation uplift

- Create (or repurpose) a release-ready `docs/worklogs/2026-04-02-authkit-readiness.md` as the single implementation note for this wave. This document locks in the next auth contract so that later work can simply follow the recorded steps.
- If we later implement a dedicated `scripts/authkit-smoke` helper, reference it here and describe the two necessary metadata checks.
- Do not mutate runtime boot scripts from this worklog; keep changes restricted to documentation/spec surfaces.

## Next steps for the auth tranche

1. Declare that `agileplus-mcp` is the canonical auth surface and log the environment keys required (`AUTHKIT_DOMAIN`, `AUTHKIT_BASE_URL`, optional DCR credentials, `AGILEPLUS_MCP_BASE_URL`, and whichever `WORKOS_…` secrets the provider requires).
2. Document the smoke-check workflow described above so implementers know which endpoints to hit and what payloads they expect.
3. Capture whichever provider metadata URLs we will use as the definitive source for future integration work.
4. Once the boot evidence exists, revisit this note and append the actual provider domain + local `/.well-known` URLs so the auth surface is no longer speculative.
