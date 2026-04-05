# Contributing to AgilePlus

Guidelines and tools for contributing to the AgilePlus project.

## Quick Links

| Resource | Purpose |
|----------|---------|
| [Documentation Guide](./documentation.md) | How to write feature documentation |
| [Recording Visuals](./recording-visuals.md) | Tools for creating GIFs/screenshots |
| [pvalidate Tool](./pvalidate.md) | Validate PR requirements locally |

## PR Requirements

All pull requests must include:

1. **Visual Evidence** - Screenshot or GIF demonstrating the feature
2. **Completed Spec** - kitty-spec with all work packages done
3. **Documentation** - Page at `docs/<category>/<feature>.md`
4. **Changelog Entry** - With visual preview link

See [PR Requirements Policy](../../GOVERNANCE_PR_REQUIREMENTS.md) for full details.

## Development Workflow

1. Create spec: `agileplus specify --title "<feature>"`
2. Work in feature worktree
3. Run tests and validation
4. Create documentation with GIFs
5. Validate with `./bin/pvalidate --feature <id>`
6. Submit PR with visual evidence

## Tools

### pvalidate

Validate your feature is PR-ready:

```bash
./bin/pvalidate --feature 023
```

### Recording GIFs

```bash
# UI features
brew install kap

# CLI features
brew install charmbracelet/tap/vhs
```

See [Recording Visuals](./recording-visuals.md) for detailed workflows.

## Getting Help

- Review [existing specs](../kitty-specs/) for examples
- Check [documentation examples](../dashboard/service-controls.md)
- Run `./bin/pvalidate --help` for validation options
