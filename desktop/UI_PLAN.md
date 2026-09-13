# AgilePlus UI Migration Plan: Electrobun → Tauri

> Migration of the desktop-bun (Electrobun) UI to the Tauri 2.x desktop app.
> See also: `TRAY_CLI.md` (tray integration), `ICONS_AND_PACKAGING.md` (icons/distribution).

---

## 1. Current State Analysis

### 1.1 desktop-bun/ (Electrobun — Source)

The existing desktop app is a step-1 Electrobun shell with offline-first local repo browsing:

| File | Lines | Purpose |
|------|-------|---------|
| `src/index.ts` | 26 | Main process: creates BrowserWindow, wires RPC |
| `src/views/index.ts` | 59 | RPC schema definition + handler (getRepoState) |
| `src/views/main.ts` | 120 | Renderer: tab switching, list rendering, bootstrap |
| `src/views/main.html` | 39 | Shell HTML: header, 3 tabs (Specs/ADRs/Traces), footer |
| `src/views/main.css` | 90 | Dark theme CSS with CSS variables |
| `src/repo-bridge.ts` | 170+ | Filesystem reads + CLI shell-out for feature state |
| `src/cli.ts` | 47 | CLI wrapper: `agileplus <args>` via child_process |
| `src/paths.ts` | 55 | Path resolution: repoRoot → specsDir, adrDir, tracesDir |
| `electrobun.config.ts` | — | Electrobun build config |
| `package.json` | — | Dependencies: electrobun |

**Key patterns from desktop-bun:**
- **RPC bridge**: Renderer calls `getRepoState` → main process reads filesystem → returns specs/adrs/traces
- **CLI shell-out**: Write operations (specify, advance) spawn `agileplus` CLI
- **Tab-based UI**: 3 tabs (Specs, ADRs, Traces) with simple list rendering
- **Dark theme**: CSS custom properties for theming
- **No framework**: Vanilla JS DOM manipulation

### 1.2 desktop/ (Tauri — Target)

| File | Lines | Purpose |
|------|-------|---------|
| `src-tauri/src/main.rs` | 5 | Entry point |
| `src-tauri/src/lib.rs` | 55 | Tauri setup: tray icon, menu, window, commands |
| `src-tauri/src/commands.rs` | 170 | 5 Tauri commands (list_features, get_feature, create_feature, update_feature_state, get_dashboard_stats) |
| `src-tauri/src/db.rs` | 48 | SQLite schema: features, work_packages, evidence |
| `src/index.html` | 154 | Placeholder dashboard (static HTML, no invoke calls wired) |
| `src-tauri/tauri.conf.json` | — | Tauri config |

**What exists vs what's needed:**

| Capability | desktop-bun | desktop/ Tauri | Gap |
|------------|-------------|----------------|-----|
| Feature list | `agileplus list --json` via CLI | `list_features` command | ✅ Command exists, needs UI |
| Dashboard stats | None | `get_dashboard_stats` command | ✅ Command exists, needs UI |
| Create feature | `agileplus specify` via CLI | `create_feature` command | ✅ Command exists, needs UI |
| Update state | `agileplus status` via CLI | `update_feature_state` command | ✅ Command exists, needs UI |
| ADR browsing | Filesystem read | Not implemented | ❌ Needs new command or CLI bridge |
| Trace browsing | Filesystem read | Not implemented | ❌ Needs new command or CLI bridge |
| Spec content | Filesystem read | Not implemented | ❌ Needs new command or CLI bridge |
| Tab navigation | HTML tabs | Placeholder only | ❌ Needs implementation |

---

## 2. UI Component Inventory

### 2.1 Components to Migrate from desktop-bun

| Component | Source | Complexity | Notes |
|-----------|--------|------------|-------|
| **App Shell** | main.html | Low | Header + tabs + content area + footer |
| **Tab Navigation** | main.ts `wireTabs()` | Low | Pure DOM, no framework needed |
| **Spec List** | main.ts `renderSpecs()` | Low | List of spec summaries with state badges |
| **ADR List** | main.ts `renderAdrs()` | Low | List of ADR summaries with status |
| **Trace List** | main.ts `renderTraces()` | Low | List of trace files with type badges |
| **Repo Bridge** | repo-bridge.ts | High | Replace with Tauri invoke() calls |
| **CLI Wrapper** | cli.ts | Medium | Replace with Tauri invoke() or tray CLI bridge |
| **Path Resolution** | paths.ts | Low | Replace with Tauri app paths |

### 2.2 New Components (Not in desktop-bun)

| Component | Purpose | Complexity |
|-----------|---------|------------|
| **Dashboard View** | Stats cards (total features, in-progress, shipped) | Low |
| **Feature Detail View** | Full feature info with spec, evidence, work packages | Medium |
| **Feature State Machine UI** | Visual state progression with advance button | Medium |
| **Work Package List** | List work packages for a feature | Medium |
| **Evidence Viewer** | Display evidence items (test results, screenshots) | Low |
| **Create Feature Dialog** | Modal/form for creating new features | Low |
| **Settings/Preferences** | App preferences (repo path, theme) | Low |

---

## 3. Tauri Migration Plan

### 3.1 Architecture Mapping

| desktop-bun Pattern | Tauri Equivalent | Notes |
|---------------------|------------------|-------|
| `Electrobun.Electroview` | `window.__TAURI__` | Tauri JS API |
| `electrobun.rpc.request.getRepoState` | `invoke('list_features')` | Direct command call |
| `RepoBridge.listSpecs()` | `invoke('list_features')` | SQLite-backed |
| `RepoBridge.listAdrs()` | New: `list_adrs` command or CLI bridge | Filesystem read |
| `RepoBridge.listTraces()` | New: `list_traces` command or CLI bridge | Filesystem read |
| `RepoBridge.readText()` | New: `read_file` command or CLI bridge | Filesystem read |
| `CLI.run()` | `invoke('run_cli', { args })` or tray CLI bridge | Shell-out |
| `AppPaths.fromCwd()` | `invoke('get_app_paths')` | Tauri path resolution |

### 3.2 Data Access Strategy

**Phase 1: SQLite-backed data (immediate)**
- Use existing Tauri commands for features, work_packages, dashboard stats
- Frontend calls `invoke('list_features')`, `invoke('get_dashboard_stats')`, etc.
- All data comes from the SQLite database via Rust commands

**Phase 2: Filesystem-backed data (follow-up)**
- ADRs, traces, and spec content are filesystem-based
- Option A: Add Tauri commands that read the filesystem (e.g., `list_adrs`, `read_spec`)
- Option B: Shell out to `agileplus` CLI for these operations
- **Recommended: Option A** for performance, Option B as fallback

**Phase 3: CLI bridge for write operations**
- Write operations (specify, advance, triage) shell out to `agileplus` CLI
- This matches the tray-cli design in `TRAY_CLI.md`
- Avoids duplicating complex orchestration logic

### 3.3 Frontend Framework Decision

**Recommendation: Vanilla JS/TS (no framework)**

Rationale:
- The existing desktop-bun UI is vanilla JS (no React/Vue/Svelte)
- The UI is simple: tabs, lists, forms, dialogs
- No build step needed (Tauri serves static files directly)
- Minimal bundle size
- Easy to maintain and debug

**If framework is desired later:**
- React + Vite: Best Tauri ecosystem support, largest community
- Svelte: Smallest bundle, closest to vanilla DOM
- Vue: Good middle ground

**Decision: Start with vanilla. Migrate to a framework only if complexity demands it.**

---

## 4. View Architecture

### 4.1 Proposed Views

```
desktop/src/
  index.html                    # App shell (header, nav, content area)
  styles/
    main.css                    # Global styles + CSS variables
    components.css              # Component-specific styles
  js/
    app.js                      # Bootstrap, routing, state management
    views/
      dashboard.js              # Dashboard view (stats cards)
      features.js               # Feature list view
      feature-detail.js         # Feature detail view (tabs: spec/evidence/WP)
      adrs.js                   # ADR list view
      traces.js                 # Trace list view
    components/
      tab-nav.js                # Tab navigation component
      stat-card.js              # Dashboard stat card
      feature-list-item.js      # Feature list item with state badge
      state-badge.js            # State badge (colored pill)
      modal.js                  # Modal dialog component
      toast.js                  # Toast notification component
    services/
      tauri-bridge.js           # Tauri invoke() wrapper
      cli-bridge.js             # CLI shell-out wrapper (for write ops)
    utils/
      state-machine.js          # Feature state transitions
      escape-html.js            # XSS prevention
```

### 4.2 View Descriptions

#### Dashboard View (Home)
- **Stats cards**: Total Features, In Progress, Shipped, Total Work Packages
- **Recent features**: Last 5 features with state badges
- **Quick actions**: Create Feature, Refresh
- **Data source**: `invoke('get_dashboard_stats')`, `invoke('list_features')`

#### Feature List View
- **List**: All features with name, state badge, created date
- **Filter**: By state (dropdown or tabs)
- **Sort**: By name, state, created date
- **Actions**: Create, Advance State, View Detail
- **Data source**: `invoke('list_features')`

#### Feature Detail View
- **Header**: Feature name, state badge, created/updated dates
- **Tabs**: Spec, Evidence, Work Packages
- **Spec tab**: Markdown content (from filesystem or CLI)
- **Evidence tab**: List of evidence items
- **Work Packages tab**: List of WPs with state and advance button
- **Actions**: Advance State, Edit, Delete
- **Data source**: `invoke('get_feature', { id })`, `invoke('list_work_packages', { feature_id })`

#### ADR List View
- **List**: All ADRs with title and status
- **Click**: Opens ADR content in detail panel
- **Data source**: `invoke('list_adrs')` or `invoke('run_cli', { args: ['list', '--type', 'adr'] })`

#### Trace List View
- **List**: All trace files with name and type (jsonl/md)
- **Click**: Opens trace content in detail panel
- **Data source**: `invoke('list_traces')` or `invoke('run_cli', { args: ['list', '--type', 'trace'] })`

---

## 5. Tray Integration

See `TRAY_CLI.md` for full tray-cli design. Key integration points:

### 5.1 Data Flow

```
Tray Menu (dynamic) ← SQLite (read-only) ← Tauri commands
Tray Actions (write) ← CLI shell-out ← agileplus binary
```

### 5.2 UI ↔ Tray Coordination

- **Tray refresh**: After any write operation in the UI, the tray menu is rebuilt
- **Window show/hide**: Tray "Show Window" toggles the main window
- **State sync**: Both UI and tray read from the same SQLite database
- **Notifications**: Write operation results shown as desktop notifications + tray tooltip

### 5.3 Shared Components

| Component | UI Usage | Tray Usage |
|-----------|----------|------------|
| `tauri-bridge.js` | `invoke('list_features')` | Direct SQLite via Rust |
| `state-machine.js` | Display valid next states | Menu only shows next valid action |
| `escape-html.js` | Render feature names | Not used (native menu) |

---

## 6. File Structure

### 6.1 Proposed Layout

```
desktop/
  src/
    index.html              # App shell
    styles/
      main.css              # Global styles + CSS variables
      components.css        # Component styles
    js/
      app.js                # Bootstrap + routing
      views/                # View modules
        dashboard.js
        features.js
        feature-detail.js
        adrs.js
        traces.js
      components/           # Reusable UI components
        tab-nav.js
        stat-card.js
        feature-list-item.js
        state-badge.js
        modal.js
        toast.js
      services/             # Data access layer
        tauri-bridge.js     # Tauri invoke() wrapper
        cli-bridge.js       # CLI shell-out wrapper
      utils/                # Shared utilities
        state-machine.js
        escape-html.js
  src-tauri/
    src/
      main.rs               # Entry point
      lib.rs                 # Tauri setup + tray
      commands.rs            # SQLite-backed commands
      db.rs                  # Database schema + state
      adrs.rs                # NEW: ADR filesystem commands
      traces.rs              # NEW: Trace filesystem commands
      cli_bridge.rs          # NEW: CLI shell-out command
    icons/                   # App icons (generated)
    tauri.conf.json          # Tauri config
    Cargo.toml               # Rust dependencies
  package.json               # Frontend dependencies (if any)
```

### 6.2 New Rust Modules Needed

| Module | Purpose | Priority |
|--------|---------|----------|
| `adrs.rs` | `list_adrs`, `read_adr` commands | P1 |
| `traces.rs` | `list_traces`, `read_trace` commands | P1 |
| `cli_bridge.rs` | `run_cli` command (shell-out to agileplus) | P2 |
| `work_packages.rs` | `list_work_packages`, `create_work_package` commands | P2 |
| `evidence.rs` | `list_evidence`, `create_evidence` commands | P2 |

---

## 7. Data Flow

### 7.1 Frontend → Tauri → SQLite (Read)

```javascript
// Frontend (js/services/tauri-bridge.js)
const features = await invoke('list_features');
const stats = await invoke('get_dashboard_stats');
```

```
┌─────────────┐     invoke()      ┌──────────────┐     SQL      ┌──────────┐
│  Frontend   │ ────────────────→ │  Tauri Cmd   │ ──────────→ │  SQLite  │
│  (JS/HTML)  │ ←──────────────── │  (Rust)      │ ←────────── │  (DB)    │
└─────────────┘    JSON response  └──────────────┘   query      └──────────┘
```

### 7.2 Frontend → Tauri → CLI (Write)

```javascript
// Frontend (js/services/cli-bridge.js)
const result = await invoke('run_cli', {
  args: ['specify', '--feature', 'auth', '--from-file', specContent]
});
```

```
┌─────────────┐     invoke()      ┌──────────────┐    spawn     ┌──────────┐
│  Frontend   │ ────────────────→ │  Tauri Cmd   │ ──────────→ │ agileplus│
│  (JS/HTML)  │ ←──────────────── │  (Rust)      │ ←────────── │   CLI    │
└─────────────┘    JSON response  └──────────────┘   stdout     └──────────┘
```

### 7.3 Frontend → Tauri → Filesystem (ADRs/Traces)

```javascript
// Frontend (js/services/tauri-bridge.js)
const adrs = await invoke('list_adrs');
const specContent = await invoke('read_spec', { featureId: 'auth' });
```

```
┌─────────────┐     invoke()      ┌──────────────┐     read     ┌──────────┐
│  Frontend   │ ────────────────→ │  Tauri Cmd   │ ──────────→ │   FS     │
│  (JS/HTML)  │ ←──────────────── │  (Rust)      │ ←────────── │  (disk)  │
└─────────────┘    JSON response  └──────────────┘   file       └──────────┘
```

---

## 8. Styling Strategy

### 8.1 CSS Approach: Vanilla CSS with Custom Properties

**Rationale:**
- Matches existing desktop-bun `main.css` pattern
- No build step required
- CSS custom properties enable theming
- Tauri serves static files directly

### 8.2 Design Tokens (CSS Variables)

```css
:root {
  /* Colors */
  --bg: #0f1115;
  --bg-elev: #161a22;
  --bg-surface: #1c2333;
  --fg: #e6e8eb;
  --fg-dim: #8a92a3;
  --accent: #4f8cff;
  --accent-hover: #6ba0ff;
  --border: #232836;
  --success: #4ade80;
  --warning: #fbbf24;
  --error: #f87171;

  /* State colors */
  --state-created: #374151;
  --state-specified: #1e40af;
  --state-researched: #7c3aed;
  --state-planned: #b45309;
  --state-implementing: #047857;
  --state-validated: #0369a1;
  --state-shipped: #15803d;
  --state-retrospected: #6b21a8;

  /* Spacing */
  --pad-xs: 4px;
  --pad-sm: 8px;
  --pad-md: 12px;
  --pad-lg: 18px;
  --pad-xl: 24px;

  /* Typography */
  --font-sans: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  --font-mono: ui-monospace, SFMono-Regular, Menlo, monospace;
  --font-size-sm: 11px;
  --font-size-base: 13px;
  --font-size-lg: 16px;

  /* Borders */
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
  --radius-full: 9999px;
}
```

### 8.3 Component Styles

Reuse patterns from desktop-bun `main.css`:
- **Lists**: `border: 1px solid var(--border)`, `border-radius: var(--radius-md)`
- **Tabs**: `aria-selected` attribute for active state
- **Cards**: `background: var(--bg-elev)`, `padding: var(--pad-lg)`
- **Badges**: Colored background + white text, `border-radius: var(--radius-full)`

---

## 9. Build & Dev Workflow

### 9.1 Development

```bash
# Start Tauri dev server (watches Rust + serves frontend)
cd desktop
pnpm tauri dev

# Frontend changes: edit src/index.html, src/js/*, src/styles/*
# Rust changes: edit src-tauri/src/*
# Both auto-reload on save
```

### 9.2 Frontend Build

**No build step for vanilla JS.** Tauri serves `src/` directly.

If adding a framework later:
```bash
# With Vite + React (example)
cd desktop
pnpm install
pnpm dev        # Start Vite dev server
pnpm tauri dev  # Start Tauri with Vite
```

### 9.3 Testing

```bash
# Rust tests
cargo test -p agileplus-desktop

# Frontend: manual testing via pnpm tauri dev
# Automated E2E: Playwright or Tauri's built-in testing (future)
```

### 9.4 Production Build

```bash
cd desktop
pnpm tauri build    # Outputs: src-tauri/target/release/bundle/
```

---

## 10. Migration Phases

### Phase 1: Foundation (Current → Next)
**Goal: Replace placeholder HTML with functional UI**

- [ ] Replace `src/index.html` with proper app shell (header, nav, content area)
- [ ] Implement `js/services/tauri-bridge.js` (invoke wrapper)
- [ ] Implement `js/views/dashboard.js` (stats cards using `get_dashboard_stats`)
- [ ] Implement `js/views/features.js` (feature list using `list_features`)
- [ ] Implement `js/components/state-badge.js` (colored state pills)
- [ ] Implement `js/app.js` (view routing, bootstrap)
- [ ] Port `main.css` design tokens to new `styles/main.css`

**Deliverable:** Functional dashboard + feature list with real data from SQLite

### Phase 2: Feature Detail (Week 2)
**Goal: Full feature management**

- [ ] Implement `js/views/feature-detail.js` (spec/evidence/WP tabs)
- [ ] Add `list_work_packages` command to Rust backend
- [ ] Implement `js/components/modal.js` (create feature dialog)
- [ ] Implement feature state advance UI (next valid state button)
- [ ] Add `cli_bridge.rs` for write operations (specify, advance)

**Deliverable:** Create features, view details, advance state

### Phase 3: ADR/Trace Browsing (Week 3)
**Goal: Filesystem-backed views**

- [ ] Add `list_adrs`, `read_adr` commands to Rust backend
- [ ] Add `list_traces`, `read_trace` commands to Rust backend
- [ ] Implement `js/views/adrs.js` (ADR list + content)
- [ ] Implement `js/views/traces.js` (trace list + content)

**Deliverable:** Browse ADRs and traces from the UI

### Phase 4: Tray Integration (Week 4)
**Goal: Dynamic tray menu + UI coordination**

- [ ] Implement `src/tray/mod.rs` (module root)
- [ ] Implement `src/tray/menu.rs` (dynamic menu builder)
- [ ] Implement `src/tray/cli.rs` (CLI bridge)
- [ ] Implement `src/tray/notify.rs` (notification helpers)
- [ ] Wire tray ↔ UI state synchronization

**Deliverable:** Dynamic tray menu with feature list and lifecycle actions

### Phase 5: Polish & Package (Week 5+)
**Goal: Production-ready distribution**

- [ ] Design AgilePlus icon (1024x1024)
- [ ] Generate all icon sizes via `tauri icon`
- [ ] Test on macOS, Windows, Linux
- [ ] Set up GitHub Actions release workflow
- [ ] Configure auto-updater
- [ ] Publish to Homebrew Cask / winget

**Deliverable:** Production-ready installers for all platforms

---

## 11. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| **ADR/Trace commands not implemented** | Phase 3 blocked | Start with CLI bridge as fallback, add native commands in parallel |
| **CLI binary not found** | Write operations fail | Graceful fallback: gray out lifecycle items, show "CLI not installed" |
| **SQLite schema changes** | Data loss | Use migrations, test upgrade path |
| **Frontend complexity grows** | Maintainability | Consider React/Svelte migration if >500 LOC of vanilla JS |
| **Cross-platform CSS issues** | Visual bugs | Test on all platforms, use system fonts, avoid platform-specific CSS |

---

## 12. Success Criteria

- [ ] Dashboard shows real stats from SQLite
- [ ] Feature list displays all features with correct state badges
- [ ] Feature detail shows spec, evidence, work packages
- [ ] Create feature works end-to-end (UI → Rust → SQLite)
- [ ] Advance feature state works (UI → CLI → SQLite → UI refresh)
- [ ] ADR and trace browsing functional
- [ ] Tray menu shows dynamic feature list
- [ ] All views styled with consistent dark theme
- [ ] No console errors in production
- [ ] Builds successfully on macOS, Windows, Linux

---

## References

- `TRAY_CLI.md` — Tray-cli module design
- `ICONS_AND_PACKAGING.md` — Icon requirements and packaging strategy
- `docs/specs/UI_ARCHITECTURE.md` — Original UI architecture spec
- `desktop-bun/src/` — Source Electrobun app (reference)
- ADR-020: Rust Core + Tauri Desktop (`docs/adr/0020-rust-core-tauri-desktop-primary.md`)
