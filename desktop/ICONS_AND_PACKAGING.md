# AgilePlus Desktop: Icon Requirements & Packaging Strategy

> ADR-020 selected Tauri 2 as the primary desktop interface. This document defines the
> icon asset requirements, generation workflow, and cross-platform packaging/distribution
> strategy for the Tauri desktop app.

---

## 1. Current State

The `desktop/src-tauri/icons/` directory contains **default Tauri logo placeholders** (16 files).
These are not AgilePlus branding and must be replaced before any public release.

**Current files (all default Tauri logos):**

```
32x32.png          icon.icns           Square30x30Logo.png
128x128.png        icon.ico            Square44x44Logo.png
128x128@2x.png     icon.png            Square71x71Logo.png
                   StoreLogo.png       Square89x89Logo.png
                                       Square107x107Logo.png
                                       Square142x142Logo.png
                                       Square150x150Logo.png
                                       Square284x284Logo.png
                                       Square310x310Logo.png
```

**`tauri.conf.json` bundle.icon references:**

```json
"icon": [
  "icons/32x32.png",
  "icons/128x128.png",
  "icons/128x128@2x.png",
  "icons/icon.icns",
  "icons/icon.ico"
]
```

---

## 2. Icon Requirements by Platform

### 2.1 Desktop (All Platforms)

| File | Size | Format | Usage |
|------|------|--------|-------|
| `32x32.png` | 32 x 32 px | PNG, RGBA, 32-bit | Windows taskbar, Linux tray |
| `128x128.png` | 128 x 128 px | PNG, RGBA, 32-bit | Windows Start Menu, Linux desktop |
| `128x128@2x.png` | 256 x 256 px | PNG, RGBA, 32-bit | HiDPI/Retina displays |
| `icon.png` | 1024 x 1024 px | PNG, RGBA, 32-bit | Source/master icon, used by some Linux DEs |
| `icon.ico` | 16/24/32/48/64/256 px | ICO (multi-layer) | Windows executable icon |
| `icon.icns` | 16-1024 px layers | ICNS | macOS app icon |

**PNG requirements:**
- Width must equal height (square)
- RGBA color space (RGB + Alpha transparency)
- 32 bits per pixel (8 bits per channel)

### 2.2 macOS (Additional)

The `.icns` format bundles multiple resolution layers:

| Layer | Pixel Size | Retina Size | Filename |
|-------|-----------|-------------|----------|
| icon_16x16 | 16 x 16 | - | `icon_16x16.png` |
| icon_16x16@2x | 32 x 32 | 16pt Retina | `icon_16x16@2x.png` |
| icon_32x32 | 32 x 32 | - | `icon_32x32.png` |
| icon_32x32@2x | 64 x 64 | 32pt Retina | `icon_32x32@2x.png` |
| icon_128x128 | 128 x 128 | - | `icon_128x128.png` |
| icon_128x128@2x | 256 x 256 | 128pt Retina | `icon_128x128@2x.png` |
| icon_256x256 | 256 x 256 | - | `icon_256x256.png` |
| icon_256x256@2x | 512 x 512 | 256pt Retina | `icon_256x256@2x.png` |
| icon_512x512 | 512 x 512 | - | `icon_512x512.png` |
| icon_512x512@2x | 1024 x 1024 | 512pt Retina | `icon_512x512@2x.png` |

The `tauri icon` command generates this `.icns` automatically from a 1024x1024 source.

### 2.3 Windows (Additional)

The `.ico` must include layers: **16, 24, 32, 48, 64, and 256 pixels**.
For optimal display in development, the 32px layer should be first.

**Windows Store logos** (generated automatically by `tauri icon`):

| File | Size | Usage |
|------|------|-------|
| `Square30x30Logo.png` | 30 x 30 | Windows Store tile |
| `Square44x44Logo.png` | 44 x 44 | Windows Store tile |
| `Square71x71Logo.png` | 71 x 71 | Windows Store tile |
| `Square89x89Logo.png` | 89 x 89 | Windows Store tile |
| `Square107x107Logo.png` | 107 x 107 | Windows Store tile |
| `Square142x142Logo.png` | 142 x 142 | Windows Store tile |
| `Square150x150Logo.png` | 150 x 150 | Windows Store tile |
| `Square284x284Logo.png` | 284 x 284 | Windows Store tile |
| `Square310x310Logo.png` | 310 x 310 | Windows Store tile |
| `StoreLogo.png` | 50 x 50 | Windows Store badge |

### 2.4 Linux

Linux uses PNG icons directly. Commonly needed sizes:

| Size | Usage |
|------|-------|
| 16 x 16 | Panel/tray icon |
| 32 x 32 | Taskbar |
| 48 x 48 | File manager |
| 64 x 64 | Desktop shortcut |
| 128 x 128 | Application launcher |
| 256 x 256 | HiDPI launcher |
| 512 x 512 | AppStream/store listing |
| 1024 x 1024 | Source/responsive |

---

## 3. Icon Generation Workflow

### 3.1 From Source Image (Recommended)

Tauri provides a built-in CLI command that generates every required format and size:

```bash
# Prerequisites: a 1024x1024 PNG or SVG with transparency
# Place your source image in desktop/

cd desktop
pnpm tauri icon ../branding/agileplus-icon-1024.png
# or
cargo tauri icon ../branding/agileplus-icon-1024.png
```

**What this generates (in `src-tauri/icons/`):**
- `32x32.png` -- Windows/Linux tray
- `128x128.png` -- Windows Start Menu
- `128x128@2x.png` -- HiDPI displays
- `icon.png` -- 1024x1024 master
- `icon.icns` -- macOS app bundle
- `icon.ico` -- Windows executable
- All Windows Store `Square*Logo.png` files
- `StoreLogo.png` -- Windows Store badge

### 3.2 Source Image Requirements

| Property | Requirement |
|----------|-------------|
| Format | PNG (preferred) or SVG with transparency |
| Dimensions | 1024 x 1024 px minimum (square) |
| Color space | RGBA (with alpha transparency) |
| Bit depth | 32-bit (8 bits per channel) |
| Background | Transparent (not white/colored rectangle) |
| Style | Simple enough to be recognizable at 16x16 |

**Design tips for small sizes:**
- Avoid thin lines (< 2px at 1024) -- they disappear at 32px
- Avoid fine text -- it becomes illegible
- Use bold, simple shapes
- Test by squinting or zooming out -- the icon should be identifiable

### 3.3 Custom Sizes (Advanced)

If you need custom PNG sizes beyond the defaults:

```bash
# Generate specific PNG sizes only (no ICO/ICNS)
pnpm tauri icon -p 16 -p 48 -p 256 -p 512 ../branding/icon.png
```

### 3.4 Manual Generation (External Tools)

If `tauri icon` output is insufficient, use:

| Tool | Purpose | Platform |
|------|---------|----------|
| [ImageMagick](https://imagemagick.org/) | Batch resize, format conversion | All |
| [IconGenerator.app](https://apps.apple.com/app/icon-generator/id483129430) | macOS icon set | macOS |
| [GIMP](https://www.gimp.org/) | ICO export with layers | All |
| [Figma](https://figma.com) | Design source, export at all sizes | Web |

**Manual .ico generation with ImageMagick:**

```bash
# Create multi-layer ICO from source
convert icon-1024.png \
  \( -clone 0 -resize 16x16 \) \
  \( -clone 0 -resize 24x24 \) \
  \( -clone 0 -resize 32x32 \) \
  \( -clone 0 -resize 48x48 \) \
  \( -clone 0 -resize 64x64 \) \
  \( -clone 0 -resize 256x256 \) \
  -delete 0 icon.ico
```

**Manual .icns generation with ImageMagick:**

```bash
# Tauri's CLI handles this, but for manual:
# See https://github.com/tauri-apps/tauri/blob/1.x/tooling/cli/src/helpers/icns.json
# Easier to use: png2icns or Tauri CLI
```

---

## 4. Source Image Placement

### Recommended structure

```
branding/
  agileplus-icon-1024.png    # Master source (1024x1024, transparent PNG)
  agileplus-icon.svg         # Vector source (for future scaling)
  agileplus-wordmark.svg     # Wordmark (optional, for docs/README)
```

**Why keep a separate branding directory?**
- The `src-tauri/icons/` directory is generated -- never edit manually
- Source art lives outside the build tree
- Designers work in `branding/`, CI generates icons
- Single source of truth for all icon variants

---

## 5. Packaging Strategy

### 5.1 macOS

| Format | Command | Notes |
|--------|---------|-------|
| **.dmg** | `pnpm tauri build` | Default output. Drag-to-Applications installer. |
| **.app bundle** | Built into DMG | Native macOS app bundle structure |
| **.tar.gz** | Manual | Portable archive (no installer) |

**macOS requirements:**
- **Apple Developer ID** for code signing (`TAURI_SIGNING_PRIVATE_KEY`, `APPLE_CERTIFICATE`)
- **Notarization** required for distribution outside App Store (`APPLE_ID`, `APPLE_PASSWORD`)
- **Hardened Runtime** enabled by default in Tauri
- Gatequard/noracker: Users see a warning without notarization

**Code signing setup (environment variables):**

```bash
export APPLE_CERTIFICATE="path/to/certificate.p12"
export APPLE_CERTIFICATE_PASSWORD="cert-password"
export APPLE_SIGNING_IDENTITY="Developer ID Application: Phenotype (TEAM_ID)"
export APPLE_ID="kooshapari@gmail.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="YOUR_TEAM_ID"
```

**DMG customization (tauri.conf.json):**

```json
{
  "bundle": {
    "macOS": {
      "minimumSystemVersion": "10.15",
      "dmg": {
        "appPosition": { "x": 180, "y": 170 },
        "applicationFolderPosition": { "x": 480, "y": 170 },
        "windowSize": { "width": 660, "height": 400 }
      }
    }
  }
}
```

### 5.2 Windows

| Format | Command | Notes |
|--------|---------|-------|
| **NSIS** (.exe) | `pnpm tauri build` | Recommended for most users. Modern installer. |
| **MSI** (.msi) | `pnpm tauri build --bundles msi` | Enterprise/silent install. Group Policy. |

**Windows requirements:**
- **Code signing** optional but recommended (`TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`)
- Without signing: SmartScreen will flag the installer
- NSIS is the default and recommended format

**NSIS customization:**

```json
{
  "bundle": {
    "windows": {
      "nsis": {
        "installMode": "both",
        "headerImage": "branding/nsis-header.bmp",
        "sidebarImage": "branding/nsis-sidebar.bmp"
      }
    }
  }
}
```

### 5.3 Linux

| Format | Command | Notes |
|--------|---------|-------|
| **AppImage** | `pnpm tauri build` | Portable, no install needed. Widest compatibility. |
| **.deb** | `pnpm tauri build --bundles deb` | Debian/Ubuntu package manager |
| **.rpm** | `pnpm tauri build --bundles rpm` | Fedora/RHEL/CentOS package manager |
| **.tar.gz** | `pnpm tauri build --bundles app` | Raw binary archive |

**Linux requirements:**
- No code signing needed (yet)
- AppImage is the most portable format
- `.deb` is essential for Debian/Ubuntu users (largest desktop Linux base)

**Linux desktop entry** (auto-generated by Tauri):

```ini
[Desktop Entry]
Name=AgilePlus Desktop
Exec=agileplus-desktop
Icon=agileplus-desktop
Type=Application
Categories=Development;
```

### 5.4 Build Commands Reference

```bash
cd desktop

# Build for current platform (all bundle formats)
pnpm tauri build

# Build with specific bundle targets
pnpm tauri build --bundles dmg          # macOS only
pnpm tauri build --bundles nsis         # Windows only
pnpm tauri build --bundles appimage     # Linux only
pnpm tauri build --bundles deb          # Linux only

# Verbose output
pnpm tauri build --verbose

# Build with debug profile (larger, no optimization)
pnpm tauri build --debug

# Generate icons from source
pnpm tauri icon ../branding/agileplus-icon-1024.png
```

---

## 6. Distribution Channels

### 6.1 Primary Channels

| Channel | Platform | Format | Notes |
|---------|----------|--------|-------|
| **GitHub Releases** | All | DMG, NSIS, AppImage, deb | Primary distribution. Tag-triggered. |
| **Homebrew** | macOS | DMG (via cask) | `brew install --cask agileplus` |
| **winget** | Windows | NSIS | `winget install AgilePlus` |
| **AUR** | Linux (Arch) | PKGBUILD | Community-maintained |

### 6.2 Optional Channels (Future)

| Channel | Platform | Notes |
|---------|----------|-------|
| **Mac App Store** | macOS | Requires Apple Developer Program ($99/yr), review process |
| **Microsoft Store** | Windows | Requires Microsoft Partner Center account |
| **Flathub** | Linux | Flatpak, wider Linux reach |
| **Snap Store** | Linux | Canonical's package format |

### 6.3 Auto-Update

Tauri has a built-in updater plugin (`tauri-plugin-updater`). Configure for GitHub Releases:

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/Phenotype/AgilePlus/releases/latest/download/latest.json"
      ],
      "pubkey": "YOUR_UPDATER_PUBLIC_KEY"
    }
  }
}
```

Generate signing keys:

```bash
pnpm tauri signer generate -w ~/.tauri/agileplus.key
# Save the private key securely (for CI)
# Distribute the public key (for updater verification)
```

---

## 7. CI/CD Integration

### GitHub Actions Release Workflow

```yaml
# .github/workflows/release.yml (sketch)
name: Release

on:
  push:
    tags: ['v*']

jobs:
  release:
    strategy:
      matrix:
        include:
          - platform: macos-latest
            target: aarch64-apple-darwin
          - platform: macos-latest
            target: x86_64-apple-darwin
          - platform: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
          - platform: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install Tauri CLI
        run: pnpm add -D @tauri-apps/cli

      - name: Build & Sign
        pnpm tauri build
        env:
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}

      - name: Upload artifacts
        uses: softprops/action-gh-release@v2
        with:
          files: desktop/src-tauri/target/release/bundle/**/*
```

---

## 8. Distribution Size Expectations

Based on ADR-020 research and typical Tauri builds:

| Platform | Format | Expected Size | Notes |
|----------|--------|---------------|-------|
| macOS | .dmg | 5-10 MB | Tiny vs Electron's 80-150 MB |
| Windows | .exe (NSIS) | 5-10 MB | Embedded WebView2 (usually pre-installed on Win 10+) |
| Windows | .msi | 5-10 MB | Same as NSIS |
| Linux | AppImage | 5-10 MB | Self-contained |
| Linux | .deb | 5-10 MB | Depends on libwebkit2gtk |

---

## 9. Action Items

### Before First Release (P0)

- [ ] Design AgilePlus icon (1024x1024 transparent PNG)
- [ ] Place source in `branding/agileplus-icon-1024.png`
- [ ] Run `pnpm tauri icon` to regenerate all sizes
- [ ] Verify `icon.icns` and `icon.ico` are correctly generated
- [ ] Set up Apple Developer ID for macOS signing
- [ ] Set up GitHub Actions release workflow
- [ ] Generate updater signing keys

### Before Public Beta (P1)

- [ ] Set up notarization pipeline (macOS)
- [ ] Test DMG installation flow on clean macOS
- [ ] Test NSIS installation flow on clean Windows
- [ ] Test AppImage on Ubuntu/Fedora
- [ ] Configure `tauri-plugin-updater` for auto-updates
- [ ] Submit to Homebrew Cask (macOS)

### Post-Launch (P2)

- [ ] Submit to winget (Windows)
- [ ] Create AUR package (Arch Linux)
- [ ] Consider Mac App Store / Microsoft Store submission
- [ ] Design tray icon variants (16x16, monochrome for dark/light mode)

---

## 10. Tray Icon Considerations

AgilePlus uses the system tray (per ADR-020). Tauri tray icons have specific needs:

| Platform | Size | Format | Notes |
|----------|------|--------|-------|
| macOS | 16x16 (22pt @2x) | PNG | Template image (monochrome, auto-adapts to dark/light menu bar) |
| Windows | 16x16 | ICO or PNG | System tray icon |
| Linux | 16x16 or 22x22 | PNG | Depends on DE (GNOME, KDE, etc.) |

**macOS template tray icons:**
- Use black icon on transparent background
- macOS automatically adjusts for dark/light mode
- Name the file with "Template" suffix or set `setTemplate(true)` in code

**Tauri tray plugin configuration:**

```rust
// In main.rs -- already enabled via features = ["tray-icon"]
let tray = tauri::tray::TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .tooltip("AgilePlus Desktop")
    .build(app)?;
```

---

## References

- [Tauri v2 Icon Documentation](https://v2.tauri.app/develop/icons/)
- [Tauri v2 Distribution Guide](https://v2.tauri.app/distribute/)
- [Tauri macOS DMG Guide](https://v2.tauri.app/distribute/dmg/)
- [ADR-020: Rust Core + Tauri Desktop](../../docs/adr/0020-rust-core-tauri-desktop-primary.md)
- [Tauri Updater Plugin](https://v2.tauri.app/plugin/updater/)
