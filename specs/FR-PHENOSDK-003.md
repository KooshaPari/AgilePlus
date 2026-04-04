---
id: FR-PHENOSDK-003
title: phenoSDK Core Package Extraction
status: specified
priority: P1
created: 2026-03-25
category: sdk
owner: phenosdk-team
source: kitty-specs/phenosdk-decompose-core
---

# FR-PHENOSDK-003: phenoSDK Core Package Extraction

## Description

Extract pheno-core package from phenoSDK monolith. Foundation modules (config, errors, logging, observability ports) become the minimal dependency for all other pheno-* packages.

## Problem

phenoSDK mixes 10+ domains in one monolith. The foundation should be extracted as pheno-core.

## Package Contents

- `pheno.config.core` (Config.from_env, from_file, load)
- `pheno.errors` (ZenMCPError hierarchy + retry/circuit breaker)
- `pheno.logging` (Console, File, JSON, Syslog, Structlog)
- `pheno.ports.observability` (Logger, Tracer, Meter, HealthChecker, Alerter)
- `pheno.ports.registry` (Registry, SearchableRegistry, ObservableRegistry)

## Acceptance Criteria

- [ ] New package: pheno-core with foundation modules
- [ ] All existing phenoSDK consumers updated to import from pheno-core
- [ ] pheno-core has pyproject.toml, tests, and CI entry
- [ ] phenoSDK depends on pheno-core (not vice versa)
- [ ] Backward compatibility maintained
- [ ] Published to Phenotype GitHub Packages

## Notes

Original: `kitty-specs/phenosdk-decompose-core/`
