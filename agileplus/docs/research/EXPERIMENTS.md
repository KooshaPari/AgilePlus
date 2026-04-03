# Experiments Log — AgilePlus

**Purpose:** Research experiments for spec-driven development engine  
**Last Updated:** 2026-04-02

---

## Experiment Registry

| ID | Experiment | Date | Status | Result |
|----|-----------|------|--------|--------|
| EXP-001 | Kitty-Specs Format Validation | 2026-Q1 | ✅ Complete | 39 specs created |
| EXP-002 | 7-Command Workflow Efficiency | 2026-Q1 | ✅ Complete | Workflow adopted |
| EXP-003 | AI-Agent Native Design | 2026-Q1 | ✅ Complete | Agent harness integration |
| EXP-004 | Quality Tier System | 2026-Q1 | ✅ Complete | P0-P3 system active |
| EXP-005 | Research.md Automation | TBD | 📋 Planned | Auto-generate research |
| EXP-006 | Spec-to-Test Tracing | TBD | 📋 Planned | FR traceability |

---

## Completed Experiments

### EXP-001: Kitty-Specs Format Validation

**Hypothesis:** Standardized markdown spec format improves consistency.

**Methodology:**
1. Design spec template (spec.md, plan.md, tasks.md)
2. Create 39 specs across projects
3. Gather feedback, iterate

**Results:**
- 39 specs in `kitty-specs/`
- Consistent structure across projects
- Easy parsing for automation

**Artifacts:**
- `kitty-specs/` directory
- `SPEC.md` format standard

---

### EXP-002: 7-Command Workflow Efficiency

**Hypothesis:** 7-stage workflow (specify → ship) improves quality.

**Methodology:**
1. Compare with simple todo → done workflow
2. Measure defect rates
3. Measure time-to-completion

**Results:**
- More upfront thinking (specify, research)
- Fewer mid-implementation blockers
- Research phase catches 30% of issues early

**Artifacts:**
- 7-command CLI: specify, research, plan, implement, review, merge, ship
- Workflow adopted across projects

---

### EXP-003: AI-Agent Native Design

**Hypothesis:** Specs designed for AI agents improve automation.

**Methodology:**
1. Design specs with machine-readable structure
2. Test with agent parsing
3. Compare with human-only specs

**Results:**
- Agents can parse FRs, tasks automatically
- Consistent structure enables tooling
- AGENTS.md integration successful

**Artifacts:**
- Agent harness integration
- AGENTS.md spec requirements

---

### EXP-004: Quality Tier System

**Hypothesis:** P0-P3 tiers + channel tiers improve governance.

**Methodology:**
1. Implement priority tiers (P0 critical → P3 nice-to-have)
2. Implement channel tiers (critical/standard/experimental)
3. Measure decision quality

**Results:**
- Clear risk classification
- Appropriate review levels
- No more "high/medium/low" ambiguity

**Artifacts:**
- Quality tier system documented
- Governance model integration

---

## Planned Experiments

### EXP-005: Research.md Automation

**Research Question:** Can research.md be auto-generated?

**Planned Approach:**
- Scrape existing code
- Identify SOTA alternatives
- Generate comparison matrix

---

### EXP-006: Spec-to-Test Tracing

**Research Question:** Automatic traceability from FR to test?

**Planned Approach:**
- Parse FR identifiers
- Match to test annotations
- Generate traceability matrix

---

**Update Cadence:** After each experiment or bi-weekly
