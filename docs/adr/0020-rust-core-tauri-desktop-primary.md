# ADR-020: Rust Core + Tauri Desktop as Primary Interface

## Status
Proposed (supersedes prior version recommending Electrobun)

## Context
AgilePlus has multiple interfaces competing for primacy:
- CLI (`./target/release/agileplus`) — current primary, works today
- MCP server (`agileplus-mcp`) — enables Codex/AI tooling, gRPC core currently blocked by platform deps
- Desktop client (`desktop/`, currently Electrobun) — step-1 exists, offline-first, incomplete
- REST API (`agileplus-api`) — requires external services, should be optional

User has confirmed: "I want to use the app and tray!!!!" — tray + desktop app are user-facing requirements.

### Electrobun vs alternatives (research-driven)

After comparing 7 desktop stacks (Tauri, Electrobun, Electron, CEF, Qt, Wails, Native Swift/WinUI/GTK), Tauri wins:

| Stack | Tray | SQLite | Binary | Memory | Rust Fit | Maturity | License |
|-------|------|--------|--------|--------|----------|----------|---------|
| **Tauri** | ✅ | ✅✅ Native | ~5-10MB | ~30-80MB | ✅✅ | ✅ | MIT |
| Electrobun | ✅ | ⚠️ IPC | ~80-150MB | ~100-300MB | ⚠️ | ⚠️ Early | MIT |
| Electron | ✅ | ⚠️ | ~80-150MB | ~100-300MB | ⚠️ | ✅✅ | MIT |
| CEF | ⚠️ | ✅ | ~80-150MB | ~100-300MB | ⚠️ | ✅✅ | BSD |
| Qt | ✅ | ✅ | ~20-50MB | ~40-100MB | ⚠️ | ✅✅ | LGPL |
| Wails (Go) | ⚠️ | ⚠️ | ~5-10MB | ~40-80MB | ❌ | ⚠️ | MIT |
| Native | ✅✅ | ✅ | ~5-20MB | ~20-50MB | ⚠️ | ✅✅ | - |

**Key reasons Tauri wins for AgilePlus:**
1. **Native SQLite** — `rusqlite` compiles into the Rust core, zero IPC, reuses `crates/agileplus-sqlite` directly
2. **Reuses Rust core** — Tauri commands are Rust functions calling into the `agileplus` crate
3. **Smallest distribution** — ~5-10MB vs ~80-150MB for Electron-family
4. **Lowest memory** — ~30-80MB vs ~100-300MB
5. **Fastest startup** — Native WebView loads in <500ms
6. **Battle-tested** — Used by Linear, Vercel, Cloudflare, 1Password
7. **Tray support** — `tauri-plugin-tray` covers the requirement
8. **Strong security** — Allowlist + CSP model

## Decision
1. **CLI** remains the **semantic process engine** — the "git of spec-driven dev"
2. **Desktop (Tauri)** is the **primary user interface** — tray + app window
3. **MCP** is the **agent-facing protocol** — how AI tools interact with AgilePlus
4. **API** is **optional/standalone** — if a REST layer is needed, it's a separate deployable

### Interface responsibilities:
| Interface | Role | Protocol |
|-----------|------|----------|
| CLI (`agileplus`) | Semantic process, scripting, CI | Rust binary, `--db` flag |
| Desktop (Tauri) | User-facing app, tray, UI | Rust + WebView, local `.agileplus/` |
| MCP (`agileplus-mcp`) | Agent-facing, tool calls | stdio/SSE, MCP protocol |
| API (optional) | Remote access, server mode | HTTP REST, standalone deployable |

### Migration from Electrobun:
- Move existing `desktop/` (Electrobun) to `desktop-bun/` for reference
- Scaffold Tauri app in `desktop/`
- Wire `crates/agileplus-sqlite` into Tauri commands
- Port UI components to React/Vue/Svelte (Tauri-compatible)
- Add `tauri-plugin-tray`, `tauri-plugin-shell`, `tauri-plugin-sql`

## Consequences

### Positive
- ✅ Reuses Rust core (no FFI, no IPC, no language bridge)
- ✅ Native SQLite access via `rusqlite`
- ✅ Smallest distribution (~5-10MB)
- ✅ Lowest memory (~30-80MB)
- ✅ Fastest startup (<500ms)
- ✅ Strong security model
- ✅ Battle-tested in production by major companies

### Negative
- ❌ Migration cost from Electrobun (1-2 weeks)
- ❌ WebView API quirks per OS (less unified than Electron)
- ❌ Smaller ecosystem than Electron
- ❌ Some advanced features need OS-specific code

## Related
- Research: `/docs/research/desktop-client-alternatives-2026-09-03.md`
- `crates/agileplus-sqlite/` — the SQLite adapter, reused via Tauri
- ADR-018: Remove Platform Services
- ADR-019: Local-First SQLite State Only
