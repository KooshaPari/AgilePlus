# Proto Specification

## Repository Overview

Proto contains Protocol Buffer (proto3) definitions for the AgilePlus system.

## Structure

```
proto/
├── agileplus/
│   └── v1/
│       ├── agents.proto     # Agent-related messages
│       ├── common.proto     # Shared/common messages
│       ├── core.proto       # Core service definitions
│       └── integrations.proto  # Integration messages
├── .editorconfig
├── .github/
├── .pre-commit-config.yaml
├── cliff.toml
└── mise.toml
```

## Proto Packages

- **agileplus.v1**: Main package for AgilePlus services

## Generated Code

- Go: `github.com/phenotype/agileplus-proto/gen/go/agileplus/v1`
- Java: `com.phenotype.agileplus.v1`

## Quality Gates

```bash
buf lint
buf format -w
```
