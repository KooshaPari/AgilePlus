# pvalidate - PR Requirements Validator

Validates that a feature is ready for PR submission.

## Installation

The tool is already available in the repository:

```bash
./bin/pvalidate --feature <feature-id>
```

To install globally (optional):

```bash
# Add to PATH
export PATH="$PATH:/path/to/agileplus/bin"

# Or symlink
ln -s /path/to/agileplus/bin/pvalidate ~/.local/bin/pvalidate
```

## Usage

### Basic Check

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
./bin/pvalidate --feature 023
```

### Specific Checks

```bash
# Check only spec completeness
./bin/pvalidate --feature 023 --check spec

# Check only documentation
./bin/pvalidate --feature 023 --check docs

# Check only changelog
./bin/pvalidate --feature 023 --check changelog

# Check only visual assets
./bin/pvalidate --feature 023 --check visual
```

### Auto-Detect Feature

If run from a feature branch, pvalidate can auto-detect:

```bash
# From branch 023-pr-visual-requirements
./bin/pvalidate
# Auto-detects feature ID 023
```

## What It Checks

| Check | Validates |
|-------|-----------|
| **spec** | Spec exists at `kitty-specs/<feature>/` |
| | All work packages marked complete |
| | Status is "Complete" |
| **docs** | Documentation page exists in `docs/` |
| | Page linked in VitePress sidebar |
| **changelog** | Entry in `CHANGELOG.md` |
| | Visual reference included |
| **visual** | GIFs/screenshots in `docs/assets/` |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All checks passed, PR ready |
| 1 | One or more checks failed |

## Output Example

```
🔍 Validating PR requirements for feature: 023

📋 Checking specification...
✓ Spec exists at kitty-specs/023-pr-visual-requirements/spec.md
✓ Work packages: 8/8 complete
✓ Spec status is Complete

📖 Checking documentation...
✓ Documentation found: docs/contributing/documentation.md
⚠ Documentation may not be linked in VitePress sidebar

📝 Checking CHANGELOG...
✓ CHANGELOG.md entry found

🎬 Checking visual assets...
⚠ No GIFs found in docs/assets/gifs/

─────────────────────────────
Summary: 4 passed, 0 failed

✅ PR REQUIREMENTS MET
   You can create your PR now!
   Remember to embed your GIF/screenshot in the PR description.
```

## Integration

### Pre-Commit Hook

The pre-commit hook automatically reminds about PR requirements on feature branches.

### CI/CD

Add to your CI pipeline:

```yaml
- name: Validate PR Requirements
  run: ./bin/pvalidate --feature ${{ github.event.pull_request.title }}
```

## Troubleshooting

### "Spec not found"

- Verify feature ID (try shorter ID like `023` instead of `023-pr-visual-requirements`)
- Check spec exists at `kitty-specs/<feature>/spec.md`

### "No documentation found"

- Document can be anywhere in `docs/` that references the feature
- Must include the feature ID in content

### "No visual assets"

- Run from repo root (where `docs/assets/` exists)
- Create `docs/assets/gifs/` directory if missing

## Related

- [Documentation Guide](./documentation.md)
- [Recording Visuals](./recording-visuals.md)
- [PR Requirements Policy](../../GOVERNANCE_PR_REQUIREMENTS.md)
