# Phenotype Quality Gate Suite

Comprehensive tooling audit, anti-pattern detection, LOC enforcement, and autofix capabilities for the Phenotype ecosystem.

## Tools Overview

| Tool | Purpose | Location |
|------|---------|----------|
| **ptool-audit-expanded** | Detect 50+ forbidden/legacy tools | `AgilePlus/bin/ptool-audit-expanded` |
| **panti-lint-expanded** | Detect 100+ code smells/anti-patterns | `AgilePlus/bin/panti-lint-expanded` |
| **ploc-enforce** | Enforce file size limits | `AgilePlus/bin/ploc-enforce` |
| **pquality-gate** | Unified quality gate (combines all) | `AgilePlus/bin/pquality-gate` |
| **pautofix** | Auto-convert legacy tools to modern | `AgilePlus/bin/pautofix` |
| **pquality-dashboard** | Generate HTML quality dashboard | `AgilePlus/bin/pquality-dashboard` |

## Expanded Capabilities

### 50+ Tool Database (ptool-audit-expanded)

**Package Managers** (9): npm, yarn, pnpm (forbidden) → bun; pip, poetry, conda, pipenv (discouraged) → uv
**Task Runners** (6): make, gulp, grunt (forbidden) → task; just, npm-scripts (discouraged)
**Test Frameworks** (8): jest, mocha, jasmine, karma (forbidden) → vitest; ava, tap (discouraged)
**Bundlers** (7): webpack, parcel (forbidden) → vite; rollup, esbuild, turbopack (discouraged)
**Linters** (8): eslint, prettier, tslint, standardjs, rome (discouraged/forbidden) → oxlint, biome, ruff
**CI/CD** (6): circleci, travis (forbidden) → github-actions; jenkins, gitlab-ci, azure-pipelines (discouraged)
**Container/Orchestration** (7): docker-compose-v1 (forbidden); docker-compose, helm, kustomize (allowed)
**Database** (5): sequelize, typeorm (discouraged) → prisma; raw-sql (discouraged)
**Documentation** (4): missing README (forbidden); wiki, notion-docs (discouraged)
**Security** (4): snyk (discouraged); osv-scanner, trivy, gitleaks (allowed)

### 100+ Anti-Pattern Database (panti-lint-expanded)

**Python** (30+): PY001-PY030
**Rust** (25+): RS001-RS025
**TypeScript/JavaScript** (35+): TS001-TS035
**Go** (15+): GO001-GO015
**Universal** (10+): ALL001-ALL010

### Autofix Capabilities (pautofix)

| Migration | From | To |
|-----------|------|-----|
| package_manager | npm/yarn/pnpm | bun |
| task_runner | Makefile | Taskfile.yml |
| test_framework | jest.config.js | vitest.config.ts |
| bundler | webpack.config.js | vite.config.ts |
| linter | .eslintrc.js | biome.json |
| python_linter | .flake8 | ruff.toml |
| ci | .travis.yml | .github/workflows/ |

## Usage Examples

### List All Forbidden Tools
```bash
./AgilePlus/bin/ptool-audit-expanded --list-forbidden
```

### Export Databases for Inspection
```bash
./bin/ptool-audit-expanded --export-database > tooling-db.json
./bin/panti-lint-expanded --export-database > anti-pattern-db.json
./bin/ploc-enforce --export-database > loc-db.json
```

### Run Autofix on a Project
```bash
./bin/pautofix --path ../Tracera --type package_manager --dry-run
./bin/pautofix --path ../Tracera --type task_runner --apply
```

### Generate Quality Dashboard
```bash
./bin/pquality-dashboard --projects . ../Tracera ../thegent ../phenoSDK --output dashboard.html
```
