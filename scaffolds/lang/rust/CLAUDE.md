# my-crate — CLAUDE.md

## Project Overview

- **Name**: my-crate
- **Description**: A Rust library
- **Language Stack**: Rust
- **Author**: Developer

## Key Commands

```bash
# Build
cargo build --release

# Test
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt
```

## Architecture

A Rust library

## Development Rules

- All public types implement `Debug` and `Clone`
- Error types use `thiserror` with `#[from]` conversions
- Full type annotations in public APIs
