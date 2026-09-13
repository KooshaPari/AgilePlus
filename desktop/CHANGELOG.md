# Changelog

All notable changes to the AgilePlus Desktop app are documented here.

## [0.1.0] - 2026-09-13

First release of the Tauri desktop client.

### Added

- Tauri 2 desktop application with system tray integration
- Feature management (list, create, update state)
- Work package CRUD
- Evidence tracking
- ADR and trace file viewing
- CLI bridge for advanced operations (Triage, Lifecycle)
- Dashboard statistics
- SQLite database with automatic migrations
- Cross-platform support (macOS aarch64/x86_64, Windows x86_64, Linux x86_64)

### Notes

- 5 Dependabot alerts are transitive/lockfile issues (glib via Tauri tray-icon, vitest/npm, cryptography/pip) - not directly fixable, need upstream updates
- Auto-update not yet configured
- macOS code signing notarization not yet set up
