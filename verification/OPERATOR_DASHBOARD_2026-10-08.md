# Operator dashboard acceptance integration

The active web entry point now renders canonical `/api/v1` feature, work-package, governance and acceptance data. Backend authentication failures, empty data and invalid HTML responses remain visible. No browser API key or sample-data fallback is used by this entry point.

Acceptance requests persist their identity and expected governance version before POST. An uncertain response keeps the request for an identical retry. The authenticated receipt lookup reads the immutable SQLite receipt without re-evaluation or state mutation, including after reopening the database. Receipt presentation describes a historical development decision, not current product satisfaction or Git promotion.

## Verification

- API HTTP acceptance suite: 10 passed, including unauthorized reads, missing receipts, receipt persistence after database reopen and sanitized storage failures.
- Operator UI suite: 9 passed, including canonical paths, lost-response identity reuse, remount recovery, distinct rejection/failure responses and HTML fallback rejection. These are mocked-fetch component tests, not browser end-to-end evidence.
- Active entry point typecheck passed with `npm run typecheck:operator`; production Vite build passed.
- Disposable real API fixture compiled. It requires explicit test database and API key environment variables and binds loopback only.
- Full legacy frontend typecheck remains blocked by missing translation catalogs and existing browser-incompatible globals.
- Browser end-to-end execution was blocked by unavailable Chromium downloads. No live tailnet, domain, TLS, desktop or Vercel deployment was exercised.

## Release boundaries

Recovery acceptance, deployment-contract and E2E workflows passed on PR #1095 before this dashboard change. Broad security workflows still report dependency advisories, including rustls-webpki and legacy GLib dependencies; these gates were not weakened. A precise Gitleaks fingerprint suppresses one verified prose false positive only.

Durable Git promotion recovery and live operator access remain release work. This change supplies development acceptance and historical receipt recovery; it does not claim promotion completion.
