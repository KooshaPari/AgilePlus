# AgilePlus Project Charter

**Document ID:** CHARTER-AGILEPLUS-001  
**Version:** 2.0.0  
**Status:** Active  
**Effective Date:** 2026-04-05  
**Last Updated:** 2026-04-05  

---

## Table of Contents

1. [Mission Statement](#1-mission-statement)
2. [Tenets](#2-tenets)
3. [Scope & Boundaries](#3-scope--boundaries)
4. [Target Users](#4-target-users)
5. [Success Criteria](#5-success-criteria)
6. [Governance Model](#6-governance-model)
7. [Charter Compliance Checklist](#7-charter-compliance-checklist)
8. [Decision Authority Levels](#8-decision-authority-levels)
9. [Appendices](#9-appendices)

---

## 1. Mission Statement

### 1.1 Primary Mission

**AgilePlus is a local-first, spec-driven development engine that orchestrates the entire feature lifecycle from specification through implementation to validation.** Our mission is to harmonize the best practices from modern agile tools into a streamlined CLI-centric workflow that reduces the idea-to-shipment gap to exactly 7 commands.

### 1.2 Vision

To become the standard tool for AI-augmented software development where:

- **Ideas Become Reality in Minutes**: From concept to structured specification in under 10 minutes
- **Quality is Automatic**: Policy-driven quality gates prevent defects from reaching production
- **Work is Traceable**: Every decision, change, and validation is hash-chained and auditable
- **Teams are Unblocked**: Clear work packages, intelligent assignment, and seamless agent dispatch
- **Knowledge is Preserved**: Specifications, plans, and evidence live in git, not proprietary systems

### 1.3 Strategic Objectives

| Objective | Target | Timeline |
|-----------|--------|----------|
| 7-command workflow adoption | 100% internal | 2026-Q2 |
| Idea-to-plan time | < 10 minutes | 2026-Q3 |
| Validation coverage | 100% functional requirements | 2026-Q3 |
| Governance compliance | Zero violations to main | 2026-Q2 |
| Multi-repo support | 10+ repositories | 2026-Q4 |

### 1.4 Value Proposition

```
┌─────────────────────────────────────────────────────────────────────┐
│                 AgilePlus Value Proposition                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  FOR INDIVIDUAL DEVELOPERS:                                         │
│  • Never start coding without a clear specification                 │
│  • AI agents handle implementation, you handle direction          │
│  • Automatic validation ensures nothing ships broken                │
│  • All work is tracked, no "what was I doing?" moments             │
│                                                                     │
│  FOR DEVELOPMENT TEAMS:                                             │
│  • Consistent spec quality across all features                      │
│  • Work packages automatically created and tracked                    │
│  • Git correlation shows what code implements which spec          │
│  • Retrospectives with actual completion data                       │
│                                                                     │
│  FOR TECH LEADS:                                                    │
│  • Hash-chained audit trail for compliance                          │
│  • Policy gates prevent quality regressions                         │
│  • Visibility into team velocity and blockers                       │
│  • Integration with existing tools (Plane.so, GitHub)               │
│                                                                     │
│  FOR AI CODING AGENTS:                                              │
│  • Structured specifications with clear acceptance criteria         │
│  • Isolated worktrees for safe experimentation                      │
│  • Validation requirements define "done"                            │
│  • Evidence collection for quality assurance                        │
│                                                                     │
│  FOR COMPLIANCE/AUDIT:                                              │
│  • SHA-256 hash-chained audit log                                   │
│  • Every state transition is traceable                                │
│  • Evidence links code to requirements                              │
│  • Tamper-evident record keeping                                    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. Tenets

### 2.1 Local-First

**All operational state lives on the developer's machine.**

- SQLite database is local, not cloud-dependent
- Specifications are files in git, not SaaS silos
- Offline capability is core, not an afterthought
- Cloud sync is optional, not required
- Data sovereignty: your data stays on your machine

### 2.2 Git-Native

**All artifacts are stored in git.**

- Specifications are markdown in version control
- Plans are files that can be diffed and reviewed
- Evidence is code that compiles and tests
- Worktrees provide isolation without losing git history
- Every change has a commit hash

### 2.3 Spec-Driven

**Every feature begins with a structured specification.**

- No code without a corresponding spec
- Specs are machine-readable (YAML structure)
- Acceptance criteria are clear and testable
- Specs drive work package generation
- Specs are validated, not just written

### 2.4 Agent-Orchestrated

**AI agents are dispatched, not replaced by custom engines.**

- Worktrees isolate agent work from main codebase
- Specifications provide context for agent implementation
- Agents collect evidence as they work
- Human review is the quality gate
- Agents are tools, not replacements

### 2.5 Governance-Backed

**Quality is enforced, not hoped for.**

- Hash-chained audit logs ensure tamper-evidence
- Policy-driven quality gates block bad changes
- Evidence requirements are explicit
- Compliance is automatic, not manual
- Zero violations reach production branches

### 2.6 Seven Command Simplicity

**The entire workflow is exactly 7 commands:**

```
specify → research → plan → implement → validate → ship → retro
```

- No hidden steps, no optional complexity
- Progressive disclosure for advanced features
- Consistent interface across all commands
- Clear exit codes for automation

### 2.7 P2P-Capable

**Teams can sync without a central server.**

- mDNS discovery for team members
- Vector clocks for conflict resolution
- CRDTs for state merging
- Optional cloud sync, not required

---

## 3. Scope & Boundaries

### 3.1 In Scope

AgilePlus provides the following capabilities:

| Domain | Components | Priority |
|--------|------------|----------|
| **Feature Lifecycle** | specify, research, plan, implement, validate, ship, retro | P0 |
| **Spec Management** | YAML specs, markdown research, plan generation | P0 |
| **Work Package Tracking** | Decomposition, assignment, status tracking | P0 |
| **Agent Dispatch** | Worktree creation, context injection, evidence collection | P0 |
| **Validation** | Policy gates, evidence checking, quality assurance | P0 |
| **Git Integration** | Correlation, worktree management, commit tracking | P0 |
| **Audit Logging** | Hash-chained entries, event sourcing | P1 |
| **Sync** | P2P replication, Plane.so, GitHub integration | P2 |
| **MCP Server** | FastMCP 3.0 integration, agent interface | P1 |

### 3.2 Out of Scope (Explicitly)

| Capability | Reason | Alternative |
|------------|--------|-------------|
| **Code generation/IDE** | Agent responsibility | Use Claude Code, Codex |
| **General project management** | Specialized tools | Use Plane.so, Linear, Jira |
| **Real-time chat** | Communication tools | Use Slack, Discord |
| **Video conferencing** | Meeting tools | Use Zoom, Meet |
| **Document editing** | Specialized editors | Use Notion, Google Docs |
| **CI/CD execution** | Infrastructure concern | Use GitHub Actions, GitLab CI |
| **Production deployment** | Operations concern | Use ArgoCD, Flux |

### 3.3 The 7 Command Workflow

```
┌─────────────────────────────────────────────────────────────────────┐
│              AgilePlus 7-Command Workflow                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────┐     ┌──────────┐     ┌──────────┐                   │
│  │ SPECIFY  │────▶│ RESEARCH │────▶│   PLAN   │                   │
│  │          │     │          │     │          │                   │
│  │ • Create │     │ • Analyze│     │ • Decompose                 │
│  │   spec   │     │   codebase   │     │ • Create WPs │
│  │ • Define │     │ • Research   │     │ • Estimate   │
│  │   FRs    │     │   domain     │     │              │
│  └──────────┘     └──────────┘     └─────┬────┘                   │
│                                           │                         │
│                                           ▼                         │
│  ┌──────────┐     ┌──────────┐     ┌──────────┐                   │
│  │   SHIP   │◀────│ VALIDATE │◀────│IMPLEMENT │                   │
│  │          │     │          │     │          │                   │
│  │ • Merge  │     │ • Verify │     │ • Agent  │                   │
│  │ • Tag    │     │   evidence   │     │   dispatch   │
│  │ • Deploy │     │ • Policy   │     │ • Evidence   │
│  │          │     │   gates    │     │   collection │
│  └──────────┘     └──────────┘     └──────────┘                   │
│       │                                                             │
│       ▼                                                             │
│  ┌──────────┐                                                       │
│  │   RETRO  │                                                       │
│  │          │                                                       │
│  │ • Review │                                                       │
│  │   metrics│                                                       │
│  │ • Learn  │                                                       │
│  │ • Improve│                                                       │
│  └──────────┘                                                       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4. Target Users

### 4.1 Primary User Personas

#### Persona 1: Solo Developer (Alex)

```
┌─────────────────────────────────────────────────────────────────────┐
│  Persona: Alex - Solo Developer                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Role: Independent developer building side projects                   │
│  Stack: Python, Rust, various frameworks                            │
│  Tools: Claude Code, GitHub, VS Code                                │
│                                                                     │
│  Pain Points:                                                       │
│    • Scope creep on personal projects                               │
│    • Forgets what features were planned                             │
│    • No structure to stay organized                                 │
│    • Context switching kills productivity                           │
│                                                                     │
│  AgilePlus Value:                                                   │
│    • Quick spec creation keeps projects focused                     │
│    • Local SQLite database tracks everything                        │
│    • 7-command workflow provides structure                          │
│    • Works offline, no SaaS dependency                                │
│                                                                     │
│  Success Metric: Complete feature from idea to ship in 1 day        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

#### Persona 2: Tech Lead (Morgan)

```
┌─────────────────────────────────────────────────────────────────────┐
│  Persona: Morgan - Tech Lead                                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Role: Engineering lead for 8-person team                           │
│  Stack: Go, Rust, TypeScript, Kubernetes                            │
│  Tools: GitHub, Plane.so, Slack, Claude Code                        │
│                                                                     │
│  Pain Points:                                                       │
│    • PRs merged without proper review                               │
│    • No audit trail for compliance                                  │
│    • Features ship without tests                                      │
│    • Hard to track what's being worked on                           │
│                                                                     │
│  AgilePlus Value:                                                   │
│    • Policy gates enforce quality standards                         │
│    • Hash-chained audit log for compliance                          │
│    • Validation requires evidence (tests, etc.)                     │
│    • Kanban view shows all active work                              │
│                                                                     │
│  Success Metric: Zero quality escapes to production                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

#### Persona 3: AI Coding Agent (Claude/Codex Interface)

```
┌─────────────────────────────────────────────────────────────────────┐
│  Persona: AI Coding Agent Interface                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Role: AI assistant implementing features                           │
│  Stack: MCP, worktrees, git, language tools                           │
│                                                                     │
│  Pain Points:                                                       │
│    • Unclear requirements lead to wrong implementation            │
│    • No isolation means risky experimentation                       │
│    • Unclear what "done" means                                      │
│    • Hard to show work was completed correctly                      │
│                                                                     │
│  AgilePlus Value:                                                   │
│    • Structured specs with clear acceptance criteria                │
│    • Isolated worktrees for safe development                        │
│    • Evidence requirements define completeness                      │
│    • MCP server provides context and records actions                │
│                                                                     │
│  Success Metric: First-attempt approval rate > 80%                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.2 Secondary Users

| User Type | Needs | AgilePlus Support |
|-----------|-------|-------------------|
| **Product Managers** | Feature tracking, roadmap visibility | Spec dashboard, cycle tracking |
| **QA Engineers** | Test traceability, validation evidence | Evidence linking, test requirements |
| **Release Engineers** | Deployment coordination, rollback | Ship command, tagging integration |
| **Security/Compliance** | Audit trails, policy verification | Hash-chained logs, policy gates |
| **Executives** | Velocity metrics, completion rates | Retro analytics, cycle reports |

---

## 5. Success Criteria

### 5.1 Key Performance Indicators (KPIs)

| KPI | Target | Measurement | Frequency |
|-----|--------|-------------|-----------|
| **Time to Plan** | < 10 minutes | Developer survey | Per feature |
| **Validation Coverage** | 100% | Evidence audit | Per validation |
| **Governance Compliance** | Zero violations | Policy gate logs | Real-time |
| **Agent Success Rate** | > 80% | First-attempt pass rate | Weekly |
| **Idea to Ship** | < 1 week median | Cycle time tracking | Weekly |
| **User Satisfaction** | NPS > 50 | Developer survey | Monthly |

### 5.2 The "7 Command" Success Metrics

```
┌─────────────────────────────────────────────────────────────────────┐
│  Command Success Criteria                                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  SPECIFY                                                            │
│  ├── Success: Spec created with FRs in < 10 minutes               │
│  ├── Quality: All P0 FRs have acceptance criteria                   │
│  └── Evidence: spec.yaml exists in kitty-specs/                     │
│                                                                     │
│  RESEARCH                                                           │
│  ├── Success: Research artifacts created                          │
│  ├── Quality: Codebase analysis identifies integration points       │
│  └── Evidence: research.md exists with findings                   │
│                                                                     │
│  PLAN                                                               │
│  ├── Success: Work packages created and assigned                  │
│  ├── Quality: Dependencies identified, no circular blockers         │
│  └── Evidence: plan.md with WP breakdown                            │
│                                                                     │
│  IMPLEMENT                                                          │
│  ├── Success: Agent completes implementation                        │
│  ├── Quality: All acceptance criteria addressed                   │
│  └── Evidence: Worktree with commits, tests passing                 │
│                                                                     │
│  VALIDATE                                                           │
│  ├── Success: All policy gates pass                               │
│  ├── Quality: Evidence links FRs to tests/code                    │
│  └── Evidence: Validation report generated                        │
│                                                                     │
│  SHIP                                                               │
│  ├── Success: Code merged to main                                 │
│  ├── Quality: No policy violations, all gates passed              │
│  └── Evidence: Git tag, Plane.so update, PR merged                │
│                                                                     │
│  RETRO                                                              │
│  ├── Success: Metrics collected and reviewed                        │
│  ├── Quality: Action items identified and tracked                 │
│  └── Evidence: retro.md with learnings                            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.3 Quarterly OKRs

#### Q2 2026: Foundation

| Objective | Key Results | Owner |
|-----------|-------------|-------|
| 7-command stability | KR1: All commands functional in core | @core-team |
| | KR2: 100% internal feature adoption | @adoption-team |
| | KR3: < 10 min average specify time | @ux-team |
| Spec-driven validation | KR1: Validation gates enforce evidence | @governance-team |
| | KR2: Zero violations to main branches | @governance-team |

---

## 6. Governance Model

### 6.1 Governance Principles

```
┌─────────────────────────────────────────────────────────────────────┐
│  AgilePlus Governance Principles                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. QUALITY GATES ARE NON-NEGOTIABLE                                │
│     • No code ships without validation                              │
│     • No exceptions for timeline pressure                           │
│     • Failed gates block merge automatically                        │
│                                                                     │
│  2. SPEC-DRIVEN IS THE DEFAULT                                      │
│     • No implementation without specification                     │
│     • Specs are validated, not just written                         │
│     • Machine-readable specs enable automation                      │
│                                                                     │
│  3. LOCAL-FIRST IS PRIVACY-FIRST                                    │
│     • Cloud sync is optional, never required                        │
│     • User data never leaves their machine without consent          │
│     • P2P sync preserves data sovereignty                           │
│                                                                     │
│  4. GIT-NATIVE IS TRANSPARENCY                                      │
│     • All artifacts in version control                              │
│     • Every change has a commit hash                                │
│     • No proprietary data silos                                     │
│                                                                     │
│  5. AGENT-ORCHESTRATED IS COLLABORATIVE                             │
│     • Agents augment humans, don't replace them                       │
│     • Human review is always the final gate                         │
│     • Evidence collection proves agent work quality                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 6.2 Governance Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│  AgilePlus Governance Structure                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│                    ┌───────────────────┐                            │
│                    │   Product Owner   │                            │
│                    │   (Final Authority)│                           │
│                    └─────────┬─────────┘                            │
│                              │                                       │
│          ┌───────────────────┼───────────────────┐                 │
│          │                   │                   │                   │
│          ▼                   ▼                   ▼                   │
│  ┌───────────────┐   ┌───────────────┐   ┌───────────────┐          │
│  │   Core        │   │   Governance  │   │   Integration │          │
│  │   Architecture│   │   Council       │   │   Council       │          │
│  │   Board       │   │               │   │               │          │
│  │               │   │ • Policy gates│   │ • Plane.so    │          │
│  │ • CLI design  │   │ • Audit       │   │ • GitHub      │          │
│  │ • Data model  │   │ • Validation  │   │ • MCP         │          │
│  │ • Protocol    │   │ • Compliance  │   │ • Sync        │          │
│  └───────────────┘   └───────────────┘   └───────────────┘          │
│                                                                     │
│  Working Groups:                                                    │
│  ├── CLI/Core (@cli-lead)                                           │
│  ├── MCP Server (@mcp-lead)                                         │
│  ├── P2P Sync (@sync-lead)                                            │
│  └── Documentation (@docs-lead)                                     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 7. Charter Compliance Checklist

### 7.1 Compliance Requirements

| Requirement | Evidence | Status | Last Verified |
|------------|----------|--------|---------------|
| **7-Command Workflow** | All commands functional | ⬜ | TBD |
| **Spec-Driven** | All features have specs | ⬜ | TBD |
| **Local-First** | SQLite local storage | ⬜ | TBD |
| **Git-Native** | All artifacts in git | ⬜ | TBD |
| **Validation Gates** | Policy enforcement active | ⬜ | TBD |
| **Audit Logging** | Hash-chained entries | ⬜ | TBD |
| **P2P Sync** | mDNS/CRDT capability | ⬜ | TBD |

### 7.2 Charter Amendment Process

| Amendment Type | Approval Required | Process |
|---------------|-------------------|---------|
| **7-command changes** | Architecture Board + Product Owner | RFC → Vote → Update |
| **Governance policy** | Governance Council | Proposal → Review → Vote |
| **Integration changes** | Integration Council | RFC → Review → Update |

---

## 8. Decision Authority Levels

### 8.1 Authority Matrix

```
┌─────────────────────────────────────────────────────────────────────┐
│  Decision Authority Matrix (RACI)                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  CORE WORKFLOW:                                                       │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ Decision              │ R        │ A       │ C        │ I      │ │
│  ├───────────────────────┼──────────┼─────────┼──────────┼────────┤ │
│  │ 7-command changes     │ Core     │ Arch    │ Gov      │ All    │ │
│  │                       │ Team     │ Board   │ Council    │ Users  │ │
│  ├───────────────────────┼──────────┼─────────┼──────────┼────────┤ │
│  │ Data model changes    │ Core     │ Arch    │ Int      │ All    │ │
│  │                       │ Team     │ Board   │ Council    │ Users  │ │
│  ├───────────────────────┼──────────┼─────────┼──────────┼────────┤ │
│  │ CLI UX changes        │ Core     │ Arch    │ UX       │ Users  │ │
│  │                       │ Team     │ Board   │ Team     │        │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  GOVERNANCE & VALIDATION:                                             │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ Decision              │ R        │ A       │ C        │ I      │ │
│  ├───────────────────────┼──────────┼─────────┼──────────┼────────┤ │
│  │ Policy gate rules     │ Gov      │ Gov     │ Arch     │ All    │ │
│  │                       │ Team     │ Council │ Board    │ Users  │ │
│  ├───────────────────────┼──────────┼─────────┼──────────┼────────┤ │
│  │ Validation criteria   │ Gov      │ Gov     │ Core     │ Users  │ │
│  │                       │ Team     │ Council │ Team     │        │ │
│  ├───────────────────────┼──────────┼─────────┼──────────┼────────┤ │
│  │ Audit requirements    │ Gov      │ Gov     │ Security │ Exec   │ │
│  │                       │ Team     │ Council │ Team     │        │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 9. Appendices

### 9.1 Glossary

| Term | Definition |
|------|------------|
| **Spec** | Feature specification with FRs and acceptance criteria |
| **WP** | Work Package - decomposed unit of implementation |
| **FR** | Functional Requirement |
| **CRDT** | Conflict-free Replicated Data Type |
| **MCP** | Model Context Protocol for AI agent integration |
| **P2P** | Peer-to-peer networking |
| **Kitty Spec** | AgilePlus specification directory |
| **Evidence** | Proof that a requirement is satisfied |

### 9.2 Related Documents

| Document | Location | Purpose |
|----------|----------|---------|
| SPEC.md | SPEC.md | Technical specification |
| ADRs | docs/adr/ | Architecture decisions |
| Workflow | docs/workflow/ | Usage documentation |

### 9.3 Charter Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 2.0.0 | 2026-04-05 | AgilePlus Team | Initial charter |

### 9.4 Ratification

This charter is ratified by:

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Product Owner | TBD | 2026-04-05 | ✓ |
| Architecture Board Chair | TBD | 2026-04-05 | ✓ |
| Governance Council Lead | TBD | 2026-04-05 | ✓ |

---

**END OF CHARTER**

*This document is a living charter. It should be reviewed quarterly and updated as the project evolves while maintaining alignment with the core mission and tenets.*
