# templates Specification

## Overview

A collection of reusable project templates for various languages, frameworks, and domains. Used as scaffolding for new projects within the Phenotype/DINOForge ecosystem.

## Template Categories

### Language Templates
- **template-lang-rust**: Rust projects (workspace, crates)
- **template-lang-python**: Python projects (FastAPI/FastMCP)
- **template-lang-swift**: Swift projects
- **template-lang-kotlin**: Kotlin projects
- **template-lang-zig**: Zig projects
- **template-lang-mojo**: Mojo projects
- **template-lang-elixir-hex**: Elixir projects (Hex package)

### Domain Templates
- **template-domain-service-api**: Service/API backend templates
- **template-domain-webapp**: Web application templates
- **template-program-ops-remote**: Program operations templates

### Utility Templates
- **linters**: Linter configuration templates
- **quality**: Quality assurance templates

## Structure

Each template includes:
- Standard AGENTS.md and governance files
- Pre-configured CI/CD workflows (`.github/workflows/`)
- Code quality tooling (Ruff, mypy, etc.)
- Documentation structure

## Usage

```bash
# Clone template for new project
git clone template-lang-rust-remote-20260402 my-new-project
```

## Dependencies

Per-template (see individual template README.md)