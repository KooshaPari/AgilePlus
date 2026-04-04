# PR Requirements Policy

**Effective Date:** 2026-04-04  
**Applies to:** All repositories in the Phenotype ecosystem

## Mandatory PR Checklist

Every pull request MUST include the following before review:

### 1. Visual Evidence (REQUIRED)

All PRs must include embedded visual evidence:

| Type | Format | Tooling |
|------|--------|---------|
| UI/Frontend | GIF/Screenshot | GitHub upload, CleanShot, Kap |
| CLI/Terminal | GIF or asciinema | vhs, asciinema, terminal GIF |
| API/Backend | Screenshot of response/docs | Insomnia/Postman screenshot |
| Workflow | Diagram or GIF | Excalidraw, GIF recording |

**Why:** Visual evidence allows stakeholders to understand changes without running the application. It creates a visual changelog that can be browsed historically.

**Placement:** Embed directly in PR description using GitHub's image upload or referencing committed assets in `.github/assets/`.

### 2. Completed Specification (REQUIRED)

All feature PRs must reference a completed kitty-spec:

- [ ] Spec exists at `kitty-specs/<feature-id>/`
- [ ] All work packages in `spec.md` marked complete
- [ ] Implementation aligns with spec acceptance criteria
- [ ] Any spec deviations documented in PR notes

**Why:** Ensures features are properly designed before implementation and provides traceability from requirement to code.

### 3. Documentation Page (REQUIRED)

Every feature must have a corresponding documentation page:

- [ ] Page created at `docs/<category>/<feature-id>/` or appropriate path
- [ ] Page includes:
  - Feature description and purpose
  - Usage instructions
  - Visual examples (screenshots/GIFs)
  - API reference (if applicable)
  - Links to related features
- [ ] Page linked in VitePress sidebar navigation
- [ ] Cross-reference in spec's `spec.md` → documentation section

**Why:** Creates a browsable feature catalog. Users and developers can discover capabilities through documentation rather than exploring the codebase.

### 4. Changelog Entry (REQUIRED for user-facing changes)

- [ ] Entry added to `CHANGELOG.md`
- [ ] Entry includes visual preview link when applicable
- [ ] Follows CalVer format: `YEAR.MONTH(WAVE).PATCH`

## Enforcement

### Automated Checks

The following workflows validate PR requirements:

1. **pr-requirements-check.yml** - Validates checklist completion
2. **docs-link-check.yml** - Validates documentation page exists and is linked
3. **spec-completion-check.yml** - Validates referenced spec is complete

### Review Process

1. PR author completes all checklist items
2. CI validates automated requirements
3. Maintainer verifies visual evidence and documentation quality
4. Approval requires all checkboxes checked

## Visual Changelog Navigation

The CHANGELOG.md serves as a visual timeline:

```markdown
## [2026.04A.0] - 2026-04-04

### Features

- **dashboard**: Add service control panel (#200)
  ![service-controls](docs/assets/gifs/service-controls.gif)
  [Documentation →](docs/dashboard/service-controls.md)

- **cli**: New `agileplus status` command (#201)  
  ![status-command](docs/assets/gifs/status-command.gif)
  [Documentation →](docs/cli/status-command.md)
```

This creates a scannable visual history where users can:
1. See features in action via GIFs
2. Navigate to documentation for each feature
3. Understand the product evolution visually

## Template for Feature Documentation Page

Create `docs/<category>/<feature-id>.md`:

```markdown
# <Feature Name>

Visual demo: ![demo](./assets/<feature>.gif)

## Overview

Brief description of what this feature does and why it exists.

## Usage

\`\`\`bash
# Example commands or code
agileplus <command>
\`\`\`

## Visual Walkthrough

![walkthrough](./assets/<feature>-walkthrough.gif)

## API Reference

If applicable, document API endpoints, parameters, responses.

## Related

- [Parent Feature](./parent-feature.md)
- [Spec: kitty-specs/<feature-id>/](../kitty-specs/<feature-id>/spec.md)
```

## Exceptions

The following PR types may skip specific requirements:

| PR Type | Visual Evidence | Spec | Docs | Changelog |
|---------|-----------------|------|------|-----------|
| Bug fix (no UI change) | Optional | N/A | Update if needed | Yes |
| Chore/deps | No | N/A | No | Optional |
| Hotfix | Post-merge | N/A | Post-merge | Yes |
| Refactor (no behavior change) | Optional | N/A | Update if needed | Optional |

## Migration Path

For existing features without documentation:

1. Create documentation page retroactively
2. Add to changelog in next release
3. Reference in retrospective PR

## Related

- [Contributing Guide](../CONTRIBUTING.md)
- [Documentation Guide](docs/contributing/documentation.md)
- [Spec Format](docs/contributing/specs.md)
