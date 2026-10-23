# dependabot moderate vuln capture (dolphin, 2026-10-18T09:28:00Z)

**Verdict**: PASS
**Subagent session**: dolphin (swarm-dispatched from shrimp)
**Wall-clock**: ~120s (API fetch 2s; jq filtering 1s; cross-ref 30s)
**API call**: `gh api /repos/KooshaPari/AgilePlus/dependabot/alerts --paginate` — exit 0, 204 alerts total, 5 in state `open`, 0 with severity `moderate` at top-level (severity is nested under `security_advisory.severity` and uses GitHub's "medium" enum which is Dependabot UI's "moderate")

## Summary

Dependabot is reporting 5 vulnerabilities on `KooshaPari/AgilePlus` default branch. All 5 are currently `state=open`, `severity=medium` (= Dependabot "moderate"). **3 of the 5 are the same advisory in 3 different manifests** (mcp CVE-2026-59951 in `python/uv.lock`, `agileplus-mcp/uv.lock`, `dispatch-mcp/uv.lock`); the other 2 are distinct (postcss-selector-parser npm; glib rust).

## Enumerated vulnerabilities

| #   | alert # | ghsa_id             | cve_id              | severity | cvss  | package                 | ecosystem | version_range                                               | fix_versions       | manifest                | status                  |
| --- | ------- | ------------------- | ------------------- | -------- | ----- | ----------------------- | --------- | ----------------------------------------------------------- | ------------------ | ----------------------- | ----------------------- |
| 1   | 217     | GHSA-84m7-p3x7-pcfv | CVE-2026-59951      | medium   | 7.5   | mcp                     | pip       | `>= 1.8.0, < 1.30.0` (also `>= 2.0.0, < 2.2.0` in advisory) | 1.30.0 (and 2.2.0) | `python/uv.lock`        | OPEN — pinned at 1.28.1 |
| 2   | 216     | GHSA-84m7-p3x7-pcfv | CVE-2026-59951      | medium   | 7.5   | mcp                     | pip       | `>= 1.8.0, < 1.30.0`                                        | 1.30.0             | `dispatch-mcp/uv.lock`  | OPEN — pinned at 1.27.1 |
| 3   | 215     | GHSA-84m7-p3x7-pcfv | CVE-2026-59951      | medium   | 7.5   | mcp                     | pip       | `>= 1.8.0, < 1.30.0`                                        | 1.30.0             | `agileplus-mcp/uv.lock` | OPEN — pinned at 1.28.1 |
| 4   | 214     | GHSA-rj75-hqrm-r3gf | CVE-2026-104844     | medium   | 5.9   | postcss-selector-parser | npm       | `< 7.1.6`                                                   | 7.1.6              | `package-lock.json`     | OPEN — pinned at 6.1.4  |
| 5   | 158     | GHSA-wrw7-89jp-8q8g | (RUSTSEC-2024-0429) | medium   | 0.0\* | glib                    | rust      | `>= 0.15.0, < 0.20.0`                                       | 0.20.0             | `./Cargo.lock`          | OPEN — pinned at 0.18.5 |

\* GHSA-wrw7-89jp-8q8g CVSS is 0.0 in the API payload (no `cvss.score` field set), but severity is `medium`. This is the same root cause as RUSTSEC-2024-0429 which is **already in `deny.toml` ignore list** with rationale (tauri/tray-icon stack pins glib 0.18, fix only in 0.20.0 behind tauri PR #16170).

## Cross-reference with deny.toml

The current `deny.toml` `[advisories].ignore` array contains exactly 2 entries:

| id                | reason (verbatim from deny.toml)                                                                                                                                                                                                                                                               |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| RUSTSEC-2024-0370 | proc-macro-error unmaintained: build-time only via glib-macros 0.18 (tauri stack); glib 0.18 pins it, no fix until tauri/tray-icon bump.                                                                                                                                                       |
| RUSTSEC-2024-0429 | glib VariantStrIter: latest resolvable glib is 0.18.5; fixed in 0.20.0 behind tauri PR #16170. LINUX-ONLY path (libappindicator->tray-icon->tauri), so cargo-deny on a macOS host warns advisory-not-detected for this entry - benign, CI runs ubuntu. Re-evaluate when tauri PR #16170 lands. |

**Docket discrepancy (IMPORTANT)**: the upstream prompt claimed the docket "notes 3 of them (RUSTSEC-2026-0370/0371/0372: zvariant, zbus_names, zvariant_derive unmaintained, ignored via PR #1157 deny.toml)". After grepping all 204 alerts and reading `deny.toml`:

- **No RUSTSEC-2026-0370 / -0371 / -0372 advisory exists** for `KooshaPari/AgilePlus` (Dependabot returns zero hits across `ghsa_id`, `cve_id`, package name patterns `zvariant|zbus`).
- The actual deny.toml entries are **RUSTSEC-2024-0370 and RUSTSEC-2024-0429** (2024, not 2026).
- Neither zvariant nor zbus_names nor zvariant_derive appear in any of the 204 alerts (open or dismissed or fixed).
- The deny.toml header explicitly notes that "stale entries (RUSTSEC-2024-0436, RUSTSEC-2024-0370, ...)" references were removed in commit-chain 2026-10-13 — so PR #1157 (or whatever PR added the ignore list) likely predates that cleanup.

**Net effect**: alert #5 (glib / RUSTSEC-2024-0429) is correctly handled — the deny.toml ignore covers cargo-deny, but Dependabot independently maintains an open alert because it sees a real upgrade path. The other 4 alerts have **no deny.toml coverage** because they're either pip (mcp) or npm (postcss-selector-parser); `cargo deny` does not scan pip or npm advisories.

## Net new (not yet ignored)

These 4 alert-numbers are NOT covered by `deny.toml` and have **real fix versions** (i.e., these are not "unmaintained" ignores; they're actionable bumps):

| alert # | advisory                                            | fix    | action required                                                                                                                                  |
| ------- | --------------------------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| 217     | GHSA-84m7-p3x7-pcfv (mcp 1.28.1)                    | 1.30.0 | bump `python/uv.lock` mcp to >=1.30.0                                                                                                            |
| 216     | GHSA-84m7-p3x7-pcfv (mcp 1.27.1)                    | 1.30.0 | bump `dispatch-mcp/uv.lock` mcp to >=1.30.0                                                                                                      |
| 215     | GHSA-84m7-p3x7-pcfv (mcp 1.28.1)                    | 1.30.0 | bump `agileplus-mcp/uv.lock` mcp to >=1.30.0                                                                                                     |
| 214     | GHSA-rj75-hqrm-r3gf (postcss-selector-parser 6.1.4) | 7.1.6  | bump `package-lock.json` postcss-selector-parser to >=7.1.6 (note: a major-version jump; check for breaking changes in PostCSS selector parsing) |

Alert #158 (glib / RUSTSEC-2024-0429) is documented-ignored but Dependabot still flags it because Dependabot ignores `deny.toml`. It will require either:

- merging tauri PR #16170 (the upstream gtk 0.19 bump), or
- dismissing the alert manually with reason `tolerable_risk` (matches the pattern used for the 3 previously-dismissed mcp HIGH alerts #141/144/146).

## Local manifest evidence

```
python/uv.lock          → mcp 1.28.1 (vulnerable; need >= 1.30.0)
agileplus-mcp/uv.lock   → mcp 1.28.1 (vulnerable; need >= 1.30.0)
dispatch-mcp/uv.lock    → mcp 1.27.1 (vulnerable; need >= 1.30.0)
package-lock.json       → postcss-selector-parser 6.1.4 (vulnerable; need >= 7.1.6)
./Cargo.lock            → glib 0.18.5 (vulnerable; need >= 0.20.0, blocked by tauri PR #16170)
```

## Recommendation

The 5-alert count is real and accurate: 4 are actionable, 1 is documented-ignored but Dependabot-uncovers. **Operator action**: dispatch a single integration subagent to (1) bump `mcp` to >=1.30.0 across all three Python lockfiles and run `ruff check python/ dispatch-mcp/ agileplus-mcp/` + smoke-test each MCP server entry point; (2) bump `postcss-selector-parser` to >=7.1.6 in `package-lock.json` and re-run any frontend builds/tests (PostCSS 7.x is a major version — expect selector-grammar tightening). After those 4 land, alert #158 (glib) remains open until tauri PR #16170 lands or is manually dismissed with `tolerable_risk` (matching the previous mcp HIGH dismissal pattern). The docket's "RUSTSEC-2026-0370/0371/0372 zvariant/zbus_names/zvariant_derive" reference appears to be stale or misstated and should be corrected in a follow-up docket update — the real deny.toml entries are RUSTSEC-2024-0370 and RUSTSEC-2024-0429.

## Files

- This report: `/tmp/dependabot_2026-10-18-result.md`
- Full gh API dump (204 alerts): `/tmp/dependabot_alerts.json` (1,263,929 bytes)
- Orchestrator trace log: `/tmp/dependabot_2026-10-18.log`
