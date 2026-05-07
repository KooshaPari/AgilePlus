#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

MODE="${1:-verify}"
case "$MODE" in
  verify)
    echo "=== Phenotype SAST Quality Gate ==="
    FAILURES=0

    # Semgrep check
    echo "[1/4] Semgrep scan..."
    if command -v semgrep &>/dev/null && [ -f "$REPO_ROOT/.semgrep.yml" ]; then
      if semgrep --config="$REPO_ROOT/.semgrep.yml" --error "$REPO_ROOT" 2>&1 | head -50; then
        echo "✓ Semgrep passed"
      else
        FAILURES=$((FAILURES + 1))
      fi
    else
      echo "⚠ Semgrep check skipped"
    fi

    # Secret scanning
    echo "[2/4] TruffleHog secret scan..."
    if command -v trufflehog &>/dev/null; then
      if trufflehog git file://"$REPO_ROOT" --only-verified 2>&1 | head -20; then
        echo "✓ Secret scanning passed"
      else
        FAILURES=$((FAILURES + 1))
      fi
    else
      echo "⚠ Secret scanning skipped"
    fi

    # Cargo checks
    echo "[3/4] Cargo checks..."
    if [ -f "$REPO_ROOT/Cargo.toml" ]; then
      cargo clippy --all-targets -- -D warnings && echo "✓ Clippy passed" || FAILURES=$((FAILURES + 1))
      cargo fmt --all -- --check && echo "✓ Format passed" || FAILURES=$((FAILURES + 1))
    fi

    # Test run
    echo "[4/4] Tests..."
    if [ -f "$REPO_ROOT/Cargo.toml" ]; then
      cargo test --all --quiet && echo "✓ Tests passed" || FAILURES=$((FAILURES + 1))
    fi

    if [ $FAILURES -eq 0 ]; then
      echo "✓ All quality gates passed!"
      exit 0
    else
      echo "❌ $FAILURES checks failed"
      exit 1
    fi
    ;;
  *)
    echo "Unknown mode: $MODE"
    exit 1
    ;;
esac
