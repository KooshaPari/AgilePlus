# Feature Specification: devenv-abstraction Archive

**Spec ID**: 009-devenv-abstraction-archive
**Created**: 2026-04-02
**Status**: Completed
**Mission**: infrastructure
**Repository**: devenv-abstraction
**Superseded by**: 008-nanovms-completion

## Overview

The `devenv-abstraction` project has been **archived and renamed** to **[NanoVMS](https://github.com/KooshaPari/nanovms)** (Nano Virtual Machine Services).

## Rationale for Rename

| Reason | Description |
|--------|-------------|
| **Clarity** | "devenv" is a common pattern; "nanovms" is distinctive |
| **Brand** | "Nano" conveys lightweight, "VMS" clearly states the purpose |
| **Scope** | The new name better reflects the two-level abstraction (Infrastructure + Platform) |
| **Uniqueness** | No conflicts with other "devenv" projects in the ecosystem |

## Migration Path

### For Users

```bash
# Old (deprecated)
git clone https://github.com/KooshaPari/devenv-abstraction.git

# New (active)
git clone https://github.com/KooshaPari/nanovms.git
```

### For Contributors

All development continues in the [NanoVMS repository](https://github.com/KooshaPari/nanovms).

## Changes from devenv-abstraction to NanoVMS

### Architecture Changes

| Aspect | devenv-abstraction (Old) | NanoVMS (New) |
|--------|-------------------------|---------------|
| **Abstraction Level** | Single layer (platform adapters) | Two levels (Infrastructure + Platform) |
| **VM Flavors** | Platform-specific (`macos`, `ios`, `android`) | Infrastructure (`native`, `lima`, `wsl`, `microvm`, `wasm`) |
| **Platform Targets** | Limited | Full roadmap (Apple, Android, AR/VR, Gaming, IoT) |
| **Status** | Active | Active |

### File Changes

| File | Action |
|------|--------|
| `README.md` | Updated with deprecation notice, redirect to NanoVMS |
| `SPEC.md` | Updated with deprecation notice, link to NanoVMS spec |
| `AGENTS.md` | Updated with rename notice |
| GitHub repo | Remains read-only for history |

## Completion Criteria

- [x] `devenv-abstraction/README.md` updated with deprecation notice
- [x] `devenv-abstraction/SPEC.md` updated with deprecation notice
- [x] `nanovms/` repository created with updated architecture
- [x] `nanovms/SPEC.md` reflects actual code implementation
- [x] `nanovms/README.md` reflects actual architecture
- [x] AgilePlus spec created for NanoVMS (008-nanovms-completion)

## Notes

- The `devenv-abstraction` repository is kept for historical reference and git history
- All future development will occur in the `nanovms` repository
- The AgilePlus spec `008-nanovms-completion` tracks ongoing NanoVMS development

---

*This spec is completed. NanoVMS (formerly devenv-abstraction) is now active at https://github.com/KooshaPari/nanovms*
