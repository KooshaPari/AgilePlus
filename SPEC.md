# AgilePlus — SPEC.md

## Architecture Overview

AgilePlus is a real-time portfolio and asset tracking application built on a Rust-based services framework. It manages user portfolios, financial data, and provides external API integrations.

## Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| Backend | Rust 2024 edition | Tokio + async runtime |
| Web Framework | Axum | REST API routes |
| Async Runtime | Tokio 1.41+ | Multi-threaded, full feature set |
| Observability | Tracing + `tracy-client` | Async-aware profiling |
| Data Layer | Custom SQL/JSON | Query-structured JSON for portfolio data |
| CLI Runner | `agileplus-cli` | Standalone binary for portfolio operations |

## Key Commands

| Command | Description |
|---------|-------------|
| `cargo build --release` | Build all Rust binaries |
| `cargo test --workspace` | Run all tests |
| `cargo clippy --workspace --all-targets` | Lint all packages |
| `cargo fmt --all -- --check` | Check formatting |
| `just check` | Run formatting + clippy + tests |
| `just lint` | Run clippy only |
| `just start` | Start the development server |
| `cargo run -p agileplus-cli` | Run the CLI tool directly |

## Design Decisions

- **Query-structured JSON for portfolio data**: Flexibility in storing heterogeneous financial instrument data without rigid schema migrations.
- **Axum + Tokio for async I/O**: Leverages Rust's async ecosystem for high-throughput API handling with minimal latency.
- **`tracy-client` for async profiling**: Zero-cost instrumentation that can be toggled at compile time; enables frame-level analysis of async task execution.

## Integration Points

- `pheno-otel` — OpenTelemetry tracing for async service handlers
- `pheno-schema` — Zod schemas for portfolio data validation and API contracts
- `pheno-utils` — Shared Rust utility functions for data transformation and crypto helpers
