# Tray-CLI Module Design

> AgilePlus desktop tray integration with the `agileplus` CLI binary.

## 1. Problem

The Tauri tray icon currently only offers "Show Window" and "Quit". Users need quick access to
AgilePlus lifecycle commands (dashboard, list features, triage, advance state) without opening the
full window or switching to a terminal.

## 2. Architecture Overview

```
+--------------------------------------------------+
|                    Tauri Tray                     |
|  +--------------------------------------------+  |
|  |  Dashboard Summary                         |  |
|  |  ─────────────────                         |  |
|  |  3 features   (1 implementing, 2 planned)  |  |
|  |  7 work packages (2 doing, 5 pending)      |  |
|  +--------------------------------------------+  |
|  |  Features                                   |  |
|  |   > auth-login    [implementing]            |  |
|  |   > dark-mode     [planned]                 |  |
|  |   > api-v2        [specified]               |  |
|  +--------------------------------------------+  |
|  |  Quick Actions                              |  |
|  |   > Triage "fix login bug"                  |  |
|  |   > Open Dashboard in Browser               |  |
|  |   > Refresh                                 |  |
|  +--------------------------------------------+  |
|  |  Show Window                                |  |
|  |  Quit                                       |  |
|  +--------------------------------------------+  |
+--------------------------------------------------+
          |                    |
          v                    v
  +----------------+   +------------------+
  |  Direct SQLite |   |  agileplus CLI   |
  |  (read-only)   |   |  (write/advance) |
  +----------------+   +------------------+
```

## 3. Module Structure

```
desktop/src-tauri/src/
  lib.rs           # existing
  commands.rs      # existing (Tauri invoke commands)
  db.rs            # existing (SQLite state)
  tray/            # NEW module
    mod.rs         # tray menu builder + event routing
    menu.rs        # menu item definitions and dynamic rebuild
    cli.rs         # shell-out bridge to agileplus binary
    notify.rs      # notification helper (tray tooltip + OS toast)
```

### 3.1 `tray/mod.rs`

Re-exports and wires the tray subsystem into `lib.rs::run()`.

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tray subsystem — dynamic menu, CLI bridge, notifications.

pub mod menu;
pub mod cli;
pub mod notify;
```

### 3.2 `tray/menu.rs`

Builds and rebuilds the `tauri::menu::Menu` with dynamic feature list.

**Key types:**

```rust
use tauri::menu::{Menu, MenuItem, Submenu, PredefinedMenuItem};
use tauri::AppHandle;

/// Build the full tray menu from current DB state.
pub fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error>;
```

**Menu structure (top to bottom):**

| Item | Type | Action |
|------|------|--------|
| **AgilePlus** | Submenu | Dashboard summary (feature/WP counts) |
| --- | Separator | |
| **Features** | Submenu | Dynamic list of active features with state badges |
| → `feature-slug` | Submenu per feature | Advance, View, Open Spec |
| --- | Separator | |
| **Quick Actions** | Submenu | Triage, Open API Dashboard, Refresh |
| --- | Separator | |
| **Show Window** | MenuItem | Existing show-window action |
| **Quit** | MenuItem | Existing quit action |

**Dynamic rebuild:** The menu is rebuilt after each write operation or on a 30-second
polling interval (configurable). Tauri 2.x supports `tray.set_menu(...)` for hot-swapping.

### 3.3 `tray/cli.rs`

Shell-out bridge to the `agileplus` binary.

```rust
use std::process::Command;
use serde_json::Value;

/// Path to the agileplus binary.
const AGILEPLUS_BIN: &str = "/Users/<REDACTED>/bin/agileplus";

/// Run an agileplus CLI command and capture JSON output.
pub async fn run_cli_command(args: &[&str]) -> Result<String, String> {
    // Spawn process, capture stdout+stderr, enforce 30s timeout.
}

/// Run a read-only CLI command that returns JSON.
pub async fn run_cli_json(args: &[&str]) -> Result<Value, String> {
    let stdout = run_cli_command(args).await?;
    serde_json::from_str(&stdout).map_err(|e| format!("JSON parse: {e}"))
}

/// Run a lifecycle command (specify, research, plan, implement, validate, ship).
/// These are long-running; the caller should show a progress indicator.
pub async fn run_lifecycle_command(
    command: &str,
    feature_slug: &str,
    extra_args: &[&str],
    on_progress: impl Fn(String) + Send + 'static,
) -> Result<String, String>;
```

**Design decisions:**

- Uses `std::process::Command` (sync) wrapped in `tokio::task::spawn_blocking` rather than
  `tokio::process::Command` to keep dependencies minimal and match the CLI's own sync patterns.
- 30-second timeout for read commands; 5-minute timeout for lifecycle commands.
- JSON output flag (`--json`) is appended automatically for commands that support it.
- stderr is captured and included in error messages for debugging.
- The binary path is resolved at startup from the Tauri config or falls back to the constant.

### 3.4 `tray/notify.rs`

Notification helpers for tray feedback.

```rust
use tauri::{AppHandle, tray::TrayIconBuilder};

/// Update the tray icon tooltip with a summary line.
pub fn update_tooltip(app: &AppHandle, summary: &str);

/// Send a desktop notification (uses tauri-plugin-notification if available,
/// falls back to tray tooltip update).
pub fn notify(app: &AppHandle, title: &str, body: &str);
```

## 4. Commands Exposed via Tray

### 4.1 Read-Only (Direct SQLite via Tauri Commands)

These call the existing `commands.rs` functions directly — no CLI binary needed.

| Tray Action | Tauri Command | Result Display |
|-------------|--------------|----------------|
| Dashboard submenu (counts) | `get_dashboard_stats` | Submenu header text |
| Feature list (with state) | `list_features` | Dynamic submenu items |
| Feature detail | `get_feature` | Tooltip / info panel |

### 4.2 Lifecycle (Shell Out to CLI Binary)

These invoke the `agileplus` binary. They require the binary to be installed.

| Tray Action | CLI Command | Notes |
|-------------|-------------|-------|
| Advance feature state | `agileplus <next-state> --feature <slug>` | e.g., `agileplus specify --feature auth` |
| Triage a string | `agileplus triage "<text>" --output json` | Opens input dialog first |
| Open API dashboard | Opens `http://localhost:<port>` in browser | Uses `open` crate or shell |
| Refresh | Rebuilds menu from DB | Re-reads SQLite, updates tooltip |

### 4.3 Lifecycle Commands Detail

The `advance feature` submenu determines the next valid state based on the current state:

```
Current State    | Next Action Label     | CLI Command
-----------------+-----------------------+---------------------------
created          | Specify               | agileplus specify --feature <slug>
specified        | Research              | agileplus research --feature <slug>
researched       | Plan                  | agileplus plan --feature <slug>
planned          | Implement             | agileplus implement --feature <slug>
implementing     | Validate              | agileplus validate --feature <slug>
validated        | Ship                  | agileplus ship --feature <slug>
shipped          | Retrospect            | agileplus retrospective --feature <slug>
retrospected     | (terminal — grayed)   | n/a
```

## 5. Integration with Existing Code

### 5.1 Changes to `lib.rs`

```rust
mod tray;  // new module

// In run():
// - Remove the inline tray menu construction (lines 21-55)
// - Replace with:
tray::menu::build_tray_menu(&app)
    .map_err(|e| e)?;

// - Add tray menu event handler that delegates to tray/menu handlers
```

### 5.2 Cargo.toml Additions

```toml
# Already present
tauri-plugin-shell = "2"

# New (optional, for richer notifications)
# tauri-plugin-notification = "2"

# For opening URLs in browser
# open = "5"
```

### 5.3 Menu Event Routing

```rust
// In tray/mod.rs or lib.rs
.on_menu_event(move |app, event| {
    let id = event.id().as_ref();
    match id {
        "show" => { /* existing show-window logic */ }
        "quit" => std::process::exit(0),
        "refresh" => { tray::menu::refresh_menu(app); }
        "triage" => { /* open input dialog, then run triage */ }
        "open-dashboard" => { /* open browser to API dashboard */ }
        id if id.starts_with("advance:") => {
            let slug = &id["advance:".len()..];
            // Run lifecycle command asynchronously
        }
        _ => {}
    }
})
```

## 6. Security Considerations

### 6.1 Command Injection Prevention

- **Never interpolate user input directly into CLI arguments.** The `cli.rs` module passes
  arguments as a `&[&str]` slice to `Command::args()`, which uses `execvp` semantics (no shell
  interpretation). This eliminates shell injection.
- The triage text input is passed as a single argument, not concatenated into a shell string.

### 6.2 Binary Path Validation

- The `agileplus` binary path is resolved from a hardcoded constant or Tauri config, not from
  user input or environment variables that could be tampered with.
- On startup, verify the binary exists and is executable; disable lifecycle menu items if not.

### 6.3 Filesystem Access

- The tray module does **not** read or write files directly. All filesystem operations go through
  the `agileplus` CLI binary or the existing SQLite state.
- The Tauri shell plugin's `Command` API runs child processes without shell expansion.

### 6.4 State Machine Enforcement

- The tray menu only exposes the **next valid state** for each feature, never arbitrary transitions.
- This matches the domain's `Feature::transition()` validation, providing defense in depth.

### 6.5 Sensitive Data

- No credentials, API keys, or tokens are stored in tray menu items or tooltip text.
- CLI output is not logged to the system log (only success/failure status).
- The triage input dialog does not persist input text after the command runs.

### 6.6 Process Lifecycle

- CLI child processes are killed if the Tauri app exits (Tauri shell plugin handles this).
- 30-second timeout on read commands; 5-minute timeout on lifecycle commands prevents zombie
  processes from consuming resources.

### 6.7 Permissions

- The tray module only runs read commands (dashboard, list) and lifecycle commands that the
  `agileplus` binary already supports. No new permissions are required beyond what the CLI
  already requests.

## 7. Error Handling

| Error Class | User-Facing Behavior |
|-------------|---------------------|
| Binary not found | Lifecycle menu items are grayed out; tooltip says "CLI not installed" |
| CLI command timeout | Notification: "Command timed out. Open window for details." |
| CLI command failure (non-zero exit) | Notification with stderr excerpt |
| SQLite read failure | Menu shows cached/stale data; refresh retries |
| Invalid state transition | Domain error displayed in notification; menu only shows valid next states |

## 8. File Changes Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `src/tray/mod.rs` | **New** | Module root, re-exports |
| `src/tray/menu.rs` | **New** | Dynamic menu builder |
| `src/tray/cli.rs` | **New** | CLI bridge to agileplus binary |
| `src/tray/notify.rs` | **New** | Tooltip + notification helpers |
| `src/lib.rs` | **Modified** | Replace inline tray setup with `tray::` calls |
| `Cargo.toml` | **Modified** | Add `open` crate (optional) |

Estimated size: ~250 lines across the 4 new files; ~15 lines changed in `lib.rs`.
