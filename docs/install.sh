#!/usr/bin/env bash
set -euo pipefail

# AgilePlus installer — curl | bash
# Usage: curl -fsSL https://raw.githubusercontent.com/KooshaPari/AgilePlus/main/docs/install.sh | bash

REPO="KooshaPari/AgilePlus"
BINARY="agileplus"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

detect_platform() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os" in
    Linux)  os="linux" ;;
    Darwin) os="macos" ;;
    MINGW*|MSYS*|CYGWIN*) os="windows" ;;
    *) echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac
  case "$arch" in
    x86_64|amd64) arch="x86_64" ;;
    arm64|aarch64) arch="aarch64" ;;
    *) echo "Unsupported arch: $arch" >&2; exit 1 ;;
  esac
  echo "${os}-${arch}"
}

main() {
  local platform version tag_url tmp_dir
  platform="$(detect_platform)"
  
  # Get latest release tag
  if command -v gh &>/dev/null; then
    version="$(gh release list -R "$REPO" --limit 1 --json tagName -q '.[0].tagName')"
  else
    version="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | head -1 | cut -d'"' -f4)"
  fi
  
  if [ -z "$version" ]; then
    echo "ERROR: could not determine latest version" >&2
    exit 1
  fi
  
  echo "Installing AgilePlus ${version} for ${platform}..."
  
  local asset_name="agileplus-${platform}"
  local archive_ext="tar.gz"
  if [[ "$platform" == *"windows"* ]]; then
    archive_ext="zip"
  fi
  
  tag_url="https://github.com/${REPO}/releases/download/${version}"
  tmp_dir="$(mktemp -d)"
  
  curl -fsSL "${tag_url}/${asset_name}.${archive_ext}" -o "${tmp_dir}/${asset_name}.${archive_ext}"
  
  mkdir -p "$INSTALL_DIR"
  
  if [[ "$archive_ext" == "tar.gz" ]]; then
    tar -xzf "${tmp_dir}/${asset_name}.${archive_ext}" -C "$tmp_dir"
    cp "${tmp_dir}/agileplus" "${INSTALL_DIR}/${BINARY}"
  else
    # Windows via Git Bash
    unzip -o "${tmp_dir}/${asset_name}.${archive_ext}" -d "$tmp_dir"
    cp "${tmp_dir}/agileplus.exe" "${INSTALL_DIR}/${BINARY}.exe"
  fi
  
  chmod +x "${INSTALL_DIR}/${BINARY}" 2>/dev/null || true
  rm -rf "$tmp_dir"
  
  echo "✓ Installed to ${INSTALL_DIR}/${BINARY}"
  echo "  Add to PATH: export PATH=\"${INSTALL_DIR}:\$PATH\""
}

main "$@"
