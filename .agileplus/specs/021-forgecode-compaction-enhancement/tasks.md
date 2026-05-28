# Tasks — Forgecode Compaction Enhancement

## Overview

Enhance the forgecode context compaction system with:
1. LLM-based semantic summarization (hybrid approach)
2. Adaptive eviction windows
3. Importance-based message preservation
4. Pre-compaction filtering
5. Metrics and observability

**Total Estimate:** 20 hours

---

## Phase 1 — Configuration & Core Types

### T1.1: Extend CompactConfig (4h)

**Status:** Open
**Assignee:** TBD
**Priority:** P1

**Deliverables:**
- [ ] Add `SummarizationStrategy` enum (Extract, Llm, Hybrid)
- [ ] Add `enable_prefilter: bool`
- [ ] Add `enable_adaptive_eviction: bool`
- [ ] Add `enable_importance_scoring: bool`
- [ ] Add `summary_max_tokens: Option<usize>`
- [ ] Add `summary_model: Option<ModelId>`
- [ ] Add `summary_timeout_secs: u64`

**Files:**
- `crates/forge_config/src/compact.rs`
- `crates/forge_domain/src/compact/compact_config.rs`

**Verification:**
- [ ] Config parses new fields correctly
- [ ] Default values work for backward compatibility

---

### T1.2: Create CompactionHistory (2h)

**Status:** Open
**Assignee:** TBD
**Priority:** P1

**Deliverables:**
- [ ] `CompactionHistory` struct
- [ ] `summary_hashes: Vec<u64>`
- [ ] `file_versions: HashMap<PathBuf, String>`
- [ ] `compaction_count: usize`
- [ ] `total_tokens_reduced: usize`
- [ ] `record_compaction()` method

**Files:**
- `crates/forge_domain/src/compact/history.rs`
- `crates/forge_domain/src/compact/mod.rs`

---

### T1.3: Create ImportanceScore Types (2h)

**Status:** Open
**Assignee:** TBD
**Priority:** P2

**Deliverables:**
- [ ] `MessageImportance` struct
- [ ] `ImportanceFactor` enum
- [ ] `calculate()` function
- [ ] `MIN_SURVIVAL_SCORE: u8 = 60`
- [ ] Integration with `ContextMessage`

**Files:**
- `crates/forge_domain/src/compact/importance.rs`
- `crates/forge_domain/src/context.rs`

---

## Phase 2 — Eviction Strategy

### T2.1: Adaptive Eviction Window (2h)

**Status:** Open
**Assignee:** TBD
**Priority:** P1

**Deliverables:**
- [ ] `adaptive_eviction()` function
- [ ] Tiered eviction percentages:
  - >95% threshold: 50% eviction
  - >85% threshold: 35% eviction
  - >70% threshold: 20% eviction
  - default: 10% eviction
- [ ] Integration with existing `CompactionStrategy`

**Files:**
- `crates/forge_domain/src/compact/strategy.rs`

---

### T2.2: Importance-Based Range Finding (1h)

**Status:** Open
**Assignee:** TBD
**Priority:** P2

**Deliverables:**
- [ ] Filter protected messages from eviction candidates
- [ ] Preserve messages with importance score >= 60
- [ ] Maintain tool call atomicity

**Files:**
- `crates/forge_domain/src/compact/strategy.rs`

---

## Phase 3 — LLM Summarization

### T3.1: Summarization Prompt Template (1h)

**Status:** Open
**Assignee:** TBD
**Priority:** P1

**Deliverables:**
- [ ] `templates/forge-summarization-prompt.md`
- [ ] Structured prompt with sections:
  - Decisions
  - Files Changed
  - Operations Summary
  - Discovered Constraints
  - Current State

**Files:**
- `templates/forge-summarization-prompt.md`

---

### T3.2: Implement LlmSummarizer (4h)

**Status:** Open
**Assignee:** TBD
**Priority:** P1

**Deliverables:**
- [ ] `LlmSummarizer` struct
- [ ] `summarize()` async function
- [ ] Model selection (compact model or agent model)
- [ ] Timeout handling (default 3s)
- [ ] Error handling with fallback

**Files:**
- `crates/forge_app/src/services/summarizer.rs`
- `crates/forge_app/src/lib.rs`

---

### T3.3: Integrate into Compactor (3h)

**Status:** Open
**Assignee:** TBD
**Priority:** P1

**Deliverables:**
- [ ] Add summarization strategy handling to `compact()`
- [ ] Extract mode: current behavior
- [ ] LLM mode: full LLM summarization
- [ ] Hybrid mode: extract then refine
- [ ] Fallback to extract on LLM failure

**Files:**
- `crates/forge_app/src/compact.rs`

---

## Phase 4 — Pre-Compaction Filtering

### T4.1: PreCompactionFilter (2h)

**Status:** Open
**Assignee:** TBD
**Priority:** P2

**Deliverables:**
- [ ] `PreCompactionFilter` struct
- [ ] `filter()` function
- [ ] `collapse_duplicates()` function
- [ ] Minimum tool result length (default: 10 chars)
- [ ] Debug pattern removal (configurable)

**Files:**
- `crates/forge_app/src/transformers/prefilter.rs`
- `crates/forge_app/src/transformers/mod.rs`

---

## Phase 5 — Templates & Output

### T5.1: Enhanced Summary Frame (1h)

**Status:** Open
**Assignee:** TBD
**Priority:** P2

**Deliverables:**
- [ ] `templates/forge-partial-summary-frame-v2.md`
- [ ] Support both structured and LLM content
- [ ] Compact format with key sections
- [ ] Backward compatible with v1

**Files:**
- `templates/forge-partial-summary-frame-v2.md`

---

## Phase 6 — Metrics

### T6.1: CompactionMetrics (1h)

**Status:** Open
**Assignee:** TBD
**Priority:** P2

**Deliverables:**
- [ ] `CompactionMetrics` struct
- [ ] Track: compaction_count, total_tokens_reduced
- [ ] Track: strategies_used, errors
- [ ] `record()` method

**Files:**
- `crates/forge_domain/src/compact/metrics.rs`
- `crates/forge_domain/src/compact/mod.rs`

---

### T6.2: Metrics Integration (1h)

**Status:** Open
**Assignee:** TBD
**Priority:** P2

**Deliverables:**
- [ ] Integrate metrics collection into `Compactor`
- [ ] Record after each compaction
- [ ] Expose metrics via API

**Files:**
- `crates/forge_app/src/compact.rs`
- `crates/forge_app/src/api.rs`

---

## Task Dependencies

```
T1.1 ──┬── T1.2 ──┬── T2.1 ──┬── T3.3 ──┬── T6.2
       │          │          │
       │          │          └── T3.1 ──┘
       │          │
       │          └── T2.2
       │
       └── T1.3 ── T2.2

T3.2 ─── T3.3
 │
 └── T3.1

T4.1 ──┬── T5.1
       │
       └── T6.1
```

---

## Verification Checklist

### Unit Tests
- [ ] Test adaptive eviction window calculation
- [ ] Test importance score calculation
- [ ] Test pre-filter removes short tool results
- [ ] Test deduplication of consecutive tool calls
- [ ] Test LLM summarizer (mocked)

### Integration Tests
- [ ] Test compaction with Extract strategy
- [ ] Test compaction with LLM strategy (mocked)
- [ ] Test compaction with Hybrid strategy
- [ ] Test fallback on LLM failure

### Manual Testing
- [ ] Compact conversation with 50 messages
- [ ] Verify tool call atomicity preserved
- [ ] Verify reasoning chain preserved
- [ ] Compare Extract vs Hybrid output quality
