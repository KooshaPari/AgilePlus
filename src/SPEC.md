# src Specification

## Overview

A Rust workspace containing game-related tools, domain logic, and utilities for the DINOForge project. Provides cryptographic foundations, testing infrastructure, and various toolchains.

## Components

### Core Library (`src/`)
- **Hash**: SHA-256 and Blake3 content hashing for content IDs
- **Encryption**: AES-GCM symmetric encryption with PBKDF2 key derivation
- **Signatures**: HMAC computation and verification

### Tools (`Tools/`)
- **Cli**: Command-line interface utilities
- **DesktopCompanion**: Desktop application companion
- **McpServer**: Model Context Protocol server
- **PackCompiler**: Asset pack compilation tools
- **VFXPrefabGenerator**: Visual effects prefab generation
- **DinoforgeMcp**: DINOForge MCP integration

### Domains (`Domains/`)
- **Warfare**: Domain-specific game logic

### Testing (`Tests/`)
- Test infrastructure and fixtures

## Dependencies

- Rust 2021 edition
- Workspace with multiple crates

## Build

```bash
cargo build --workspace
cargo test --workspace
```