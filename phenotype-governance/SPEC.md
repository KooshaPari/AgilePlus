# phenotype-governance Specification

Canonical definition of system behavior.

## Overview

phenotype-governance is the centralized governance and operations repository for the Phenotype ecosystem. It provides standardized CI/CD templates, linting configurations, Git hooks, and operational tooling to ensure consistency across all Phenotype projects.

## Features

### CI/CD Templates

Standardized CI/CD pipelines for multiple languages:

- **Rust** — `templates/ci/ci-rust.yml.template`
- **TypeScript** — `templates/ci/ci-typescript.yml.template`
- **Python** — `templates/ci/ci-python.yml.template`
- **Go** — `templates/ci/ci-go.yml.template`

Additional pipeline templates:

- Coverage reporting (`coverage-rust.yml.template`)
- Container builds (`container-rust.yml.template`)
- Releases (`release-rust.yml.template`)
- ADR validation (`adr-validation.yml.template`)

### Linting & Code Quality Configurations

Unified linting configurations:

- **Rust** — `configs/rust/rustfmt.toml`, `configs/rust/clippy.toml`
- **TypeScript** — `configs/typescript/eslint.config.js`
- **Python** — `configs/python/ruff.toml`
- **Go** — `configs/go/golangci.yml`
- **Universal** — `configs/universal/.editorconfig`, `configs/_typos.toml`

### Git Hooks

Automated enforcement of coding standards:

- Pre-commit hooks (`hooks/pre-commit`)
- Pre-push hooks (`hooks/pre-push`)
- Commit message validation (`hooks/commit-msg`)
- Prepare commit message (`hooks/prepare-commit-msg`)

### GitHub Workflows

Automated CI/CD and security scanning:

- **CI** — `.github/workflows/ci.yml`
- **Release** — `.github/workflows/release.yml`
- **Quality Gate** — `.github/workflows/quality-gate.yml`
- **Security Guard** — `.github/workflows/security-guard.yml`
- **SAST Quick** — `.github/workflows/sast-quick.yml`
- **SAST Full** — `.github/workflows/sast-full.yml`
- **Rust Quality** — `.github/workflows/rust-quality.yml`

### Security Scanning

Semgrep rules for security analysis:

- `unsafe-patterns.yml` — Common unsafe patterns
- `secrets-detection.yml` — Secret detection
- `architecture-violations.yml` — Architecture compliance

### Documentation

VitePress-based documentation system:

- Architecture Decision Records (ADRs)
- User journeys
- User stories
- Traceability matrix

## Architecture

```
phenotype-governance/
├── configs/                 # Linting and code quality configs
│   ├── rust/
│   ├── typescript/
│   ├── python/
│   ├── go/
│   └── universal/
├── docs/                    # VitePress documentation
│   ├── adr/                # Architecture Decision Records
│   ├── journeys/
│   ├── stories/
│   └── traceability/
├── hooks/                   # Git hooks
├── templates/               # CI/CD and integration templates
│   ├── ci/                 # CI pipeline templates
│   ├── rust/               # Rust-specific templates
│   └── github/             # GitHub-specific templates
├── .github/workflows/       # GitHub Actions workflows
└── scripts/                 # Operational scripts
```

## Requirements

- **FR-001**: Provide CI/CD templates for Rust, TypeScript, Python, and Go
- **FR-002**: Unified linting configurations across all supported languages
- **FR-003**: Git hooks for pre-commit and pre-push validation
- **FR-004**: GitHub Actions workflows for CI/CD and security
- **FR-005**: Security scanning rules via Semgrep
- **FR-006**: Documentation system with ADR support
- **FR-007**: Integration checklist template for new projects
- **FR-008**: Devcontainer configuration for standardized development environments
