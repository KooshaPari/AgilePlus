# SDLC/Methodology Comparison for Agent-Based Spec-Driven Development

**Date:** 2026-09-03
**Scope:** Agile, Waterfall, XP, Shape Up, Semantic Process, and methodology fit for agent-based development
**Perspective:** AgilePlus as a client-side semantic process engine for humans and agents

---

## Methodology Landscape

| Methodology | Origin | Process Type | Ceremonies | Agent Fit | Storage Model | Governance |
|-------------|--------|--------------|------------|-----------|---------------|------------|
| **Waterfall** | 1970s (DoD) | Sequential phases | Gate reviews | ❌ Low | Documentation | Manual gates |
| **Agile/Scrum** | 2001 (manifesto) | Iterative, time-boxed | Sprint planning, daily standup, review, retrospective | ⚠️ Medium | Backlog (Jira, etc.) | Scrum master |
| **XP (Extreme Programming)** | 1996 (Kent Beck) | Engineering practices | Pair programming, TDD, CI | ✅ High | Code + tests | Engineering culture |
| **Kanban** | 2000s (Toyota) | Continuous flow | WIP limits, standup | ✅ High | Board (software) | Process owner |
| **Shape Up** | 2018 (Basecamp) | Fixed time, variable scope | Shaping, betting, building | ⚠️ Medium | Spec + bets | Product owners |
| **Lean** | 2000s (Toyota) | Value-stream optimization | Value stream mapping | ⚠️ Medium | Value streams | Lean leaders |
| **SAFe** | 2011 (Scaled Agile) | Scaled Agile | PI planning, ART sync | ❌ Low | Program boards | Enterprise |
| **DSDM** | 1994 | Time-boxed, MoSCoW | Feasibility, timeboxing | ⚠️ Medium | MoSCoW backlog | Business sponsor |
| **Semantic Process (AgilePlus)** | 2026 (Phenotype) | Spec→Trace→Govern→Ship | None (code-first) | ✅✅ Maximum | SQLite + git | Governance as code |

---

## Agent Fit Analysis

### Why Traditional Methods Fail with Agents

**Scrum ceremonies require humans:**
- Daily standup needs presence and language
- Sprint review needs stakeholder feedback
- Retrospective needs shared context and trust
- Planning poker needs shared estimation norms

**Waterfall gates are manual:**
- Requirements sign-off requires authority
- Design review needs domain expertise
- Acceptance testing needs human judgment
- Go-live requires human approval

**XP practices are partially automatable:**
- TDD → agents can write tests (✅)
- Pair programming → agents can share context (✅ partial)
- CI → agents trigger and respond (✅)
- Refactoring → agents can suggest (✅ partial)

### Why Semantic Process Fits Agents

Agents excel at:
1. **Spec parsing** — structured specs are machine-readable
2. **Trace generation** — mapping spec→code→test is deterministic
3. **Governance enforcement** — code rules are verifiable, not opinion
4. **State tracking** — SQLite provides single source of truth
5. **Cycle management** — time-boxed delivery units map to agent runs

**AgilePlus's semantic process maps directly:**
```
Human writes spec     →  agent parses, creates feature record
Agent generates plan  →  agent creates work packages
Agent implements      →  agent writes code, commits, links to WP
Agent validates       →  agent runs governance checks
Agent ships           →  agent merges, archives, records in audit chain
Human reviews         →  human inspects audit trail, approves/holds
```

---

## Spec-Driven Development Patterns

### Pattern 1: Spec-as-Contract
- **Definition:** Spec is the executable contract between human intent and agent output
- **AgilePlus:** Feature spec → audit chain → acceptance criteria → shipped state
- **Best for:** Regulated domains, safety-critical, auditable systems
- **Agents:** Validate against spec, fail if acceptance criteria not met
- **Tools:** AgilePlus, OpenSpec (API contracts)

### Pattern 2: Spec-as-Context
- **Definition:** Spec provides context to agents, not enforcement
- **AgilePlus:** Spec enriches agent prompts, agent decides implementation
- **Best for:** Creative domains, exploration, research
- **Agents:** Use spec as input, not constraint
- **Tools:** Spec Kit, Cursor rules, Continue spec modes

### Pattern 3: Spec-as-Record
- **Definition:** Spec is a human-readable record of decisions
- **AgilePlus:** Audit trail records all decisions, spec is the formalization
- **Best for:** Post-hoc analysis, compliance, retrospective
- **Agents:** Read spec for context, generate reports from audit trail
- **Tools:** BMAD-METHOD, spec-kitty (Kanban + specs)

### Pattern 4: Spec-as-Workflow
- **Definition:** Spec defines the workflow steps agents execute
- **AgilePlus:** Plan → implement → validate → ship as state transitions
- **Best for:** Repetitive, well-defined tasks, automated pipelines
- **Agents:** Execute each step, report completion
- **Tools:** Factory.ai Droid, AWS Kiro

---

## AgilePlus Semantic Process Model

```
┌─────────────────────────────────────────────────────────┐
│                    AGILEPLUS CORE                        │
│                                                         │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐           │
│  │  SPEC    │───▶│  PLAN    │───▶│  IMPLEMENT│          │
│  │  (human) │    │ (agent)  │    │ (agent)  │           │
│  └──────────┘    └──────────┘    └──────────┘           │
│       │               │               │                 │
│       ▼               ▼               ▼                 │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐           │
│  │ VALIDATE │◀───│ GOVERN   │◀───│  SHIP    │           │
│  │ (agent)  │    │ (code)   │    │ (agent)  │           │
│  └──────────┘    └──────────┘    └──────────┘           │
│       │               │               │                 │
│       └───────────────┴───────────────┘                 │
│                    │                                    │
│                    ▼                                    │
│            ┌─────────────┐                              │
│            │  AUDIT      │                              │
│            │  CHAIN      │                              │
│            │ (immutable) │                              │
│            └─────────────┘                              │
└─────────────────────────────────────────────────────────┘
```

### State Machine

```
created → specified → researched → planned → implementing → validated → shipped → retrospected
              ↑                                                    │
              └────────────────────────────────────────────────────┘
                              (feedback loop)
```

### Work Package States

```
planned → doing → review → done (+ blocked at any point)
```

---

## Agent-Based Development: 40x Scope Analysis

### Agent Types and Their Roles

| Agent Type | Role | Interaction | Frequency |
|------------|------|-------------|-----------|
| **Plan Agent** | Generate delivery plan, WPs | Reads spec, writes plan | Per feature |
| **Implement Agent** | Write code, tests | Reads plan, commits | Per WP |
| **Validate Agent** | Governance, quality checks | Reads code, spec | Per WP |
| **Review Agent** | Human-readable review | Reads implementation | Per ship |
| **Research Agent** | Codebase scan, feasibility | Reads code, writes research | Pre-plan |
| **Triage Agent** | Classify incoming items | Reads context, routes | Continuous |
| **Audit Agent** | Verify audit chain integrity | Reads chain, validates | Periodic |
| **Govern Agent** | Enforce policies | Reads rules, checks compliance | Continuous |

### 40x Scope Coverage

| Category | Tools/Methods | Agent Integration |
|----------|---------------|-------------------|
| **PM/ALM/SDLC** | Jira, Linear, Trello, Asana, Azure DevOps | Webhooks, APIs (external) |
| **Spec Governance** | Spec Kit, OpenSpec, BMAD | Plugin-based (partial) |
| **Spec Writing** | Notion, Confluence, Google Docs | External |
| **Code Generation** | Copilot, Cursor, Aider, Codeium | IDE-native |
| **Testing** | Jest, pytest, Playwright, Cypress | CLI-native |
| **CI/CD** | GitHub Actions, GitLab CI, Jenkins | Native |
| **Monitoring** | Datadog, Sentry, Grafana | External |
| **Deployment** | Vercel, Render, Netlify, AWS | CLI-native |

**Key insight:** No existing tool covers the full 40x scope. AgilePlus covers spec→govern→ship natively. Other tools cover adjacent concerns. Integration is via CLI, not plugins.

---

## Conclusion: Best Methodology for Agent-Based Spec-Driven Development

**AgilePlus's semantic process (Spec→Trace→Govern→Ship) is the optimal methodology for agent-based development because:**

1. **Code-first governance** eliminates subjective review gates
2. **Traceability** connects human intent to agent output to shipped code
3. **State machine** gives agents clear, deterministic steps
4. **Audit chain** provides immutable record of all decisions
5. **No ceremonies** — agents don't need standups, sprint planning, or retrospectives
6. **Local-first** — works offline, on any machine, no cloud dependency
7. **VSCode-like** — `.agileplus/` is the `.vscode/` of spec-driven development

**Traditional methodologies are suboptimal for agents because:**
- Ceremonies require humans (standups, reviews, retros)
- Manual gates don't scale to machine-speed development
- No traceability from spec to shipped code
- Cloud dependency for collaboration
- Governance is opinion-based, not code-enforced
