#!/bin/bash
# Publish Rust crate to crates.io

set -e

cd "$(dirname "$0")"

echo "=== Checking traceability-macros ==="
cargo check

echo ""
echo "=== Running tests ==="
cargo test

echo ""
echo "=== Building release ==="
cargo build --release

echo ""
echo "=== Publishing to crates.io ==="
echo "Command: cargo publish"
echo ""
read -p "Press Enter to publish to crates.io, or Ctrl+C to cancel..."

cargo publish

echo ""
echo "=== Published successfully ==="
echo "Crate: traceability-macros"
echo "Version: $(grep version Cargo.toml | head -1 | cut -d'"' -f2)"
