# Prompts Specification

## Repository Overview

Prompts is a collection of CLI command templates and prompt instructions used by the AgilePlus system.

## Structure

```
prompts/
├── subcommands/              # CLI command prompt templates
│   ├── context-load-*.md    # Context loading prompts
│   ├── devops-*.md          # DevOps operation prompts
│   ├── escape-*.md          # Escape hatch prompts
│   ├── git-*.md             # Git operation prompts
│   ├── governance-*.md      # Governance prompts
│   ├── meta-*.md            # Meta command prompts
│   ├── sync-*.md            # Sync operation prompts
│   └── triage-*.md          # Triage operation prompts
├── .editorconfig
├── .github/
├── .pre-commit-config.yaml
├── cliff.toml
└── mise.toml
```

## Purpose

Each `.md` file in `subcommands/` defines a prompt template for a specific CLI subcommand. These are used to generate agent instructions dynamically.

## Quality Gates

```bash
pre-commit run --all-files
```
