# Portfolio and Web Apps

## Meta

- **ID**: 020-portfolio-and-web-apps
- **Title**: Complete Portfolio and Web Applications
- **Created**: 2026-04-01
- **State**: specified
- **Scope**: Cross-repo (15 repositories)

## Context

The Phenotype ecosystem includes 15 portfolio and web application repositories spanning personal projects, internal tools, and external integrations. These applications have varying levels of completion, maintenance status, and strategic value.

koosha-portfolio is a personal portfolio site that needs completion. Parpoura appears in both private and public repos requiring deduplication. phenodocs is a documentation site that needs integration with the docs engine. FixitGo and FixitRs are bug tracking tools in different languages. Dino is a game project. Tracera appears in both observability and web apps contexts. cloud (Kilo-Org) is a cloud management interface. portage, colab, and vibeproxy are utility web apps. Planify is a planning tool. MCPForge is an MCP-related web interface. Synthia is an AI assistant interface. Tossy is a utility app.

This spec completes portfolio and web apps by deciding keep/archive/integrate for each, with special attention to external repos that need integration decisions.

## Problem Statement

Portfolio and web apps are incomplete and fragmented:
- **Personal portfolio** — koosha-portfolio needs completion
- **Duplicate repos** — Parpoura and Tracera appear in multiple contexts
- **Incomplete apps** — several web apps are prototypes without completion
- **External repos** — unclear which external repos should be integrated
- **Maintenance burden** — apps not regularly maintained or updated
- **No unified strategy** — no clear decision framework for app lifecycle

## Goals

- Complete koosha-portfolio personal portfolio site
- Deduplicate Parpoura and Tracera across contexts
- Complete or archive incomplete web apps
- Decide keep/archive/integrate for external repos
- Establish maintenance strategy for portfolio and web apps
- Document all portfolio and web app decisions

## Non-Goals

- Creating new web applications beyond completion
- Migrating web apps to different frameworks
- Rewriting web app internals (completion only)
- Deprecating any web app functionality (complete or archive)

## Repositories Affected

| Repo | Language | Type | Current State | Action |
|------|----------|------|---------------|--------|
| koosha-portfolio | TypeScript/Next.js | Personal portfolio | Incomplete | Complete portfolio site |
| Parpoura | TypeScript | Data pipeline web app | Duplicate with private | Deduplicate, complete or archive |
| phenodocs | TypeScript/VitePress | Documentation site | Partial | Complete, integrate with docs engine |
| FixitGo | Go/HTMX | Bug tracking | Prototype | Complete or archive |
| FixitRs | Rust/Leptos | Bug tracking | Prototype | Complete or archive |
| Dino | TypeScript | Game project | Prototype | Archive (non-strategic) |
| Tracera | Rust/Web | Profiling UI | Duplicate with observability | Deduplicate, complete or archive |
| cloud (Kilo-Org) | TypeScript | Cloud management | Incomplete | Complete or archive |
| portage | TypeScript | Package manager UI | Prototype | Archive (non-strategic) |
| colab | TypeScript | Collaboration tool | Prototype | Archive (non-strategic) |
| vibeproxy | TypeScript | Vibe coding proxy | Prototype | Complete or archive |
| Planify | TypeScript | Planning tool | Incomplete | Complete or archive |
| MCPForge | TypeScript | MCP web interface | Prototype | Complete, integrate with agent framework |
| Synthia | TypeScript | AI assistant UI | Incomplete | Complete or archive |
| Tossy | TypeScript | Utility app | Prototype | Archive (non-strategic) |

## Technical Approach

### Phase 1: Audit and Decision Framework (Week 1-2)
1. Audit all 15 web apps: features, completion status, strategic value
2. Define decision framework (keep/archive/integrate)
3. Categorize apps by strategic value and completion effort
4. Identify duplicates and external integration candidates

### Phase 2: Portfolio Completion (Week 2-4)
1. Complete koosha-portfolio personal portfolio site
2. Add project showcases with links to active repos
3. Implement responsive design and accessibility
4. Deploy and verify production readiness

### Phase 3: Deduplication (Week 4-5)
1. Deduplicate Parpoura across private and public contexts
2. Deduplicate Tracera across observability and web app contexts
3. Archive superseded duplicates
4. Update references to archived duplicates

### Phase 4: App Completion or Archive (Week 5-8)
1. Complete strategic apps: phenodocs, MCPForge
2. Archive non-strategic apps: Dino, portage, colab, Tossy
3. Evaluate remaining apps for completion or archive
4. Document all decisions with rationale

### Phase 5: External Integration Decisions (Week 8-9)
1. Evaluate external repos for integration
2. Plan integration for approved external repos
3. Archive external repos not approved for integration
4. Document integration decisions

### Phase 6: Maintenance Strategy (Week 9-10)
1. Establish maintenance schedule for active web apps
2. Define lifecycle management for web apps
3. Create monitoring and alerting for production apps
4. Document maintenance procedures

## Success Criteria

- koosha-portfolio completed and deployed
- Parpoura and Tracera deduplicated
- Strategic web apps completed (phenodocs, MCPForge)
- Non-strategic web apps archived
- External repos evaluated with keep/archive/integrate decisions
- Maintenance strategy established and documented
- All decisions documented with rationale

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking changes during deduplication | Medium | Careful merge, preserve all functionality |
| User disruption from archived apps | Low | Communication, migration guides where applicable |
| Scope creep from additional app features | Medium | Strict scope enforcement, defer to future specs |
| External repo integration complexity | Medium | Clear integration criteria, phased approach |
| Maintenance burden from completed apps | Medium | Realistic maintenance strategy, automation |

## Work Packages

| ID | Description | State |
|----|-------------|-------|
| WP001 | Audit and decision framework | specified |
| WP002 | Portfolio completion | specified |
| WP003 | Deduplication | specified |
| WP004 | App completion or archive | specified |
| WP005 | External integration decisions | specified |
| WP006 | Maintenance strategy | specified |

## Traces

- Related: 012-github-portfolio-triage
- Related: 014-observability-stack-completion
- Related: 016-agent-framework-expansion
