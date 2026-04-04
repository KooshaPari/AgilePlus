---
id: FR-PHENOSDK-004
title: phenoSDK LLM Package + PyO3 Rust Layer
status: specified
priority: P1
created: 2026-03-25
category: sdk
owner: phenosdk-team
source: kitty-specs/phenosdk-decompose-llm
---

# FR-PHENOSDK-004: phenoSDK LLM Package + PyO3 Rust Layer

## Description

Extract pheno-llm package from monolith with high-value CPU-intensive components (ensemble routing 884 LOC, context folding 489 LOC). Optional Rust acceleration via PyO3.

## Package Contents

- `pheno-llm`: routing/, optimization/, protocol/ modules
- Pure Python implementation first
- PyO3 optional feature: pheno-llm-rs crate

## PyO3 Target Functions

- `ensemble_router.route()` — voting/scoring hot loop
- `context_folder.fold()` — tokenizer string manipulation
- Expose as: `from pheno_llm_rs import route, fold`

## Acceptance Criteria

- [ ] pheno-llm package with routing, optimization, protocol modules
- [ ] pheno-llm depends on pheno-core only
- [ ] Pure Python implementation functional
- [ ] PyO3 optional feature for acceleration
- [ ] Benchmark: 10x speedup target for routing
- [ ] Published to Phenotype GitHub Packages

## Notes

Original: `kitty-specs/phenosdk-decompose-llm/`
