# Implementation: CLI Tools Consolidation

## Spec ID
017

## Current State (0→Current)
**Status**: In Progress

Consolidating CLI tools across Phenotype projects.

## 0→Current Evolution
### Phase 1: Foundation
- CLI inventory completed
- Consolidation strategy defined
- Architecture designed

### Phase 2: Core Features
- Unified CLI framework
- Command standardization
- Shared components

### Phase 3: Refinement
- Documentation
- Testing
- Distribution

## Current Implementation
### Components
- Unified CLI framework (heliosCLI-based)
- Standard commands
- Shared utilities

### Data Model
- Command: name, description, flags[], subcommands[], handler
- CLIConfig: global_flags, theme, output_format

### API Surface
- Main CLI entry point
- Plugin API for commands
- Configuration API

## FR Traceability
| FR-ID | Description | Test References |
|-------|-------------|----------------|
| FR-001 | CLI framework | cli/ framework |
| FR-002 | Standard commands | cli/commands/ |
| FR-003 | Shared utilities | cli/utils/ |

## Future States (Current→Future)
### Planned
- Full consolidation
- Unified help system
- Auto-completion

### Considered
- GUI companion
- Web-based CLI

### Backlog
- Full documentation
- Tutorial suite

## Verification
- [ ] CLI builds
- [ ] Commands work
- [ ] Help displays

## Changelog
| Date | Change | Notes |
|------|--------|-------|
| 2026-04-02 | Initial spec | CLI consolidation |
