# scripts Specification

## Overview

A collection of automation scripts for the Phenotype/DINOForge project ecosystem. Provides tooling for documentation sync, service management, CI/CD, asset handling, and developer workflow automation.

## Components

### Documentation Auto-Sync (`doc-sync/`)
- **Purpose**: Index, track, and synchronize documentation across the monorepo
- **Features**:
  - Spec marker extraction (FR-*, ADR-*, P*.*, UJ-*, NFR-*, E*.*.*)
  - Cross-reference mapping
  - Change detection with SHA-256 hashing
  - JSON index output for tooling integration

### MCP Service Harness (`services/`)
- **Purpose**: Cross-platform service management for MCP (Model Context Protocol)
- **Platforms**:
  - Windows: Task Scheduler via `register-mcp-task.ps1`
  - Linux: systemd user service
  - macOS: launchd agent
- **Features**: Health checks, PID file management, watcher mode

### Other Scripts
- **CI/CD**: `deploy-sast*.sh`, `quality-gate.sh`, `deploy-workflows.sh`
- **Asset Management**: `download_priority_assets.py`, `download_models_web.py`
- **Developer Tools**: `dev-up.sh`, `bootstrap-dev.sh`, `workspace-cleanup.sh`
- **Governance**: `validate-governance.sh`, `generate-evidence.sh`

## Usage

```bash
# Documentation indexing
python scripts/doc-sync/ingester.py

# MCP service (Windows)
pwsh -File scripts/services/mcp-service.ps1 -Action Install
```

## Dependencies

- Python 3.10+ for doc-sync
- PowerShell for service management
- Standard POSIX tools for shell scripts