# State-of-the-Art Analysis: AgilePlus

**Domain:** Spec-driven development engine with kitty-specs workflow  
**Analysis Date:** 2026-04-02  
**Standard:** 5-Star Research Depth

---

## Executive Summary

AgilePlus provides a spec-driven development engine with 7-command workflow (specify, research, plan, implement, review, merge, ship). It competes in the development workflow/tooling space against Jira, Linear, GitHub Projects, and emerging AI-native tools.

**Key Finding:** AgilePlus differentiates through **spec-as-code** (markdown specs in git), **AI-agent native workflow**, and **tiered quality gates**. Most tools are GUI-first; AgilePlus is CLI-first with AI integration.

---

## Alternative Comparison Matrix

### Project Management Tools

| Solution | Model | Git Native | AI Native | CLI | Spec-as-Code | Maturity |
|----------|-------|------------|-----------|-----|--------------|----------|
| **Jira** | Ticket-based | ❌ | ❌ (Bolt-on) | Limited | ❌ | L5 |
| **Linear** | Issue-based | Partial | ❌ | ✅ | ❌ | L5 |
| **GitHub Projects** | Board | ✅ | Partial (Copilot) | ❌ | ❌ | L4 |
| **Notion** | Doc/DB | ❌ | Partial | ❌ | ❌ | L4 |
| **Asana** | Task | ❌ | ❌ | Limited | ❌ | L5 |
| **Shortcut** | Story | Partial | ❌ | ✅ | ❌ | L4 |

### Spec-Driven / Doc-as-Code Tools

| Solution | Spec Format | Git Native | Validation | AI Integration | Maturity |
|----------|-------------|------------|------------|----------------|----------|
| **ADR Tools** (adr-tools) | Markdown | ✅ | ❌ | ❌ | L4 |
| **MADR** | Markdown | ✅ | ❌ | ❌ | L4 |
| **RFC Tools** | Various | Partial | ❌ | ❌ | L3 |
| **DocOps** | Various | ✅ | Partial | ❌ | L3 |
| **AgilePlus (selected)** | kitty-specs | ✅ | ✅ | ✅ | L3 |

### AI-Native Dev Tools

| Solution | Focus | Git Native | CLI | Maturity |
|----------|-------|------------|-----|----------|
| **GitHub Copilot Workspace** | PR generation | ✅ | ❌ | L4 |
| **Sweep** | Issue → PR | ✅ | ❌ | L3 |
| **Codegen** | Code agents | Partial | ❌ | L3 |
| **OpenAI Codex CLI** | Code editing | ✅ | ✅ | L4 |
| **AgilePlus** | Spec → Ship | ✅ | ✅ | L3 |

---

## Academic References

1. **"Specification and Development of Interactive Systems"** (Broy, 2001)
   - Formal specification methods
   - Application: AgilePlus spec structure

2. **"Literate Programming"** (Knuth, 1984)
   - Code and documentation together
   - Application: kitty-specs as literate specs

3. **"Doc-as-Code: Documentation Management for the Modern Era"**
   - Version-controlled docs
   - Application: specs in git

4. **"Behavior-Driven Development"** (North, 2006)
   - Given-When-Then specifications
   - Application: FR structure in specs

5. **"Living Documentation"** (Chelimsky et al., 2010)
   - Executable specifications
   - Application: Traced specs to tests

6. **"The Cathedral and the Bazaar"** (Raymond, 1999)
   - Open source development models
   - Application: Community contribution via specs

7. **"Design Patterns" (GoF)**
   - Template Method pattern
   - Application: spec → research → plan → implement pattern

8. **"Lean Software Development"** (Poppendieck)
   - Eliminate waste, amplify learning
   - Application: Spec-first eliminates rework

9. **"Continuous Delivery"** (Humble & Farley, 2010)
   - Deployment pipeline
   - Application: ship command

10. **"Accelerate"** (Forsgren et al., 2018)
    - DORA metrics
    - Application: Quality gates in workflow

---

## Innovation Log

### AgilePlus Novel Solutions

1. **Kitty-Specs Format**
   - Innovation: Standardized markdown spec with sections
   - Contrast: Free-form docs or rigid ticket fields
   - Research: Literate programming, BDD
   - Status: 39 specs in `kitty-specs/`

2. **7-Command Workflow**
   - Innovation: specify → research → plan → implement → review → merge → ship
   - Contrast: Most tools have create → in-progress → done
   - Research: State machines, process modeling
   - Status: CLI commands defined

3. **AI-Agent Native**
   - Innovation: Designed for AI agents, not just humans
   - Contrast: Human-centric tools adapted for AI
   - Research: Agent-based systems
   - Status: Agent harness integration

4. **Quality Tier System**
   - Innovation: P0-P3 priority + channel tiers (critical, standard, experimental)
   - Contrast: Simple priority (high/medium/low)
   - Research: SRE error budgets, risk management
   - Status: Governance model

5. **FR Traceability**
   - Innovation: All tests reference Functional Requirements
   - Contrast: Tests often lack requirement links
   - Research: DO-178C, requirements engineering
   - Status: Enforced in AGENTS.md

---

## Gaps vs. SOTA

| Gap | SOTA | AgilePlus Status | Priority |
|-----|------|------------------|----------|
| GUI | Jira, Linear | CLI only | P2 |
| Integrations | Jira (1000+) | Limited | P2 |
| Enterprise features | Jira | Basic | P3 |
| Mobile app | Linear | None | P3 |
| Reporting | Jira dashboards | Worklog | P2 |
| Community | Linear | Internal | P3 |

---

## External Links

- kitty-specs structure: `AgilePlus/kitty-specs/`
- BDD: https://cucumber.io/docs/bdd/
- Living Documentation: https://leanpub.com/livingdocumentation
- MADR: https://adr.github.io/madr/

---

**Next Update:** 2026-04-16
