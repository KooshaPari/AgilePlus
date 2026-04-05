---
id: FR-ARCHIVE-001
title: CodeProjects Archive Manifest
status: specified
priority: P3
created: 2026-03-25
category: maintenance
owner: kooshapari
source: kitty-specs/codeprojects-archive-manifest
---

# FR-ARCHIVE-001: CodeProjects Archive Manifest

## Description

Reduce orphan friction under `/Users/kooshapari/CodeProjects/archive`. Create README or MANIFEST for each top-level tree; document zip blobs.


## User Stories

### US-1: Core Functionality (P1)
**Given** a user of the system,
**When** they interact with this feature,
**Then** the system behaves as specified with proper traceability.

### US-2: Integration Scenario (P2)
**Given** the component is part of the ecosystem,
**When** integrated with other components,
**Then** it maintains FR traceability and governance compliance.

## Acceptance Criteria

- [ ] `archive/MANIFEST.md` at CodeProjects root or `archive/README.md` listing directories, size tier, git yes/no, suggested disposition
- [ ] At least three no-git directories get one-paragraph provenance note

## Notes

Work may span paths outside Phenotype monorepo; track evidence paths in spec tasks.

Original: `kitty-specs/codeprojects-archive-manifest/`
