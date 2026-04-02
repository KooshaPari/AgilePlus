# Work Packages: Portfolio and Web Apps — Complete Portfolio and Web Applications

**Inputs**: Design documents from `kitty-specs/020-portfolio-and-web-apps/`
**Prerequisites**: spec.md, access to all 15 web app repositories
**Scope**: Cross-repo (15 repositories) — portfolio, web apps, external repos

---

## WP-001: koosha-portfolio — Complete Portfolio Site

- **State:** planned
- **Sequence:** 1
- **File Scope:** koosha-portfolio repository (TypeScript/Next.js)
- **Acceptance Criteria:**
  - Complete portfolio site with project showcases linked to active repos
  - Responsive design with mobile, tablet, and desktop breakpoints
  - Accessibility compliance (WCAG 2.1 AA)
  - Performance: Lighthouse score ≥90 for performance, accessibility, SEO
  - Deployed to production with CI/CD pipeline
  - All tests passing
- **Estimated Effort:** M

Complete the koosha-portfolio personal portfolio site with project showcases, responsive design, and accessibility. The site links to active repositories in the Phenotype ecosystem, providing a public-facing showcase of work.

### Subtasks
- [ ] T001 Audit current koosha-portfolio: existing pages, components, gaps
- [ ] T002 Complete homepage: hero section, about, featured projects
- [ ] T003 Complete project showcase pages with links to active repos
- [ ] T004 Implement responsive design: mobile, tablet, desktop breakpoints
- [ ] T005 Implement accessibility: ARIA labels, keyboard navigation, color contrast
- [ ] T006 Optimize performance: image optimization, code splitting, caching
- [ ] T007 Set up CI/CD pipeline: build, test, deploy on push
- [ ] T008 Deploy to production and verify
- [ ] T009 Run Lighthouse audit: verify ≥90 scores for performance, accessibility, SEO
- [ ] T010 Write tests for key components and pages
- [ ] T011 Run quality checks: lint, type-check, test

### Dependencies
- None (can start independently)

### Risks & Mitigations
- Design scope creep: Define MVP pages first, add enhancements incrementally
- Performance issues: Audit early, optimize images and bundles

---

## WP-002: Parpoura — Complete App Implementation

- **State:** planned
- **Sequence:** 2
- **File Scope:** Parpoura repositories (private and public — deduplication required)
- **Acceptance Criteria:**
  - Parpoura deduplicated across private and public contexts (single source of truth)
  - Data pipeline web app complete with core features
  - API endpoints functional and documented
  - UI complete with data visualization
  - ≥80% test coverage
  - All quality checks passing
- **Estimated Effort:** M

Complete the Parpoura data pipeline web app and resolve the duplication between private and public repositories. One version is kept as the source of truth, the other is archived with a migration path.

### Subtasks
- [ ] T012 Audit private Parpoura: features, code quality, test coverage
- [ ] T013 Audit public Parpoura: features, code quality, test coverage
- [ ] T014 Compare private vs public: identify differences, decide keep/archive
- [ ] T015 Merge unique features from archived version into kept version
- [ ] T016 Complete data pipeline UI: pipeline visualization, status monitoring
- [ ] T017 Complete API endpoints: pipeline management, data processing, results
- [ ] T018 Implement data visualization: charts, graphs, real-time updates
- [ ] T019 Archive superseded version with migration README
- [ ] T020 Write tests for merged app (target: ≥80% coverage)
- [ ] T021 Add documentation: setup, API reference, deployment guide
- [ ] T022 Run quality checks across kept version

### Dependencies
- None (can start independently, but benefits from WP-001 patterns)

### Risks & Mitigations
- Data loss during deduplication: Comprehensive feature audit before merge decision
- API compatibility: Version APIs, document breaking changes

---

## WP-003: phenodocs — Complete VitePress Federation Hub

- **State:** planned
- **Sequence:** 3
- **File Scope:** phenodocs repository (TypeScript/VitePress)
- **Acceptance Criteria:**
  - Complete VitePress documentation federation hub
  - Integrated with phenotype-docs-engine for content sourcing
  - All Phenotype ecosystem docs federated and accessible
  - Search functionality across all federated docs
  - Responsive design with mobile support
  - Deployed to production
  - All quality checks passing
- **Estimated Effort:** M

Complete phenodocs as the VitePress federation hub that aggregates documentation from across the Phenotype ecosystem. It integrates with phenotype-docs-engine for content sourcing and provides unified search across all federated documentation.

### Subtasks
- [ ] T023 Audit current phenodocs: existing pages, federation setup, gaps
- [ ] T024 Complete VitePress configuration: theme, navigation, sidebar
- [ ] T025 Integrate with phenotype-docs-engine: content sourcing, sync mechanism
- [ ] T026 Implement doc federation: aggregate docs from all Phenotype repos
- [ ] T027 Implement search: full-text search across all federated docs
- [ ] T028 Implement responsive design: mobile-friendly navigation and reading
- [ ] T029 Set up CI/CD: build, test, deploy on content changes
- [ ] T030 Deploy to production and verify federation
- [ ] T031 Write tests for federation and search functionality
- [ ] T032 Add documentation: contributing guide, federation setup
- [ ] T033 Run quality checks: lint, type-check, test

### Dependencies
- None (can start independently)

### Risks & Mitigations
- Federation complexity: Start with single source, add sources incrementally
- Search performance: Use VitePress built-in search, optimize for large doc sets

---

## WP-004: FixitGo + FixitRs — Complete Fix-It Tools

- **State:** planned
- **Sequence:** 4
- **File Scope:** FixitGo repository (Go/HTMX), FixitRs repository (Rust/Leptos)
- **Acceptance Criteria:**
  - FixitGo: complete bug tracking tool with HTMX-based UI
  - FixitRs: complete bug tracking tool with Leptos-based UI
  - Decision documented: keep both, merge into one, or archive one
  - Both tools have ≥80% test coverage (if kept)
  - All quality checks passing
- **Estimated Effort:** M

Complete both FixitGo and FixitRs as bug tracking tools, then decide whether to keep both implementations or consolidate into one. FixitGo uses Go/HTMX for a server-rendered approach, while FixitRs uses Rust/Leptos for a WASM-based approach.

### Subtasks
- [ ] T034 Audit FixitGo: existing bug tracking features, HTMX integration, gaps
- [ ] T035 Complete FixitGo: bug creation, assignment, status tracking, search
- [ ] T036 Complete FixitGo UI: HTMX-based forms, real-time updates, responsive design
- [ ] T037 Audit FixitRs: existing bug tracking features, Leptos integration, gaps
- [ ] T038 Complete FixitRs: bug creation, assignment, status tracking, search
- [ ] T039 Complete FixitRs UI: Leptos-based components, WASM rendering
- [ ] T040 Compare both tools: feature parity, performance, maintainability
- [ ] T041 Make keep/merge/archive decision with documented rationale
- [ ] T042 Write tests for both tools (target: ≥80% coverage each)
- [ ] T043 Add documentation for both tools
- [ ] T044 Run quality checks across both repos

### Dependencies
- None (can start independently)

### Risks & Mitigations
- Maintaining two implementations: Decision at T041 should favor one unless both serve distinct purposes
- Leptos maturity: Test WASM compatibility, document any limitations

---

## WP-005: External Repos Triage — Decide Keep/Archive/Integrate

- **State:** planned
- **Sequence:** 5
- **File Scope:** 7 external repositories (portage, colab, vibeproxy, Planify, MCPForge, Synthia, Tossy)
- **Acceptance Criteria:**
  - Each external repo evaluated with: purpose, completion status, strategic value, maintenance cost
  - Decision documented for each: keep, archive, or integrate
  - Approved integrations planned with migration paths
  - Archived repos documented with rationale
  - Maintenance strategy established for kept repos
  - All decisions documented with rationale
- **Estimated Effort:** M

Evaluate the 7 external repositories and decide whether to keep, archive, or integrate each. This includes portage (package manager UI), colab (collaboration tool), vibeproxy (vibe coding proxy), Planify (planning tool), MCPForge (MCP web interface), Synthia (AI assistant UI), and Tossy (utility app).

### Subtasks
- [ ] T045 Audit portage: purpose, completion status, strategic value
- [ ] T046 Audit colab: purpose, completion status, strategic value
- [ ] T047 Audit vibeproxy: purpose, completion status, strategic value
- [ ] T048 Audit Planify: purpose, completion status, strategic value
- [ ] T049 Audit MCPForge: purpose, completion status, strategic value, integration with agent framework
- [ ] T050 Audit Synthia: purpose, completion status, strategic value
- [ ] T051 Audit Tossy: purpose, completion status, strategic value
- [ ] T052 Evaluate each repo: keep, archive, or integrate with rationale
- [ ] T053 Plan integrations for approved repos: migration path, timeline
- [ ] T054 Archive non-strategic repos: Dino, portage, colab, Tossy (per spec)
- [ ] T055 Complete strategic repos: phenodocs (WP-003), MCPForge
- [ ] T056 Establish maintenance strategy for kept repos: update cadence, ownership
- [ ] T057 Document all decisions with rationale in portfolio documentation

### Dependencies
- WP-001 through WP-004 (understanding of completed apps informs external repo decisions)

### Risks & Mitigations
- Integration complexity: Start with simple integrations, defer complex ones
- Maintenance burden: Be realistic about kept repos, archive aggressively for non-strategic apps

---

## Dependency & Execution Summary

```
WP-001 (koosha-portfolio) ───────────── first, no deps
WP-002 (Parpoura dedup + complete) ──── first, no deps (parallel with WP-001)
WP-003 (phenodocs federation hub) ───── first, no deps (parallel with WP-001, WP-002)
WP-004 (FixitGo + FixitRs) ──────────── first, no deps (parallel with WP-001, WP-002, WP-003)
WP-005 (External repos triage) ──────── depends on WP-001 through WP-004
```

**Parallelization**: WP-001 through WP-004 can run in parallel (independent codebases). WP-005 is the final integration and decision step.

**MVP Scope**: WP-001 alone completes the portfolio site. WP-003 completes the docs hub. WP-005 makes strategic decisions about the remaining apps.
