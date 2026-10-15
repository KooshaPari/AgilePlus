# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/KooshaPari/AgilePlus/releases/tag/agileplus-desktop-v0.2.0) - 2026-10-15

### Added

- *(desktop)* add splash screen, spinners, branding, and fix Tauri IPC
- *(desktop)* add crash reporting with local log files
- *(desktop)* add macOS code signing, notarization, and auto-updater
- *(desktop)* generate placeholder AgilePlus icons (1024x1024 → 32x32)
- *(desktop)* add ADR/trace views, fix tauri config, add CSS
- *(desktop)* add backend commands, tray module, and filesystem bridges
- *(desktop)* scaffold Tauri desktop app with tray and SQLite

### Fixed

- *(ci)* unblock rust linker, ruff py311 target, cargo-machete unused deps
- *(desktop)* keep tauri's test feature out of the release dependency graph
- resolve CI release build failures
- *(desktop)* connect to CLI's real .agileplus/agileplus.db

### Other

- Make promotion resumable across Git and SQLite; repair dependency and browser CI gates
- *(fmt)* format the workspace with CI's exact toolchain
- *(desktop)* format the Tauri test support with the desktop crate's own edition
- *(test)* clear CI fmt and clippy failures in test files this campaign added
- *(desktop)* cover the Tauri commands through the real IPC layer
- bring forward uncommitted changes
- PII sweep — redact names, emails, paths across all files
- *(release)* bump desktop to v0.2.0
- cargo-dist init, fix repo URL consistency, track desktop Cargo.toml
- apply cargo fmt to all crates
