# Package Publishing Guide

This guide covers publishing packages to all four supported package registries: npm, PyPI, crates.io, and Go modules.

## Overview

| Ecosystem | Registry | Naming Convention | Workflow File |
|-----------|----------|-------------------|---------------|
| npm | [registry.npmjs.org](https://registry.npmjs.org) | `@phenotype/*` | `publish-npm.yml` |
| Python | [pypi.org](https://pypi.org) | `phenotype-*` | `publish-pypi.yml` |
| Rust | [crates.io](https://crates.io) | `phenotype-*` | `publish-crate.yml` |
| Go | [proxy.golang.org](https://proxy.golang.org) | `github.com/KooshaPari/phenotype-go-*` | `publish-go.yml` |

---

## npm (@phenotype/* scope)

### Prerequisites

1. **npm account** with access to the `@phenotype` organization
2. **GitHub Secret**: `NPM_TOKEN` - An npm automation token with publish rights

### Setting up npm Token

1. Log in to [npmjs.com](https://www.npmjs.com)
2. Go to Access Tokens → Generate New Token → Automation
3. Copy the token value
4. Add to GitHub Secrets:
   - Repository → Settings → Secrets and variables → Actions
   - New repository secret: `NPM_TOKEN`

### Package Structure

```json
{
  "name": "@phenotype/auth-ts",
  "version": "0.1.0",
  "description": "Phenotype authentication for TypeScript",
  "main": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "files": ["dist/"],
  "scripts": {
    "build": "tsc -p tsconfig.json",
    "test": "vitest run",
    "prepublishOnly": "npm run build && npm test"
  },
  "publishConfig": {
    "access": "public"
  },
  "license": "MIT"
}
```

### Publishing

**Via GitHub Actions (recommended):**

```bash
# Trigger workflow_dispatch with package name and version
gh workflow run publish-npm.yml -f package=auth-ts -f version=0.1.0 -f dry_run=false
```

**Via git tag:**

```bash
git tag @phenotype/auth-ts@0.1.0
git push origin @phenotype/auth-ts@0.1.0
```

**Manual (not recommended for CI/CD environments):**

```bash
cd phenotype-auth-ts
npm version 0.1.0
npm publish --access public
```

### Available Packages

| Package | Path | Description |
|---------|------|-------------|
| `@phenotype/auth-ts` | `phenotype-auth-ts` | Authentication library |
| `@phenotype/docs-engine` | `phenotype-docs-engine` | Documentation engine |
| `@phenotype/agent-core` | `phenotype-agent-core` | Agent core library |
| `@phenotype/task-engine` | `phenotype-task-engine` | Task engine |
| `@phenotype/research-engine` | `phenotype-research-engine` | Research engine |
| `@phenotype/config-ts` | `phenotype-config-ts` | TypeScript config |
| `@phenotype/ui` | `phenotype-hub/packages/ui` | UI components |

---

## PyPI (phenotype-* packages)

### Prerequisites

1. **PyPI account** with 2FA enabled
2. **Trusted Publisher** configured (recommended) OR API token
3. **GitHub Secret**: `PYPI_API_TOKEN` (if not using Trusted Publishing)

### Setting up PyPI Trusted Publisher (Recommended)

Trusted Publishers allow publishing without long-lived tokens:

1. Log in to [pypi.org](https://pypi.org)
2. Go to Account settings → Publishing
3. Add a new pending publisher:
   - **PyPI Project Name**: `phenotype-sdk`
   - **Owner**: `KooshaPari`
   - **Repository name**: `repos`
   - **Workflow name**: `publish-pypi.yml`
   - **Environment name**: `pypi`

### Package Structure (pyproject.toml)

```toml
[project]
name = "phenotype-sdk"
version = "0.1.0"
description = "Phenotype SDK for infrastructure and operations"
readme = "README.md"
requires-python = ">=3.9"
license = {text = "MIT"}
authors = [
    {name = "Phenotype Team", email = "info@phenotype.dev"}
]
classifiers = [
    "Development Status :: 3 - Alpha",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
]
keywords = ["phenotype", "sdk", "infrastructure"]

[project.urls]
Homepage = "https://github.com/KooshaPari/phenotype"
Documentation = "https://docs.phenotype.dev"
Repository = "https://github.com/KooshaPari/phenotype.git"
"Bug Tracker" = "https://github.com/KooshaPari/phenotype/issues"

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.hatch.build.targets.wheel]
packages = ["src/phenosdk"]
```

### Python Package Naming

| Local Name | PyPI Name | Path |
|------------|-----------|------|
| `phenosdk` | `phenotype-sdk` | `python/phenosdk` |
| `pheno-core` | `phenotype-core` | `python/pheno-core` |
| `pheno-atoms` | `phenotype-atoms` | `python/pheno-atoms` |
| `pheno-agents` | `phenotype-agents` | `python/pheno-agents` |
| `pheno-llm` | `phenotype-llm` | `python/pheno-llm` |
| `pheno-mcp` | `phenotype-mcp` | `python/pheno-mcp` |

### Publishing

**Via GitHub Actions (recommended):**

```bash
# Publish phenosdk as phenotype-sdk v0.1.0
gh workflow run publish-pypi.yml -f package=phenosdk -f version=0.1.0 -f dry_run=false
```

**Via git tag:**

```bash
git tag phenotype-phenosdk-v0.1.0
git push origin phenotype-phenosdk-v0.1.0
```

**Manual (for testing only):**

```bash
cd python/phenosdk
python -m build
twine upload dist/*
```

---

## crates.io (phenotype-* crates)

### Prerequisites

1. **crates.io account** with verified email
2. **GitHub Secret**: `CARGO_TOKEN` - API token from crates.io

### Setting up crates.io Token

1. Log in to [crates.io](https://crates.io)
2. Account Settings → API Tokens
3. Generate a new token with `publish-new` and `publish-update` scopes
4. Add to GitHub Secrets as `CARGO_TOKEN`

### Crate Structure (Cargo.toml)

```toml
[package]
name = "phenotype-error-core"
version = "0.2.0"
edition = "2021"
license = "MIT"
description = "Error handling primitives for the Phenotype ecosystem"
repository = "https://github.com/KooshaPari/phenotype"
documentation = "https://docs.rs/phenotype-error-core"
keywords = ["phenotype", "error", "error-handling"]
categories = ["rust-patterns"]
authors = ["Phenotype Team <info@phenotype.dev>"]
rust-version = "1.70"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1.0"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

### Required Metadata for Publishing

Every crate must have in `Cargo.toml`:

```toml
[package]
name = "phenotype-<name>"
version = "<semver>"
edition = "2021"
license = "MIT"  # or "Apache-2.0", etc.
description = "<clear description>"
repository = "https://github.com/KooshaPari/phenotype"
```

### Publishing

**Via GitHub Actions (recommended):**

```bash
gh workflow run publish-crate.yml -f crate=phenotype-error-core -f dry_run=false
```

**Via git tag:**

```bash
git tag phenotype-error-core-v0.2.0
git push origin phenotype-error-core-v0.2.0
```

**Manual:**

```bash
cd crates/phenotype-error-core
cargo publish --token $CARGO_TOKEN
```

### Available Crates

| Crate | Path | Description |
|-------|------|-------------|
| `phenotype-error-core` | `crates/phenotype-error-core` | Error handling |
| `phenotype-test-infra` | `crates/phenotype-test-infra` | Test infrastructure |
| `phenotype-contracts` | `crates/phenotype-contracts` | Shared contracts |
| `phenotype-health` | `crates/phenotype-health` | Health checks |
| `phenotype-cache-adapter` | `crates/phenotype-cache-adapter` | Cache adapter |
| `phenotype-state-machine` | `crates/phenotype-state-machine` | State machine |
| `phenotype-policy-engine` | `crates/phenotype-policy-engine` | Policy engine |
| `phenotype-mcp` | `crates/phenotype-mcp` | MCP protocol |

---

## Go Modules (phenotype-go-*)

### Prerequisites

1. **GitHub repository** at `github.com/KooshaPari/phenotype-go-*`
2. **Proper module path** in `go.mod`

### Module Structure (go.mod)

```go
module github.com/KooshaPari/phenotype-go-kit

go 1.22

require (
    github.com/go-chi/chi/v5 v5.2.2
    github.com/google/uuid v1.6.0
)
```

### Publishing

Go modules don't require registry authentication. They use **semantic version tags** in Git:

**Via GitHub Actions (recommended):**

```bash
gh workflow run publish-go.yml -f module=template-commons/phenotype-go-kit -f version=v0.1.0 -f dry_run=false
```

**Via git tag (manual):**

```bash
# For a module at template-commons/phenotype-go-kit
git tag template-commons/phenotype-go-kit/v0.1.0
git push origin template-commons/phenotype-go-kit/v0.1.0
```

### Tag Format

| Module Path | Tag Format |
|-------------|------------|
| `template-commons/phenotype-go-kit` | `template-commons/phenotype-go-kit/v0.1.0` |
| `template-commons/phenotype-go-auth` | `template-commons/phenotype-go-auth/v0.1.0` |
| `template-commons/phenotype-go-cli` | `template-commons/phenotype-go-cli/v0.1.0` |

### Available Modules

| Module | Path | Description |
|--------|------|-------------|
| `phenotype-go-kit` | `template-commons/phenotype-go-kit` | Base Go kit |
| `phenotype-go-auth` | `template-commons/phenotype-go-auth` | Auth utilities |
| `phenotype-go-cli` | `template-commons/phenotype-go-cli` | CLI framework |
| `phenotype-go-middleware` | `template-commons/phenotype-go-middleware` | HTTP middleware |
| `phenotype-go-config` | `template-commons/phenotype-go-config` | Configuration |

---

## GitHub Secrets Required

Add these secrets to the repository (Settings → Secrets and variables → Actions):

| Secret | Required For | How to Obtain |
|--------|--------------|---------------|
| `NPM_TOKEN` | npm publishing | npm automation token |
| `CARGO_TOKEN` | crates.io publishing | crates.io API token |
| `PYPI_API_TOKEN` | PyPI publishing (fallback) | PyPI API token |
| `GITHUB_TOKEN` | All (auto-provided) | Auto-generated by GitHub |

### Using Trusted Publishing (Preferred for PyPI)

Instead of `PYPI_API_TOKEN`, configure Trusted Publishers on PyPI:

1. Go to [pypi.org/manage/account/publishing](https://pypi.org/manage/account/publishing)
2. Add a pending publisher for each package
3. No token needed in GitHub secrets

---

## Version Management

### Semantic Versioning

All packages follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.y.z): Breaking changes
- **MINOR** (x.Y.z): New features, backwards compatible
- **PATCH** (x.y.Z): Bug fixes

### Version Bump Commands

**npm:**
```bash
npm version patch   # 0.1.0 -> 0.1.1
npm version minor   # 0.1.0 -> 0.2.0
npm version major   # 0.1.0 -> 1.0.0
```

**Python:**
```bash
# Edit pyproject.toml manually or use hatch
hatch version patch
hatch version minor
hatch version major
```

**Rust:**
```bash
# Edit Cargo.toml manually
# Use cargo-edit for convenience:
cargo install cargo-edit
cargo bump patch
cargo bump minor
cargo bump major
```

**Go:**
```bash
# Tag-based versioning
git tag <path>/v0.1.1
git tag <path>/v0.2.0
git tag <path>/v1.0.0
```

---

## Pre-Publish Checklist

Before publishing any package:

- [ ] All tests pass
- [ ] Version number updated in package manifest
- [ ] CHANGELOG.md updated
- [ ] Documentation complete
- [ ] GitHub Secrets configured (if needed)
- [ ] Dry run successful

### Running Dry Runs

**npm:**
```bash
npm publish --dry-run
```

**Python:**
```bash
python -m build
twine check dist/*
```

**Rust:**
```bash
cargo publish --dry-run
```

**Go:**
```bash
# No dry run needed - just validate the module
go mod verify
go test ./...
```

---

## Troubleshooting

### npm: "403 Forbidden"

- Verify `NPM_TOKEN` is valid and not expired
- Ensure the package name is correct (`@phenotype/*`)
- Check you have publish rights to the organization

### PyPI: "Invalid API Token"

- Use Trusted Publishing instead of tokens (more secure)
- If using tokens, ensure it has the correct scope
- Verify 2FA is enabled on PyPI account

### crates.io: "already exists"

- You cannot re-publish the same version
- Bump the version number in `Cargo.toml`
- Use `cargo yank` to yank a broken version

### Go: "module not found"

- Ensure the module path in `go.mod` matches the repository
- Wait 1-2 minutes after tagging for proxy.golang.org to index
- Use `GOPROXY=direct` to bypass proxy

---

## Phase 2.4/2.5 Status

### Completed

- [x] npm workflow (`publish-npm.yml`)
- [x] PyPI workflow (`publish-pypi.yml`)
- [x] crates.io workflow (`publish-crate.yml`)
- [x] Go module workflow (`publish-go.yml`)
- [x] Package guide documentation

### Required Actions

1. **Add GitHub Secrets**:
   - `NPM_TOKEN` - for npm publishing
   - `CARGO_TOKEN` - for crates.io publishing
   - Configure Trusted Publishers on PyPI (preferred over tokens)

2. **Update Package Metadata**:
   - Ensure all `Cargo.toml` files have proper metadata
   - Update Python `pyproject.toml` files with correct PyPI names
   - Make npm packages public (`"private": false`)

3. **Test Publishing**:
   - Run each workflow with `dry_run=true` first
   - Publish a test version to verify setup

---

## Related Documentation

- [npm Publishing Best Practices](https://docs.npmjs.com/packages-and-modules/contributing-packages-to-the-registry)
- [PyPI Trusted Publishers](https://docs.pypi.org/trusted-publishers/)
- [crates.io Publishing Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Go Module Versioning](https://go.dev/doc/modules/version-numbers)
