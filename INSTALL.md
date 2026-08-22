# AgilePlus - Spec-driven Governance Framework

Phenotype-org spec-driven development framework for AI agents.

## Quick Install

### One-liner (Windows PowerShell)
```powershell
irm https://raw.githubusercontent.com/KooshaPari/AgilePlus/main/install.ps1 | iex
```

### Chocolatey
```powershell
choco install agileplus
```

### WinGet
```powershell
winget install KooshaPari.AgilePlus
```

### From Source
```bash
git clone https://github.com/KooshaPari/AgilePlus
cd AgilePlus
cargo build --release -p agileplus-cli -p agileplus-dashboard
./target/release/agileplus --help
```

## Usage

```bash
agileplus dashboard           # Launch the dashboard
agileplus rubric score <repo> # Score governance
agileplus specify             # Create feature specs
agileplus cockpit publish     # Publish scorecard
agileplus dag pick            # Pick work from DAG
agileplus --help              # All commands
```

## Uninstall

```powershell
irm https://raw.githubusercontent.com/KooshaPari/AgilePlus/main/uninstall.ps1 | iex
```

## Links

- [GitHub](https://github.com/KooshaPari/AgilePlus)
- [Releases](https://github.com/KooshaPari/AgilePlus/releases)
- [Documentation](https://github.com/KooshaPari/AgilePlus/blob/main/README.md)

## License

MIT