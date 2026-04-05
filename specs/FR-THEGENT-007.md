---
id: FR-THEGENT-007
title: thegent Dotfiles Manager Consolidation
status: specified
priority: P2
created: 2026-03-25
category: tooling
owner: thegent-team
source: kitty-specs/thegent-dotfiles-consolidation
---

# FR-THEGENT-007: thegent Dotfiles Manager Consolidation

## Description

Consolidate thegent as the dotfiles/bootstrap manager for all systems. Centralize governance, templates, and hooks scattered across dozens of repos and locations.

## Problem

- Governance scattered across repos
- Templates in template-* repos
- Hooks in ~/.claude/hooks/
- Orphan project configs in CodeProjects/orphans/

## Goal

Run `thegent setup` to configure any macOS/Linux/Windows system with single command.


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

- [ ] thegent/templates/ contains ALL project templates
- [ ] thegent/hooks/ contains ALL Claude hooks
- [ ] thegent/dotfiles/ manages shell config, brew packages, dev tools
- [ ] `thegent setup <profile>` command bootstraps a system
- [ ] thegent/crates/ Rust libs extracted to published crates
- [ ] README: single-command system setup documented

## Consolidation Sources

- ~/.claude/ hooks → thegent/hooks/
- template-commons, template-lang-* repos → thegent/templates/
- ~/.claude/CLAUDE.md governance → thegent/governance/
- Orphan project configs → thegent/dotfiles/

## Notes

Original: `kitty-specs/thegent-dotfiles-consolidation/`
