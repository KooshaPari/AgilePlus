# Organization Artifact Consolidation Plan

## Executive Summary

This document defines the consolidation strategy for artifacts across all 26 Phenotype repositories.

## Current State (Post-Creation)

- **26 Repositories** with full artifact coverage
- **Central Artifacts** in AgilePlus/
- **Per-Repo Artifacts** minimal but complete

## 3-Group Framework Implementation

| Group | Location | Count | Status |
|-------|----------|-------|--------|
| **Artifacts** | Central + Per-Repo | 260+ files | ✅ Complete |
| **Task Items** | Central specs/ | 46 FRs | ✅ Complete |
| **Governance** | Per-repo + Central | 26 repos | ✅ Complete |

## Artifact Matrix (26 Repositories × 10 Types)

| Repo | CLAUDE | AGENTS | PRD | ADR | GOV | ARCH | SEC | specs | plan | validate |
|------|--------|--------|-----|-----|-----|------|-----|-------|------|----------|
| Tracera | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| phenoSDK | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| thegent | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| heliosCLI | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| agent-wave | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| phenotype-agent-core | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| phenotype-cli-core | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| pheno-cli | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| phenotype-mcp-testing | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| phenotype-gauge | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| phenotype-governance | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| phenotype-validation | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| PhenoVCS | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Benchora | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Authvault | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Planify | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Apisync | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| KodeVibeGo | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| PolicyStack | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Portalis | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Quillr | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Schemaforge | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Settly | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Stashly | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tasken | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tokn | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

## Central Artifacts (AgilePlus/)

```
AgilePlus/
├── ADR/
│   ├── README.md (index)
│   ├── INDEX.md (full list)
│   └── templates/
├── ARCHITECTURE/
│   ├── README.md
│   ├── system-overview.md
│   └── patterns/
├── GOVERNANCE/
│   ├── README.md
│   ├── standards.md
│   └── quality-gates.md
├── SECURITY/
│   ├── README.md
│   └── policies.md
├── templates/
│   ├── ADR-template.md
│   ├── PRD-template.md
│   └── spec-template.md
└── CONSOLIDATION.md (this file)
```

## Governance Enforcement

All 26 repos now have:
- `validate_governance.py` - Local validation script
- `.phenotype/ai-traceability.yaml` - AI attribution
- `.github/workflows/traceability.yml` - CI/CD

Run validation:
```bash
cd <repo> && python3 validate_governance.py
```

## Consolidation Strategy

### Central (Source of Truth)
- Global FR specifications
- Architecture patterns
- Governance standards
- Security policies
- Templates

### Per-Repo (Implementation)
- Local context (CLAUDE.md)
- Agent rules (AGENTS.md)
- Project docs (PRD.md, ADR.md)
- Implementation specs
- Validation scripts

## Success Metrics

| Metric | Target | Actual |
|--------|--------|--------|
| Repos with full artifacts | 26/26 | 26/26 ✅ |
| Central artifacts created | 15+ | 15+ ✅ |
| FR traceability | 100% | 100% ✅ |
| Governance compliance | 100% | 100% ✅ |

## Commands

```bash
# Validate single repo
cd <repo> && python3 validate_governance.py

# Check all repos
for r in */; do echo "=== $r ===" && python3 $r/validate_governance.py 2>/dev/null | grep -E "PASS|FAIL"; done

# Check drift
./AgilePlus/bin/ptrace check-drift --path . --threshold 90
```

## Status: ✅ COMPLETE

All 26 repositories have complete artifact coverage following the 3-Group Framework.

Last Updated: 2026-04-04
