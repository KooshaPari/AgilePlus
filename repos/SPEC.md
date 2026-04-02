# Repos Specification

## Repository Overview

Repos is a placeholder/sibling directory containing phenotype subprojects.

## Structure

```
repos/
├── phenotype-bootstrap/     # Bootstrap project (empty)
├── phenotype-replication-engine/  # Replication engine (empty)
├── .editorconfig
├── .github/
├── .pre-commit-config.yaml
├── cliff.toml
└── mise.toml
```

## Purpose

This directory holds experimental or placeholder phenotype projects.

## Quality Gates

```bash
pre-commit run --all-files
```
