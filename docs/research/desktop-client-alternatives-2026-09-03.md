# Desktop Client Alternatives Research

**Date:** 2026-09-03  
**Scope:** All viable desktop client stacks compared for AgilePlus  
**Requirements:** tray support, local SQLite access, cross-platform (macOS/Windows/Linux), offline-first, fast startup  

---

## Requirement Matrix

| Requirement | Why it matters for AgilePlus |
|-------------|------------------------------|
| Tray (system notification area) | User explicitly asked: "I want to use the app and tray" |
| Local SQLite access | Must read/write `.agileplus/agileplus.db` |
| Cross-platform | macOS (primary), Windows, Linux |
| Offline-first | Core architecture principle |
| Fast startup | CLI-first culture, no 5-second launcher |
| Resource efficient | Local dev box has limits |
| Reasonable binary size | Distribution matters |
| Existing Rust integration | Can embed rusqlite via FFI |

---

## Stack Comparison

### 1. Tauri (Rust + WebView)

**Architecture:** Rust core, OS-native WebView (WKWebView on macOS, WebView2 on Windows, WebKitGTK on Linux)

| Attribute | Value | Notes |
|-----------|-------|-------|
| **Tray support** | ✅ Yes | `tauri-plugin-tray` |
| **SQLite access** | ✅ Excellent | Pure Rust via `rusqlite`, no IPC |
| **Cross-platform** | ✅ All 3 | Native WebView per OS |
| **Offline-first** | ✅ Yes | Local assets, no network required |
| **Startup** | ⚡ <500ms | Native WebView is instant |
| **Binary size** | ✅ ~5-10MB | WebView is OS-provided, not bundled |
| **Memory** | ✅ ~30-80MB | Much less than Electron |
| **Security** | ✅ Strong | Rust + Tauri's allowlist model |
| **Maturity** | ✅ Stable | 1.0 released, used by Linear, Vercel, etc. |
| **Rust core** | ✅ Native | No FFI needed |
| **UI language** | TS/JS/HTML/CSS or any web | React, Svelte, Vue, etc. |
| **License** | MIT/Apache-2.0 | Permissive |

**Strengths:**
- Smallest binary size of all web-based stacks
- Lowest memory footprint
- Native SQLite integration (no IPC overhead)
- Rust core = same language as AgilePlus core
- Used by Linear, Vercel, Cloudflare, 1Password

**Weaknesses:**
- WebView API quirks per OS (less unified than Electron)
- Smaller ecosystem than Electron
- Some advanced features require OS-specific code

### 2. Electrobun (Electron + Bun)

**Architecture:** Electron shell + Bun runtime

| Attribute | Value | Notes |
|-----------|-------|-------|
| **Tray support** | ✅ Yes | Electron Tray API |
| **SQLite access** | ⚠️ Via Node binding | `better-sqlite3` or `bun:sqlite` |
| **Cross-platform** | ✅ All 3 | Electron |
| **Offline-first** | ✅ Yes | Bundled assets |
| **Startup** | ⚠️ 1-3s | Electron cold start |
| **Binary size** | ❌ 80-150MB | Full Chromium bundled |
| **Memory** | ❌ 100-300MB | Chromium overhead |
| **Security** | ⚠️ Electron sandbox | Needs explicit config |
| **Maturity** | ⚠️ Early | Young project, breaking changes |
| **Rust core** | ⚠️ FFI only | Sidecar process or native module |
| **UI language** | TS/JS/HTML | Bun + Node ecosystem |
| **License** | MIT | Permissive |

**Strengths:**
- Bun runtime is fast
- Already partially implemented (commit 5062716a)
- TS/JS ecosystem

**Weaknesses:**
- Large binary (80-150MB)
- High memory (100-300MB)
- Single-maintainer risk (very young project)
- Not native to Rust core
- Electron-style security pitfalls

### 3. Electron (vanilla)

| Attribute | Value | Notes |
|-----------|-------|-------|
| **Tray** | ✅ | Native |
| **SQLite** | ⚠️ Node binding | |
| **Binary** | ❌ 80-150MB | |
| **Memory** | ❌ 100-300MB | |
| **Maturity** | ✅✅ Stable | Used by VSCode, Slack, Discord |
| **Rust FFI** | ⚠️ Sidecar | |
| **License** | MIT | |

**Verdict:** Same downsides as Electrobun, more mature. But still large.

### 4. Native Swift (macOS) / WinUI (Windows) / GTK (Linux)

| Attribute | Value | Notes |
|-----------|-------|-------|
| **Tray** | ✅ | OS-native |
| **SQLite** | ✅ | System SQLite on macOS, libsqlite3 elsewhere |
| **Binary** | ✅ 5-20MB | Native compilation |
| **Memory** | ✅ 20-50MB | Lowest possible |
| **Startup** | ⚡ <200ms | |
| **Cross-platform** | ❌ Three codebases | Swift + C#/.NET + GTK/Vala |
| **Maturity** | ✅✅ Stable | |
| **Rust core** | ❌ FFI only | Would call into Rust binary |
| **License** | Platform-dependent | |

**Strengths:**
- Best possible performance and UX on each platform
- Smallest binaries
- OS-native look and feel

**Weaknesses:**
- Three separate codebases (Swift, C#, GTK)
- Highest maintenance cost
- Slowest to ship features across all platforms
- Doesn't reuse Rust core except via CLI

### 5. Qt (C++/QML)

| Attribute | Value | Notes |
|-----------|-------|-------|
| **Tray** | ✅ | QSystemTrayIcon |
| **SQLite** | ✅ | QtSql with SQLite driver |
| **Binary** | ✅ 20-50MB | Static linking possible |
| **Memory** | ✅ 40-100MB | |
| **Startup** | ⚡ ~500ms | |
| **Cross-platform** | ✅ All 3 | Same codebase |
| **Maturity** | ✅✅ Stable | Used by KDE, VLC, etc. |
| **Rust core** | ⚠️ FFI | C-ABI only |
| **License** | LGPL/GPL | Restrictive |

**Strengths:**
- Mature, stable, cross-platform
- Good performance
- Native widgets available

**Weaknesses:**
- LGPL license concern
- QML is not as mainstream as web
- C++ build complexity
- Doesn't reuse Rust core directly

### 6. Wails (Go + WebView)

| Attribute | Value | Notes |
|-----------|-------|-------|
| **Tray** | ⚠️ Limited | Plugin-based |
| **SQLite** | ⚠️ Via Go binding | |
| **Binary** | ✅ 5-10MB | |
| **Memory** | ✅ 40-80MB | |
| **Startup** | ⚡ ~500ms | |
| **Cross-platform** | ✅ All 3 | |
| **Maturity** | ⚠️ Mid | Newer than Tauri |
| **Rust core** | ❌ FFI only | Go would call Rust via C-ABI |
| **UI language** | TS/JS | |
| **License** | MIT | |

**Verdict:** Similar to Tauri but Go instead of Rust. Worse fit because AgilePlus core is Rust.

### 7. CEF (Chromium Embedded Framework)

| Attribute | Value | Notes |
|-----------|-------|-------|
| **Tray** | ⚠️ Manual | Not built-in |
| **SQLite** | ✅ Via native binding | |
| **Binary** | ❌ 80-150MB | |
| **Memory** | ❌ 100-300MB | |
| **Startup** | ❌ 1-3s | |
| **Cross-platform** | ✅ All 3 | |
| **Maturity** | ✅✅ Stable | Used by Spotify, Discord |
| **Rust core** | ⚠️ C++ binding | |
| **UI language** | TS/JS | |
| **License** | BSD | Permissive |

**Verdict:** Same downsides as Electron, more control, more complexity. Not recommended.

---

## Cross-Platform Stack Decision Matrix

| Stack | Tray | SQLite | Binary | Memory | Rust Fit | Maturity | License | Score |
|-------|------|--------|--------|--------|----------|----------|---------|-------|
| **Tauri** | ✅ | ✅✅ | ✅✅ | ✅✅ | ✅✅ | ✅ | MIT | **9/10** |
| **Electron** | ✅ | ✅ | ❌ | ❌ | ⚠️ | ✅✅ | MIT | 5/10 |
| **Electrobun** | ✅ | ✅ | ❌ | ❌ | ⚠️ | ⚠️ | MIT | 5/10 |
| **CEF** | ⚠️ | ✅ | ❌ | ❌ | ⚠️ | ✅✅ | BSD | 4/10 |
| **Qt** | ✅ | ✅ | ✅ | ✅ | ⚠️ | ✅✅ | LGPL | 6/10 |
| **Wails (Go)** | ⚠️ | ✅ | ✅ | ✅ | ❌ | ⚠️ | MIT | 5/10 |
| **Native (Swift/WinUI/GTK)** | ✅ | ✅ | ✅✅ | ✅✅ | ⚠️ | ✅✅ | Platform | 5/10* |

*Native is best per-platform but requires 3x codebases. Net TCO is higher.

---

## Recommendation

**Switch from Electrobun to Tauri.**

### Why Tauri wins for AgilePlus:

1. **Native SQLite access** — `rusqlite` compiles directly into the Rust core. No IPC, no FFI, no protocol bridge. The desktop app and CLI share the exact same SQLite adapter (`crates/agileplus-sqlite`).

2. **Reuses Rust core** — Tauri commands are Rust functions. We can call `agileplus` crate functions directly from the UI layer with zero serialization overhead.

3. **Smallest distribution** — ~5-10MB vs ~80-150MB for Electron/Electrobun/CEF. Matches the "client-side, lightweight" principle.

4. **Lowest memory** — ~30-80MB vs ~100-300MB. Fits the "local dev tool" UX expectation.

5. **Fastest startup** — Native WebView loads in <500ms. Matches CLI-first culture.

6. **Battle-tested** — Used by Linear, Vercel, Cloudflare, 1Password, etc. for production apps.

7. **Tray support** — `tauri-plugin-tray` covers the tray requirement.

8. **Security model** — Tauri's allowlist + CSP model is more secure than Electron's default-allow.

### Migration cost from Electrobun:

- `desktop/src/` (Electrobun TS/Bun code) → `desktop/src-tauri/` (Rust) + `desktop/src/` (TS web frontend)
- Reuse existing UI components if they were vanilla TS/React
- Trade Bun runtime for WebView JS APIs (compatible)
- Add `tauri-plugin-tray`, `tauri-plugin-shell`, `tauri-plugin-sql`

### Effort estimate:

- Rewrite desktop client: 1-2 weeks (medium effort)
- Better long-term: yes (smaller, faster, secure, integrated)

---

## SQLite Alternatives In-Depth

### Current choice: SQLite (via `rusqlite`, bundled)

### Alternatives considered:

| DB | Storage | FTS | Vector | Sync | Multi-proc | License | Fit |
|----|---------|-----|--------|------|------------|---------|-----|
| **SQLite (current)** | Single file | ✅ Built-in | ⚠️ via extension | ❌ | ⚠️ WAL | Public Domain | ✅ Best fit |
| **LanceDB** | Columnar (files) | ❌ | ✅ Native | ❌ | ❌ | Apache-2.0 | ❌ Vector-only, wrong fit |
| **DuckDB** | Single file | ✅ Strong | ✅ Native | ❌ | ⚠️ | MIT | ✅ For analytics, not OLTP |
| **Beads (beads-rs)** | Single file | ❌ | ❌ | ❌ | ❌ | MIT | ⚠️ Niche, narrow focus |
| **LMDB** | Single file (mmap) | ❌ | ❌ | ❌ | ⚠️ | OpenLDAP | ⚠️ Read-optimized, not great for writes |
| **RocksDB** | Directory | ❌ | ❌ | ❌ | ✅ | GPL/Apache | ❌ Heavy, server-grade |
| **Redb** | Single file | ❌ | ❌ | ❌ | ❌ | MPL-2.0 | ⚠️ New, unproven |
| **Sled** | Single file | ❌ | ❌ | ❌ | ⚠️ | MIT | ⚠️ Unstable, single-maintainer |
| **PostgreSQL** | Server | ✅ | ✅ via pgvector | ✅ | ✅ | PostgreSQL | ❌ Server, contradicts local-first |

### SQLite in detail:

**Why SQLite is correct for AgilePlus:**

1. **Single file = single source of truth** — `.agileplus/agileplus.db` is portable, git-trackable (excluding WAL files), easy to backup.

2. **Bundled via `rusqlite`** — No external dependency, no `libsqlite3` requirement. Compiles into the binary.

3. **WAL mode** — Supports concurrent reads with one writer. Perfect for CLI + MCP + Desktop reading the same DB.

4. **Foreign keys + constraints** — Enforce schema integrity at the DB level.

5. **Migrations** — Already proven in `crates/agileplus-sqlite/src/migrations/` (26 migrations).

6. **JSON support** — JSON1 extension for structured data.

7. **FTS5** — Full-text search built-in.

8. **Mature, battle-tested** — Used by every browser, mobile OS, language runtime.

9. **Public domain** — No license concerns.

10. **Tooling** — `sqlite3` CLI, DB Browser for SQLite, many GUI tools.

### LanceDB comparison:

- **Designed for:** AI/ML vector search
- **Data model:** Columnar, Arrow-based
- **Strengths:** Native vector embeddings, fast similarity search
- **Weaknesses:** Not designed for OLTP (many small writes), no FTS, no foreign keys, immature for general use
- **Fit for AgilePlus:** ❌ Wrong tool. AgilePlus doesn't need vector search (yet). Adding LanceDB would require two DBs.

### DuckDB comparison:

- **Designed for:** OLAP (analytics)
- **Strengths:** Columnar, fast aggregations, vector support
- **Weaknesses:** Single-writer, slower for many small writes
- **Fit for AgilePlus:** ⚠️ Could complement SQLite for analytics, but not a replacement

### Beads comparison:

- **Designed for:** Git-backed issue tracking
- **Strengths:** Git-native, simple
- **Weaknesses:** Niche focus, no relations, no governance, no audit chain
- **Fit for AgilePlus:** ❌ Too narrow, doesn't support governance/traceability

### LMDB/Sled/RocksDB comparison:

- All are key-value stores, not relational
- AgilePlus needs relational features (FK, JOIN, transactions across tables)
- None support FTS or vector natively (without extensions)

---

## Final Recommendation Summary

### Desktop client: **Tauri**

Switch from Electrobun to Tauri. Lower memory, smaller binary, native SQLite access, reuses Rust core.

### State storage: **SQLite (keep)**

No change. SQLite is the optimal choice for client-side, single-file, relational storage. LanceDB/DuckDB/Beads are wrong tools for this use case. SQLite is mature, bundled, fast, and portable.

---

## Migration Plan (if approved)

1. Create branch `fix/desktop-tauri-migration-20260903`
2. Move `desktop/` to `desktop-bun/` (preserve for reference)
3. Scaffold Tauri app in `desktop/`
4. Wire `crates/agileplus-sqlite` into Tauri commands
5. Port UI from React/TS (if present) to Tauri-compatible web stack
6. Add `tauri-plugin-tray` for tray support
7. Add `tauri-plugin-sql` for typed SQLite access
8. Build, package, verify on macOS/Windows/Linux
9. Update docs

**Estimated effort:** 1-2 weeks
