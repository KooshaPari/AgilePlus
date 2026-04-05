# Feature Specification: PR Visual Requirements Policy

**Feature ID**: `023-pr-visual-requirements`  
**Feature Branch**: `eco/pr-visual-requirements`  
**Created**: 2026-04-04  
**Status**: Complete  
**Mission**: governance

## Overview

Establish mandatory PR requirements for visual evidence (screenshots/GIFs), completed specifications, and documentation pages. Creates a visual changelog that can be browsed to see features in action without running the application.

### Objectives

1. All PRs must include embedded visual evidence
2. All feature PRs must reference a completed kitty-spec
3. All features must have a corresponding documentation page with visual examples
4. CHANGELOG entries include visual previews for scannable history

### Success Criteria

- [ ] PR template updated with visual requirements checklist
- [ ] Governance policy documented
- [ ] Workflow created to validate PR requirements
- [ ] CONTRIBUTING.md updated
- [ ] Example documentation pages created
- [ ] Documentation guide written
- [ ] Spec for this work created (dog-food)

---

## Work Packages

### WP1: Governance Documentation
**Status**: ✅ Complete

Create the PR Requirements Policy document.

- [x] Create `GOVERNANCE_PR_REQUIREMENTS.md`
- [x] Define visual evidence requirements
- [x] Define spec completion requirements
- [x] Define documentation requirements
- [x] Define changelog format with visual previews
- [x] Document exception cases

**Visual Evidence**: 
![pr-template](../../assets/gifs/pr-template-updated.gif)

---

### WP2: PR Template Update
**Status**: ✅ Complete

Update the GitHub PR template with new checkboxes.

- [x] Update `.github/pull_request_template.md`
- [x] Add Visual Evidence section with instructions
- [x] Add Specification section
- [x] Add Documentation section
- [x] Add Changelog section

**Visual Evidence**: 
![pr-template](../../assets/gifs/pr-template-updated.gif)

---

### WP3: CI Workflow
**Status**: ✅ Complete

Create workflow to validate PR requirements.

- [x] Create `.github/workflows/pr-requirements.yml`
- [x] Validate checklist items are checked
- [x] Post comment if requirements not met

---

### WP4: Documentation Structure
**Status**: ✅ Complete

Create documentation pages and structure.

- [x] Create `docs/contributing/documentation.md` guide
- [x] Create `docs/dashboard/` section
- [x] Create `docs/cli/` section
- [x] Create `docs/api/` section
- [x] Create example `docs/dashboard/service-controls.md`
- [x] Update `docs/.vitepress/site-meta.mjs` with sidebar
- [x] Update `docs/index.md` with navigation

**Visual Evidence**:
![docs-structure](../../assets/gifs/docs-structure.gif)

---

### WP5: Changelog Update
**Status**: ✅ Complete

Update CHANGELOG.md with visual navigation header.

- [x] Add visual navigation header
- [x] Add template for visual entries
- [x] Update existing entries format reference

---

### WP6: CONTRIBUTING.md Update
**Status**: ✅ Complete

Update contribution guidelines.

- [x] Add PR Requirements section
- [x] Link to full policy
- [x] Add quick reference table
- [x] Update development workflow

---

### WP7: GOVERNANCE.md Reference
**Status**: ✅ Complete

Link PR requirements from main governance.

- [x] Update `GOVERNANCE.md` PR merge line
- [x] Add changelog entry for governance change

---

### WP8: Create This Spec (Dog-food)
**Status**: ✅ Complete

Create the spec for this governance work.

- [x] Create `kitty-specs/023-pr-visual-requirements/`
- [x] Write `spec.md` with all WPs
- [x] Write `meta.json`
- [x] Create template at `templates/kitty-spec.md`

---

## Documentation

- **Policy**: [GOVERNANCE_PR_REQUIREMENTS.md](../../GOVERNANCE_PR_REQUIREMENTS.md)
- **Contributing**: [CONTRIBUTING.md](../../CONTRIBUTING.md)
- **Docs Guide**: [docs/contributing/documentation.md](../../docs/contributing/documentation.md)
- **Example Page**: [docs/dashboard/service-controls.md](../../docs/dashboard/service-controls.md)

## Changelog Entry

```markdown
- **governance**: Add PR visual requirements policy (#023)
  ![pr-requirements](docs/assets/gifs/pr-requirements.gif)
  [Documentation →](docs/contributing/documentation.md)
```

## Related

- [GOVERNANCE.md](../../GOVERNANCE.md)
- [CLAUDE.md](../../CLAUDE.md)
