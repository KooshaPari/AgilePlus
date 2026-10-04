# Next deployable release gate — AgilePlus

Date: 2026-10-04

## Target topology

- Dashboard frontend: Vercel Hobby after production browser auth/API routing is closed.
- Rust API + persistent SQLite: owner's desktop.
- Public ingress: Cloudflare Tunnel to Caddy to agileplus-api.
- Incremental hosting spend: $0.

## Release definition

The next cloud release requires:

1. canonical atomic acceptance Rust tests execute and pass;
2. real-SQLite HTTP acceptance tests execute and pass;
3. desktop API stack builds and restarts cleanly;
4. Cloudflare Tunnel exposes health/readiness;
5. browser credentials do not contain the operator API key;
6. production browser auth/same-origin proxy contract is implemented;
7. dashboard production API-origin contract is implemented;
8. one browser journey reads real backend state;
9. one browser acceptance journey reaches the canonical atomic endpoint;
10. SQLite backup/restore and desktop reboot recovery are tested.

## Time-to-release estimate

Assuming focused work and no large dashboard regression:

- remote operator/API alpha (no public dashboard): **1–2 working days**;
- first usable authenticated SaaS-style dashboard alpha: **4–7 working days**;
- stronger alpha with acceptance UI + worker/recovery visibility: **8–15 working days**.

The main uncertainty is dashboard/browser authentication and integration, not
the desktop hosting mechanism. The old CLI-only self-host stack has been
superseded by the real Rust API deployment target.
