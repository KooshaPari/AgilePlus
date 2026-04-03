# ADR-001: Kitty-Specs Format for Spec-Driven Development

**Status:** Accepted  
**Date:** 2026-04-02  
**Authors:** AgilePlus Team

---

## Context

AgilePlus needs a spec format that is:
1. Human-readable (markdown)
2. Machine-parseable (structured)
3. Version-controlled (git)
4. AI-agent friendly

### Research Reviewed

1. **"Literate Programming"** (Knuth, 1984)
2. **"Behavior-Driven Development"** (North, 2006)
3. **"Docs-as-Code"** (Gentle, 2020)
4. **Existing tools:** Jira, Linear, Notion, GitHub Issues

### Alternatives Considered

| Approach | Pros | Cons | Research |
|----------|------|------|----------|
| **Jira tickets** | Rich features | Proprietary, not git-native | Industry standard |
| **GitHub Issues** | Git-native | Limited structure | Common practice |
| **Notion docs** | Flexible | Not version-controlled | Popular |
| **Markdown + YAML frontmatter** | Flexible | Inconsistent | Various |
| **Kitty-Specs (selected)** | Structured, git-native, parseable | Learning curve | Knuth 1984 |

---

## Decision

**Adopt Kitty-Specs format with 4-file structure.**

### Format

```
kitty-specs/<feature-id>/
├── spec.md      # Requirements, scope, acceptance criteria
├── plan.md      # Technical approach, architecture
├── tasks.md     # Work packages, assignments
└── research.md  # SOTA analysis, findings
```

### Key Features

1. **Human-readable:** Markdown for easy reading
2. **Structured sections:** Standard headings for parsing
3. **FR traceability:** Functional Requirements with IDs
4. **Machine-parseable:** Can extract tasks, requirements automatically
5. **Git-native:** Version control, diffs, collaboration

---

## Consequences

### Positive

1. **Version control:** Specs evolve with code
2. **Collaboration:** PR reviews for specs
3. **Automation:** Parse specs for tooling
4. **Portability:** Markdown works everywhere
5. **Literate programming:** Knuth's vision realized

### Negative

1. **Learning curve:** Team must learn format
2. **Discipline required:** Must maintain specs
3. **No GUI:** CLI/text only

### Neutral

1. **Storage:** Files in git vs. database

---

## Research Links

- Literate Programming: https://en.wikipedia.org/wiki/Literate_programming
- BDD: https://cucumber.io/docs/bdd/
- Docs-as-Code: https://www.writethedocs.org/guide/docs-as-code/

---

## Implementation

- `kitty-specs/` — 39 specs created
- `SPEC.md` — Format documentation
- AGENTS.md — Spec requirements for agents

---

**Supersedes:** N/A  
**Superseded by:** N/A
