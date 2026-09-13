#!/bin/bash
# build-universal.sh — Build a universal macOS binary (arm64 + x86_64)
# for AgilePlus Desktop using Tauri 2.
#
# Prerequisites:
#   - Rust toolchain with both aarch64-apple-darwin and x86_64-apple-darwin targets
#   - pnpm, Xcode command-line tools
#
# Usage:
#   cd desktop
#   ./scripts/build-universal.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

BUNDLE_NAME="AgilePlus Desktop"
EXEC_NAME="agileplus-desktop"
ARM64_APP="src-tauri/target/aarch64-apple-darwin/release/bundle/macos/${BUNDLE_NAME}.app"
X86_64_APP="src-tauri/target/x86_64-apple-darwin/release/bundle/macos/${BUNDLE_NAME}.app"
UNIVERSAL_DIR="src-tauri/target/universal/release/bundle/macos"
UNIVERSAL_APP="${UNIVERSAL_DIR}/${BUNDLE_NAME}.app"

echo "=== Building AgilePlus Desktop universal binary ==="

# ── Ensure Rust targets are installed ──────────────────────────────
echo "Checking Rust targets..."
rustup target add aarch64-apple-darwin 2>/dev/null || true
rustup target add x86_64-apple-darwin 2>/dev/null || true

# ── Build for aarch64 (Apple Silicon) ──────────────────────────────
echo ""
echo "--- Building for aarch64-apple-darwin ---"
pnpm tauri build --target aarch64-apple-darwin

# ── Build for x86_64 (Intel) ──────────────────────────────────────
echo ""
echo "--- Building for x86_64-apple-darwin ---"
pnpm tauri build --target x86_64-apple-darwin

# ── Validate build outputs ────────────────────────────────────────
if [ ! -d "$ARM64_APP" ]; then
  echo "ERROR: arm64 build output not found at: $ARM64_APP"
  exit 1
fi
if [ ! -d "$X86_64_APP" ]; then
  echo "ERROR: x86_64 build output not found at: $X86_64_APP"
  exit 1
fi

# ── Create universal .app ──────────────────────────────────────────
echo ""
echo "--- Creating universal binary ---"
rm -rf "$UNIVERSAL_APP"
cp -R "$ARM64_APP" "$UNIVERSAL_APP"

# Merge the main binary
ARM64_BIN="${ARM64_APP}/Contents/MacOS/${EXEC_NAME}"
X86_64_BIN="${X86_64_APP}/Contents/MacOS/${EXEC_NAME}"
UNIVERSAL_BIN="${UNIVERSAL_APP}/Contents/MacOS/${EXEC_NAME}"

lipo -create "$ARM64_BIN" "$X86_64_BIN" -output "$UNIVERSAL_BIN"
echo "  Merged main binary:"
file "$UNIVERSAL_BIN"

# Merge all other Mach-O binaries in the bundle (helper processes, frameworks, etc.)
echo "  Merging additional binaries..."
find "${UNIVERSAL_APP}/Contents" -type f -perm +111 | while read -r f; do
  [ -f "$f" ] || continue
  file_type=$(file -b "$f" | head -1)
  case "$file_type" in
    *"Mach-O"*)
      arm64_variant="${ARM64_APP}/Contents/${f#"${UNIVERSAL_APP}/Contents/"}"
      if [ -f "$arm64_variant" ]; then
        if file -b "$arm64_variant" | grep -q "Mach-O"; then
          lipo -create "$f" "$arm64_variant" -output "$f" 2>/dev/null || true
          echo "    + $(basename "$f")"
        fi
      fi
      ;;
  esac
done

# Re-sign the universal bundle
codesign --force --deep --sign - "$UNIVERSAL_APP" 2>/dev/null || true

echo ""
echo "Universal binary created successfully!"
echo "  App: $UNIVERSAL_APP"
echo ""

# ── Create DMG (optional, requires create-dmg) ────────────────────
if command -v create-dmg &>/dev/null; then
  DMG_OUT="${UNIVERSAL_DIR}/AgilePlus Desktop-universal.dmg"
  echo "Creating universal DMG..."
  create-dmg \
    --volname "AgilePlus Desktop" \
    --window-pos 200 120 \
    --window-size 600 400 \
    --icon-size 100 \
    --icon "${BUNDLE_NAME}.app" 175 190 \
    --hide-extension "${BUNDLE_NAME}.app" \
    --app-drop-link 425 190 \
    --no-internet-enable \
    "$DMG_OUT" \
    "${UNIVERSAL_APP}" 2>/dev/null || true

  if [ -f "$DMG_OUT" ]; then
    echo "Universal DMG created: $DMG_OUT"
  else
    echo "WARNING: DMG creation failed (create-dmg may not be installed)"
  fi
else
  echo "NOTE: Install 'create-dmg' to auto-generate a DMG"
  echo "      brew install create-dmg"
fi

echo ""
echo "Done."
