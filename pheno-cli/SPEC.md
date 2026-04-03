# pheno-cli Specification

Canonical definition of the system behavior.

## Overview

`pheno` is a Go-based CLI for org-wide release governance, automated publishing, and developer experience tooling across Phenotype repositories.

## Architecture

```
pheno (root)
├── bootstrap  # Initialize new repositories with Phenotype standards
├── publish    # Publish packages to registries
├── promote   # Promote releases through environments
├── scaffold  # Scaffold new projects from templates
├── audit     # Audit dependencies and licenses
├── matrix    # Generate dependency matrices
└── cleanup   # Clean up stale resources
```

## Commands

### bootstrap

Initialize a new repository with Phenotype standards.

```bash
pheno bootstrap [flags]

Flags:
  --template string   Template to use (default: "cargo")
  --org string       Organization name
  --repo string      Repository name
  --private          Create private repository
```

**Behavior:**
1. Create repository structure from template
2. Initialize git with remote
3. Install pre-commit hooks
4. Create initial CI/CD configuration
5. Register with phenotype-hub

### publish

Publish packages to registries.

```bash
pheno publish [flags]

Flags:
  --dry-run         Preview without publishing
  --registry string Package registry (default: "crates.io")
  --package string  Package to publish
  --version string  Version to publish
```

**Behavior:**
1. Validate version against semver
2. Run governance checks (specs, tests, linting)
3. Build release artifacts
4. Push to registry
5. Create GitHub release

### promote

Promote releases through environments.

```bash
pheno promote [flags]

Flags:
  --from string     Source environment
  --to string       Target environment
  --release string  Release to promote
  --approve         Auto-approve promotion
```

**Environments:** `dev` → `staging` → `production`

**Behavior:**
1. Validate release exists in source
2. Run integration tests
3. Request approval if required
4. Update environment config
5. Notify downstream systems

### scaffold

Scaffold new projects from templates.

```bash
pheno scaffold [command] [flags]

Commands:
  library    Create a new Rust library crate
  binary     Create a new application
  workspace  Create a new monorepo workspace

Flags:
  --name string      Project name
  --path string      Output directory
  --template string  Template variant
```

### audit

Audit dependencies and licenses.

```bash
pheno audit [flags]

Flags:
  --format string   Output format (table, json, csv)
  --fail string     Fail on severity (low, medium, high, critical)
  --fix            Attempt to fix issues
```

**Checks:**
- Dependency vulnerabilities (cargo-audit)
- License compliance
- Code coverage thresholds
- Test pass rates

### matrix

Generate dependency matrices.

```bash
pheno matrix [flags]

Flags:
  --output string   Output file
  --format string   Format (markdown, json, csv)
  --depth int       Dependency depth (default: 3)
```

### cleanup

Clean up stale resources.

```bash
pheno cleanup [flags]

Flags:
  --dry-run         Preview without deleting
  --age int         Age in days (default: 30)
  --type string     Resource type (branches, tags, releases)
```

## Configuration

Configuration file: `~/.config/pheno/config.toml`

```toml
[defaults]
org = "KooshaPari"
registry = "crates.io"

[github]
token = "${GITHUB_TOKEN}"
owner = "KooshaPari"

[governance]
require_specs = true
require_tests = true
min_coverage = 80

[environments]
dev = { branch = "main", auto_promote = true }
staging = { branch = "staging", requires_approval = true }
production = { branch = "production", requires_approval = true }
```

## Data Models

### Release

```go
type Release struct {
    Version   string       `json:"version"`
    Commit    string       `json:"commit"`
    Author    string       `json:"author"`
    Date      time.Time   `json:"date"`
    Changelog string       `json:"changelog"`
    Artifacts []Artifact   `json:"artifacts"`
    Status    ReleaseStatus `json:"status"`
}

type ReleaseStatus string
const (
    Draft     ReleaseStatus = "draft"
    Published ReleaseStatus = "published"
    Promoted  ReleaseStatus = "promoted"
)
```

### Environment

```go
type Environment struct {
    Name               string   `json:"name"`
    Branch             string   `json:"branch"`
    AutoPromote        bool     `json:"auto_promote"`
    RequiresApproval   bool     `json:"requires_approval"`
    Approvers          []string `json:"approvers"`
}
```

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/releases` | GET | List all releases |
| `/api/v1/releases` | POST | Create new release |
| `/api/v1/releases/:id/promote` | POST | Promote release |
| `/api/v1/environments` | GET | List environments |
| `/api/v1/audit` | POST | Run audit |

## Governance Rules

1. **Specs Required**: Every feature requires a corresponding spec
2. **Tests Required**: Minimum 80% coverage for production
3. **Review Required**: 1 approval from CODEOWNER
4. **Changelog Required**: All releases require changelog entry
5. **Version Compliance**: Must follow semver

## Error Handling

Exit codes:
- `0`: Success
- `1`: General error
- `2`: Configuration error
- `3`: Governance check failed
- `4`: Network/API error
- `5`: Validation error