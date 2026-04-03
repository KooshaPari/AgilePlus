# Phenotype Governance

Infrastructure governance for Phenotype ecosystem projects.

## Overview

This repository provides:
- **CI/CD templates** - Language-specific GitHub Actions workflows
- **Git hooks** - Pre-commit, commit-msg, pre-push validation
- **Linter configs** - clippy.toml, ruff.toml, golangci.yml, etc.
- **Bootstrap script** - Apply governance to any project
- **Validation script** - Check project compliance

## Quick Start

```bash
# Apply all governance to a project
./scripts/bootstrap.sh /path/to/project \
  --project-name "my-project" \
  --description "My project description"

# Validate a project
./scripts/validate.sh /path/to/project
```

## Scripts

### bootstrap.sh

Apply governance pillars to a project.

```bash
# Apply all pillars
./scripts/bootstrap.sh . --pillar all

# Apply specific pillar
./scripts/bootstrap.sh . --pillar ci
./scripts/bootstrap.sh . --pillar security

# Dry run
./scripts/bootstrap.sh . --dry-run
```

**Available Pillars:**
| Pillar | Description |
|--------|-------------|
| `all` | Apply everything |
| `linters` | Linter configurations |
| `ci` | CI/CD workflows |
| `security` | Security scanning |
| `benchmarks` | Benchmark CI |
| `hooks` | Pre-commit + git hooks |
| `dependabot` | Dependency automation |
| `docs` | CLAUDE.md, README.md |
| `agileplus` | AgilePlus spec scaffolding |
| `devcontainer` | Dev container |
| `codeowners` | GitHub CODEOWNERS |
| `release` | Release workflows |
| `coverage` | Code coverage |
| `container` | Container builds |
| `adr` | ADR validation |

### validate.sh

Validate a project against governance standards.

```bash
# Validate project
./scripts/validate.sh /path/to/project

# Verbose output
./scripts/validate.sh -v /path/to/project
```

## Structure

```
phenotype-governance/
├── scripts/
│   ├── bootstrap.sh      # Apply governance
│   └── validate.sh      # Validate compliance
├── templates/
│   └── ci/              # CI workflow templates
│       ├── ci-rust.yml.template
│       ├── ci-python.yml.template
│       ├── ci-typescript.yml.template
│       ├── ci-go.yml.template
│       └── ...
├── hooks/               # Git hooks
│   ├── commit-msg
│   ├── pre-push
│   ├── pre-commit
│   └── prepare-commit-msg
├── configs/             # Linter configurations
│   ├── rust/
│   ├── python/
│   ├── typescript/
│   └── go/
└── docs/
    └── adr-template.md
```

## Templates

### CI Workflows

| Language | Template | Features |
|----------|----------|----------|
| Rust | `ci-rust.yml.template` | check, test, fmt, clippy, build |
| Python | `ci-python.yml.template` | lint, format, typecheck, test |
| TypeScript | `ci-typescript.yml.template` | lint, typecheck, test, build |
| Go | `ci-go.yml.template` | check, test, lint, vet, build |

### Security Workflows

- `security-rust.yml.template` - CodeQL + cargo audit + cargo deny
- `security-python.yml.template` - Ruff security + pip-audit + bandit
- `security-generic.yml.template` - CodeQL + npm audit + trivy

### Specialized Workflows

- `benchmarks-rust.yml.template` - Criterion benchmarks
- `benchmarks-python.yml.template` - pytest-benchmark
- `release-rust.yml.template` - crates.io publishing
- `coverage-rust.yml.template` - Tarpaulin + Codecov
- `container-rust.yml.template` - Docker + GHCR
- `adr-validation.yml.template` - ADR format validation

## Scaffolds

Canonical starter templates are in `scaffolds/`:

```bash
# Use a scaffold
cp -r scaffolds/lang/rust my-project
cd my-project
# Update {{placeholders}}

# Or bootstrap fresh
mkdir my-project
cd my-project
../phenotype-governance/scripts/bootstrap.sh . \
  --project-name "my-project" \
  --description "My project" \
  --pillar all
```

## Placeholders

Templates use these placeholders (replaced by bootstrap):

| Placeholder | Description |
|-------------|-------------|
| `{{project_name}}` | Project name (kebab-case) |
| `{{description}}` | Project description |
| `{{author}}` | Author name |
| `{{module_name}}` | Go module path |

## Contributing

1. Fork and create feature branch
2. Update relevant templates
3. Test with bootstrap script
4. Submit PR

## License

MIT

/// @trace GOV-001: Policy Engine
/// @trace GOV-002: RBAC/ABAC
/// @trace GOV-003: Compliance Auditing
