# my-package — CLAUDE.md

## Project Overview

- **Name**: my-package
- **Description**: A TypeScript library
- **Language Stack**: TypeScript
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

A TypeScript library

## Development Rules

- All public types implement `Debug` and `Clone`
- Error types use `thiserror` with `#[from]` conversions
- Full type annotations in public APIs
