# AgilePlus Bin Tools

Command-line tools for the AgilePlus project.

## Available Tools

| Tool | Purpose | Usage |
|------|---------|-------|
| `pvalidate` | Validate PR requirements | `./bin/pvalidate --feature <id>` |

## pvalidate

Validates that a feature is ready for PR submission by checking:

- ✅ Spec exists at `kitty-specs/<feature>/`
- ✅ All work packages marked complete
- ✅ Documentation page exists
- ✅ CHANGELOG.md entry present
- ✅ Visual assets (GIFs/screenshots) available

### Installation

No installation needed - run directly from repo root:

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
./bin/pvalidate --feature 023
```

### Documentation

See [docs/contributing/pvalidate.md](../docs/contributing/pvalidate.md) for full documentation.

## Contributing

Add new tools here with:
1. Executable script in this directory
2. Documentation in `docs/contributing/`
3. Entry in this README
