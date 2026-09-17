# Installing AgilePlus

AgilePlus ships multiple installation channels. Pick whichever fits your workflow.

| Channel | Best for | Command |
|---------|----------|---------|
| **curl (macOS/Linux)** | Quick install without Rust | `curl -fsSL ... \| bash` |
| **PowerShell (Windows)** | One-line Windows install | `irm ... \| iex` |
| **Bun (cross-platform)** | JavaScript/TypeScript runtime users | `bun run ...` |
| **cargo install** | Rust developers | `cargo install agileplus-cli` |
| **Prebuilt binaries** | Manual download | [GitHub Releases](https://github.com/KooshaPari/AgilePlus/releases) |
| **From source** | Contributors building the workspace | `git clone` + `cargo build` |
| **Start Menu** (Windows) | Desktop launcher | `packaging/start-menu.ps1` |

## macOS / Linux (curl)

```bash
curl -fsSL https://raw.githubusercontent.com/<REDACTED>/AgilePlus/main/docs/install.sh | bash
```

Customises the install directory via `INSTALL_DIR`:

```bash
INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/<REDACTED>/AgilePlus/main/docs/install.sh | bash
```

## Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/<REDACTED>/AgilePlus/main/docs/install.ps1 | iex
```

The installer adds `%LOCALAPPDATA%\AgilePlus\bin` to your user PATH automatically.

## Bun (cross-platform)

```bash
bun run https://raw.githubusercontent.com/<REDACTED>/AgilePlus/main/docs/install.ts
```

## From crates.io

```bash
cargo install agileplus-cli --locked
```

## From source

```bash
git clone https://github.com/KooshaPari/AgilePlus.git
cd AgilePlus
cargo build --release --package agileplus-cli
```

The built binary will be at `target/release/agileplus` (or `target/release/agileplus.exe` on Windows).

## Prebuilt binaries (GitHub Releases)

Tagged releases (`v*`) publish matrix-built archives for Linux, macOS (x86_64 + Apple Silicon), and Windows.

1. Open [Releases](https://github.com/KooshaPari/AgilePlus/releases).
2. Download the archive for your platform:
   - `agileplus-<version>-agileplus-linux-x86_64.tar.gz`
   - `agileplus-<version>-agileplus-macos-aarch64.tar.gz`
   - `agileplus-<version>-agileplus-macos-x86_64.tar.gz`
   - `agileplus-<version>-agileplus-windows-x86_64.zip`
3. Extract and place `agileplus` (or `agileplus.exe`) on your `PATH`.

## Windows Start Menu shortcut (Phenotype-Apps)

After installing the binary (any channel above):

```powershell
.\packaging\start-menu.ps1
```

## Verify installation

```bash
agileplus --version
agileplus --help
```

## Quick project bootstrap

No `agileplus init` command exists. In a git repo, create or revise a feature spec with the SDD CLI (interactive interview, or `--from-file`):

```bash
cd my-project
agileplus specify --feature my-feature
# or: agileplus specify --feature my-feature --from-file ./draft-spec.md
agileplus list
```

Platform health is **not** a top-level `agileplus status` (that is not a product feature command). Use:

```bash
agileplus platform status
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `agileplus: command not found` | Add `~/.local/bin` or `~/.cargo/bin` to `PATH` |
| Build fails on `protoc` | Install protobuf compiler 28.x |
| crates.io publish fails in CI | Ensure `CARGO_REGISTRY_TOKEN` secret is set |
| Start Menu shortcut has no icon | Add `packaging/agileplus.ico` or pass `-IconPath` |
