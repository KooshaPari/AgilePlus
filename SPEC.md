# AgilePlus — SPEC.md

## Architecture Overview

AgilePlus is a real-time portfolio and asset tracking application built on a 21-crate Rust workspace with an Axum HTTP/gRPC backend, a SQLite-backed local store, an Electrobun desktop app, and a React/TypeScript dashboard.

## Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| Backend | Rust 2024 edition | Axum HTTP server + Tonic gRPC |
| Async Runtime | Tokio 1.41+ | Multi-threaded, full feature set |
| Web Framework | Axum | REST API routes |
| Persistence | SQLite | SQLite-backed local store via GORM |
| Frontend | React + TypeScript | Web dashboard |
| Desktop | Electrobun | Native desktop app |
| Observability | Tracing + `tracy-client` | Async-aware profiling |
| CLI | clap | Top-level binary `agileplus` |

## Key Commands

| Command | Description |
|---------|-------------|
| `cargo build --release` | Build all Rust binaries |
| `cargo test --all-features` | Run all tests with all features enabled |
| `cargo clippy --all-targets --all-features -- -D warnings` | Lint all packages |
| `cargo fmt --all` | Format all code |
| `cargo fmt --all --check` | Check formatting without modifying |
| `cargo deny check advisories` | Audit dependencies for vulnerabilities |
| `cargo tarpaulin --workspace` | Generate coverage report |
| `cargo run -p agileplus-cli` | Run the CLI tool directly |

## Design Decisions

- **21-crate workspace for modularity**: Domain, application, infrastructure, and interface layers are separated into individual crates to enforce clean architecture boundaries and enable independent testing.
- **Hexagonal architecture (Ports & Adapters)**: Domain types are pure Rust structs with zero external dependencies; ports are async traits; adapters are swappable implementations selected at compile time.
- **SQLite for local-first persistence**: SQLite provides a lightweight, zero-config database that aligns with the local-first design philosophy, eliminating the need for external database infrastructure.

## Integration Points

- `phenotype-error-core` — Shared error types and error handling primitives across the workspace
- `phenotype-logging` — Structured logging infrastructure for observability
- `pheno-axum-stack` — Axum middleware and stack utilities for the HTTP server
- `pheno-errors` — Error taxonomy and structured error responses for API endpoints
- `pheno-tracing` — OpenTelemetry tracing integration for distributed observability
- `pheno-config` — Typed configuration loading and validation
