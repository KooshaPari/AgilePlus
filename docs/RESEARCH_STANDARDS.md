# Research Standards Framework — repos shelf

**Version:** 1.0.0  
**Effective Date:** 2026-04-02  
**Scope:** All 134 repositories in the repos shelf  
**Standard:** 5-Star Research Depth (matching shelf-level quality)

---

## Research Depth Levels

### ⭐⭐⭐⭐⭐ (5-Star) — Exemplar Standard
**Required for:** Core infrastructure, novel solutions, competitive domains

**Artifacts:**
- [ ] SOTA Analysis (20+ alternatives/competitors compared)
- [ ] Academic paper references (3+ relevant papers)
- [ ] Experiment framework with documented results
- [ ] Decision records with research backing (ADRs)
- [ ] Session documentation (6-file structure)
- [ ] Innovation log (novel solutions documented)
- [ ] External research links (web sources, blogs, talks)
- [ ] Functional Requirements traceability to research

**Examples:** `MULTI_AGENT_ORCHESTRATION_COMPARISON_2026.md`, `docs/worklogs/RESEARCH.md`

### ⭐⭐⭐⭐☆ (4-Star) — Production Standard
**Required for:** Active development projects, service APIs

**Artifacts:**
- [ ] SOTA Analysis (10+ alternatives compared)
- [ ] Key paper/whitepaper references (1-2)
- [ ] Architecture Decision Records (ADRs)
- [ ] Session documentation (4+ of 6 files)
- [ ] Innovation notes (key novel solutions)
- [ ] Research-backed specs

### ⭐⭐⭐☆☆ (3-Star) — Standard Project
**Minimum for:** All other projects, templates

**Artifacts:**
- [ ] Competitor/alternative comparison (5+)
- [ ] ADR for major decisions
- [ ] Session documentation (2+ files)
- [ ] Research links in PRD/SPEC

---

## Required Documentation Structure

Every project MUST maintain:

```
docs/
├── research/              # SOTA analysis, papers, experiments
│   ├── SOTA.md           # State-of-the-art comparison (REQUIRED)
│   ├── PAPERS.md         # Academic references
│   ├── EXPERIMENTS.md    # Experiment results
│   └── INNOVATIONS.md    # Novel solutions log
├── sessions/              # Work session documentation
│   └── YYYYMMDD-feature/
│       ├── 01_RESEARCH.md
│       ├── 02_SPECIFICATIONS.md
│       ├── 03_DAG_WBS.md
│       ├── 04_IMPLEMENTATION_STRATEGY.md
│       ├── 05_KNOWN_ISSUES.md
│       └── 06_TESTING_STRATEGY.md
└── adr/                   # Architecture Decision Records
    └── NNNN-title.md
```

---

## Project Tiers

### Tier 1: Core Infrastructure (5-Star Required)
- AgilePlus — spec-driven development engine
- thegent — dotfiles/config management
- heliosCLI — CLI framework
- heliosApp — TUI application framework
- phenoSDK — SDK core
- BytePort — platform core

### Tier 2: Active Ecosystem (4-Star Required)
- All `phenotype-*` crates
- All `thegent-*` plugins
- All `template-*` repos
- PolicyStack, Authvault, Apisync

### Tier 3: Specialized Tools (3-Star Minimum)
- All other 134 repositories

---

## SOTA Analysis Template

Every project MUST create `docs/research/SOTA.md`:

```markdown
# State-of-the-Art Analysis: [Project Name]

## Domain
[Define the problem space]

## Alternatives Comparison

| Solution | Approach | Pros | Cons | Maturity |
|----------|----------|------|------|----------|
| [Name] | [Arch] | [...] | [...] | L5/L4/etc |

## Research Papers

1. **[Title]** ([Authors], [Year])
   - [Summary and relevance to project]
   - [Link or DOI]

## Innovation Opportunities

- [Novel approach 1 with research backing]
- [Novel approach 2 with research backing]

## Decision Rationale

[Why this project's approach was chosen over alternatives]
```

---

## Audit Checklist Per Project

```markdown
## [Project Name] Research Audit

### Current State
- [ ] Has docs/research/ directory
- [ ] Has docs/sessions/ directory
- [ ] Has docs/adr/ directory
- [ ] Has SOTA.md with 10+ comparisons
- [ ] Has PAPERS.md with 1+ references
- [ ] Has documented experiments
- [ ] Has innovation log

### Research Depth Rating
- [ ] ⭐⭐⭐⭐⭐ (20+ comparisons, 3+ papers, full experiments)
- [ ] ⭐⭐⭐⭐☆ (10+ comparisons, 1-2 papers, partial experiments)
- [ ] ⭐⭐⭐☆☆ (5+ comparisons, basic ADRs)
- [ ] ⭐⭐☆☆☆ (minimal research)
- [ ] ⭐☆☆☆☆ (no research documentation)

### Gap Analysis
- [Specific gaps and remediation plan]
```

---

## Compliance

### New Projects
- Research framework MUST be created before implementation
- No PR without research documentation for 4/5-star projects
- Subagent audit MUST pass before merge

### Existing Projects
- Tier 1: Research backfill required within 2 weeks
- Tier 2: Research backfill required within 1 month
- Tier 3: Research backfill required within 2 months

### CI/CD Integration
- `agileplus research validate` MUST pass
- Research files checked for UTF-8 encoding
- Links in research docs validated

---

## Enforcement

### Pre-Commit Hooks
```bash
# Verify research docs exist for 4/5-star projects
./scripts/validate-research.sh
```

### PR Checklist
- [ ] Research documentation updated
- [ ] SOTA analysis current
- [ ] ADRs linked to research

### Subagent Audits
Sage agents will continuously audit projects and flag gaps.

---

**Next Steps:**
1. Run master audit across all 134 repos
2. Create research backfill plans per tier
3. Deploy subagents for batch research documentation creation
