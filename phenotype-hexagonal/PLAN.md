# Phenotype Hexagonal Implementation Plan

## Overview

Consolidate hexagonal architecture implementations into a unified, multi-language framework.

## Current State

**Separate Repos:**
- HexaGo (Go implementation)
- HexaPy (Python implementation)
- HexaType (TypeScript implementation)
- Hexacore (Rust implementation)
- hexagon-python, hexagon-rs, hexagon-ts (ARCHIVED)

## Target State

**Unified Repo:** `phenotype-hexagonal`
- `rust/` - Hexacore + HexaGo adapters
- `go/` - HexaGo
- `python/` - HexaPy
- `typescript/` - HexaType
- `docs/` - Cross-language documentation
- `examples/` - Multi-language examples

## Phases

### Phase 1: Repository Migration (Week 1)

**Day 1-2: Rust Migration**
- [ ] Copy Hexacore to `rust/core/`
- [ ] Copy HexaGo Rust parts to `rust/adapters/`
- [ ] Create workspace Cargo.toml
- [ ] Set up CI

**Day 3-4: Go Migration**
- [ ] Copy HexaGo to `go/`
- [ ] Reorganize structure
- [ ] Update go.mod
- [ ] Set up CI

**Day 5: Python Migration**
- [ ] Copy HexaPy to `python/`
- [ ] Reorganize structure
- [ ] Update pyproject.toml
- [ ] Set up CI

### Phase 2: TypeScript Migration & Standardization (Week 2)

**Day 1-2: TypeScript Migration**
- [ ] Copy HexaType to `typescript/`
- [ ] Reorganize structure
- [ ] Update package.json

**Day 3-4: Cross-Language Documentation**
- [ ] Create unified documentation
- [ ] Document patterns across languages
- [ ] Create comparison matrix

**Day 5: Archive Old Repos**
- [ ] Archive HexaGo, HexaPy, HexaType, Hexacore
- [ ] Archive hexagon-python, hexagon-rs, hexagon-ts
- [ ] Add deprecation notices

### Phase 3: Integration & Examples (Week 3)

**Week 3: Create Examples**
- [ ] E-commerce example in all 4 languages
- [ ] API Gateway example
- [ ] Event-driven example
- [ ] Documentation site

### Phase 4: Polish & Release (Week 4)

**Week 4:**
- [ ] Final testing
- [ ] Documentation completion
- [ ] Version alignment
- [ ] Release v1.0.0

## Resource Estimate

- 1 engineer, 4 weeks
- Cross-language expertise helpful
- Can be parallelized by language

## Benefits

1. **Unified Documentation:** Single source for all languages
2. **Consistent Patterns:** Same concepts across languages
3. **Easier Maintenance:** One repo instead of 4+
4. **Cross-Language Examples:** See same problem solved in multiple ways
5. **Reduced Complexity:** Clearer structure

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking existing users | Gradual migration, keep old repos until v1.0.0 |
| Git history loss | Preserve in archived repos, document migration |
| Language-specific complexity | Clear separation in directory structure |
| Testing complexity | Per-language CI pipelines |
