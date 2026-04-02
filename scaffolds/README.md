# Phenotype Scaffolds

Canonical starter templates for Phenotype ecosystem projects.

## Structure

```
scaffolds/
├── lang/           # Language-specific starters
│   ├── rust/       # Rust library (Cargo.toml)
│   ├── python/     # Python library (pyproject.toml)
│   ├── go/         # Go module (go.mod)
│   └── typescript/ # TypeScript library (package.json)
├── arch/           # Architecture patterns (planned)
│   └── hexagonal/  # Hexagonal architecture
├── cli/            # CLI application starters (planned)
└── domain/          # Domain-specific starters (planned)
    ├── api/        # REST/gRPC API
    └── webapp/     # Web application
```

## Usage

### Quick Start

```bash
# Clone scaffold
cp -r scaffolds/lang/rust my-new-project
cd my-new-project

# Replace placeholders
sed -i '' 's/{{project_name}}/my-new-project/g' Cargo.toml
sed -i '' 's/{{description}}/My awesome project/g' Cargo.toml
sed -i '' 's/{{author}}/Your Name/g' Cargo.toml

# Initialize git
git init
git add .
git commit -m "Initial commit from scaffold"
```

### Alternative: Use Bootstrap

```bash
# Create project directory
mkdir my-new-project && cd my-new-project

# Bootstrap from scratch
../phenotype-governance/scripts/bootstrap.sh . \
  --project-name "my-new-project" \
  --description "My awesome project" \
  --pillar all
```

## Scaffolds

### Rust (`lang/rust/`)

- `Cargo.toml` - Rust package manifest
- `src/lib.rs` - Library entry point
- `src/main.rs` - Binary entry point
- `tests/integration.rs` - Integration test
- `.github/workflows/` - CI, security, coverage, release
- `.pre-commit-config.yaml` - Pre-commit hooks
- `CLAUDE.md`, `AGENTS.md`, `README.md` - Docs

### Python (`lang/python/`)

- `pyproject.toml` - Python package manifest
- `{{project_name}}/` - Package source
- `.github/workflows/` - CI, security, ADR validation
- `.pre-commit-config.yaml` - Pre-commit hooks

### Go (`lang/go/`)

- `go.mod` - Go module
- `main.go` - Entry point
- `main_test.go` - Tests
- `.github/workflows/` - CI, security, ADR validation
- `.pre-commit-config.yaml` - Pre-commit hooks

### TypeScript (`lang/typescript/`)

- `package.json` - NPM package
- `tsconfig.json` - TypeScript config
- `src/` - Source files
- `.github/workflows/` - CI, security, ADR validation
- `.pre-commit-config.yaml` - Pre-commit hooks

## Placeholders

| Placeholder | Description | Example |
|-------------|-------------|---------|
| `{{project_name}}` | Project name (kebab-case) | `my-project` |
| `{{description}}` | Project description | `A useful library` |
| `{{author}}` | Author name | `Jane Developer` |
| `{{module_name}}` | Go module path | `github.com/user/my-project` |

## Governance

All scaffolds include full governance from `phenotype-governance/`:

- CI/CD workflows (lint, test, build)
- Security scanning (CodeQL, audit)
- Pre-commit hooks
- Dependabot configuration
- Documentation templates
- AgilePlus spec scaffolding
- Dev container configuration
- CODEOWNERS

## Creating New Projects

```bash
# Option 1: Copy scaffold
cp -r scaffolds/lang/rust my-project
cd my-project
# Update placeholders manually

# Option 2: Bootstrap fresh project
mkdir my-project && cd my-project
/path/to/phenotype-governance/scripts/bootstrap.sh . \
  --project-name "my-project" \
  --description "My project" \
  --pillar all
```

## Maintenance

When updating governance, re-apply to all scaffolds:

```bash
for scaffold in scaffolds/lang/*/; do
  ./phenotype-governance/scripts/bootstrap.sh "$scaffold" \
    --project-name "template" \
    --description "Template" \
    --pillar all --force
done
```
