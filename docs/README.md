# AgilePlus Documentation

Central documentation for the Phenotype AgilePlus ecosystem.

## Quick Links

- [FR Specifications](../specs/) - 46 Feature Requirements
- [Traceability CLI](../bin/ptrace) - FR traceability tool
- [Governance](../GOVERNANCE.md) - Organization standards

## Getting Started

```bash
# Install ptrace CLI
curl -sL https://raw.githubusercontent.com/phenotype/AgilePlus/main/bin/ptrace -o ~/bin/ptrace
chmod +x ~/bin/ptrace

# Validate FR specs
./scripts/validate-fr-ids.sh

# Check drift
./bin/ptrace check-drift --path . --threshold 90
```

## Documentation Structure

- `specs/` - Feature Requirements (FR-XXX-NNN format)
- `ADR/` - Architecture Decision Records
- `ARCHITECTURE/` - System-wide architecture docs
- `GOVERNANCE/` - Organization governance rules
- `SECURITY/` - Security policies
- `templates/` - Artifact templates for all repos

Last Updated: 2026-04-04
