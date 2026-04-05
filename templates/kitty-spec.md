# Template: Kitty-Spec

Copy this template to create a new feature specification.

```markdown
# Feature Specification: <Feature Name>

**Feature ID**: `XXX-feature-name`  
**Feature Branch**: `<category>/feature-name`  
**Created**: YYYY-MM-DD  
**Status**: Draft  
**Mission**: <software-dev|research|governance>

## Overview

Brief description of what this feature does and why it exists.

### Objectives

1. Objective 1
2. Objective 2
3. Objective 3

### Success Criteria

- [ ] Criterion 1
- [ ] Criterion 2

---

## Work Packages

### WP1: <Name>
**Status**: 🚧 In Progress

Description of this work package.

- [ ] Task 1
- [ ] Task 2
- [ ] Task 3

**Visual Evidence**: (GIF/screenshot when complete)

---

### WP2: <Name>
**Status**: ⬜ Not Started

Description.

- [ ] Task 1
- [ ] Task 2

---

## Visual Evidence

### Screenshots/GIFs

![Feature demo](../../assets/gifs/feature-demo.gif)

### Terminal Recording

```bash
# Use vhs for terminal recordings
vhs < demo.tape > docs/assets/gifs/feature-cli.gif
```

## Documentation

- **Docs Page**: `docs/<category>/feature-name.md`
- **API Reference**: (if applicable)
- **Related Features**: Link to related docs

## Changelog Entry Template

```markdown
- **scope**: Description (#PR)
  ![feature-demo](docs/assets/gifs/feature.gif)
  [Documentation →](docs/category/feature.md)
```

## Related

- [Parent Feature](../XXX-parent/spec.md)
- [Dependencies: WPX from Feature YYY](../YYY-feature/spec.md)
```

## Status Legend

| Icon | Status |
|------|--------|
| ⬜ | Not Started |
| 🚧 | In Progress |
| ✅ | Complete |
| ⏸️ | Blocked |
| 🔄 | In Review |

## Creating a New Spec

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus

# Use the CLI
agileplus specify --title "<Feature Name>" --description "<Description>"

# Or manually:
mkdir kitty-specs/XXX-feature-name
cp templates/kitty-spec.md kitty-specs/XXX-feature-name/spec.md
# Edit and fill in details
```

## Checklist for PR

Before submitting PR for this feature:

- [ ] All work packages marked complete
- [ ] Visual evidence attached to PR
- [ ] Documentation page created at `docs/<category>/feature.md`
- [ ] Docs page linked in VitePress sidebar
- [ ] Changelog entry added with visual preview
- [ ] Cross-references to related specs added
