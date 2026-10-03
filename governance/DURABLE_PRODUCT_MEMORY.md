# Durable Product Memory / Genesis Doctrine

**Status:** controlling documentation doctrine for Tracera and AgilePlus specification programs.  
**Date:** 2026-09-29.

## Problem

The product program spans many repositories, agents, chats, years, aliases, discarded approaches, experiments and large user-authored intent dumps.

Chat/model memory is not the authoritative long-term memory system.

The limiting failure is not only "an agent may forget." The human sponsor/operator cannot be expected to retain:
- every historical rationale;
- every alternative already investigated;
- every subtle constraint;
- every naming lineage;
- every rejected architecture and why;
- every experiment and its interpretation;
- the prerequisite reasoning that originally produced a decision.

If this knowledge exists only in chat history, memory summaries, or one person's recollection, it is at risk of effective loss.

## Doctrine

Repository + registry documentation is **durable external product memory**.

It must support two recovery targets:

### Agent recovery

A capable new agent with no prior chat context can reconstruct enough accepted product meaning, history, evidence and work boundaries to continue without silently re-solving or undoing prior decisions.

### Human recovery

The product owner can return after months or years, having retained only the broad product idea, and recover:
- what the product is;
- why it exists;
- how the idea evolved;
- what alternatives were considered;
- what was decided;
- why;
- what remains uncertain;
- what was tried;
- what failed;
- how current architecture follows from that history.

Human recoverability is a first-class acceptance criterion, not a side effect of agent-readable docs.

## Genesis class

Some artifacts are **Genesis documents**: unusually high-fidelity records of product origin and governing intent.

Genesis should preserve:
- original problem and personal pain that motivated the product;
- earliest known formulation;
- historical names/aliases;
- major user-authored intent dumps, faithfully summarized and linked to raw provenance where available;
- evolution of the thesis over time;
- major conceptual breakthroughs;
- constraints/invariants that repeatedly survived redesign;
- abandoned interpretations and why;
- relationship to contemporaneous external work without falsely claiming inspiration;
- unresolved philosophical/product questions.

Genesis is not a README and not a chronological transcript dump. It is a curated reconstruction that makes the product's conceptual lineage recoverable.

## Dossier class

A product dossier is the high-level recoverability hub.

It should answer:
- identity / aliases;
- one-paragraph and deep product definitions;
- canonical role and ownership boundary;
- Genesis;
- accepted intent;
- mature horizon;
- current state;
- stages/CVP;
- architecture;
- ontology;
- SOTA/alternatives;
- research corpus;
- decisions/ADRs;
- requirements/traceability;
- journeys;
- quality model;
- oracle/MACE;
- implementation map;
- known gaps/findings;
- experiments;
- pilot/case-study plan;
- release/lifecycle;
- work/development relationships;
- glossary;
- source/provenance index;
- "where do I start?" reading paths.

The dossier may be an index/folder rather than one giant document.

## Specialized documents must be substantive

Files such as:
- INTENT
- GENESIS
- SOTA
- CVP
- BOUNDARY
- CURRENT-STATE
- ARCHITECTURE
- ONTOLOGY
- JOURNEYS
- MACE/AUTOGRADER
- PILOT
- COMPLETION-GATE
- INTERNAL-ARCHAEOLOGY
- SOURCE-COVERAGE
- DECISION-LOG
- GLOSSARY

must not exist merely to satisfy a checklist.

A shallow placeholder is worse than an explicit missing document because it creates false confidence that context has been preserved.

Each specialized artifact should contain the depth warranted by the product and cross-link to evidence rather than duplicating unstable facts blindly.

## Preserve rationale, not private chain-of-thought

The durable record should capture **decision rationale and evidence**:
- problem;
- alternatives;
- observations;
- tradeoffs;
- decision;
- consequences;
- confidence;
- unresolved questions.

It does not need hidden/private model reasoning. It needs enough explicit rationale for another human or agent to independently reconstruct why the decision is defensible.

## Raw intent preservation

Large user-authored product-intent dumps are high-value source artifacts.

When practical:
1. preserve a faithful source record or durable citation/reference;
2. create a curated synthesis;
3. distinguish verbatim/raw user intent from assistant interpretation;
4. extract stable invariants and open questions;
5. update Genesis/Intent/Dossier;
6. record what architecture/specification changed because of the input.

Do not rely on future memory retrieval to rediscover the dump.

## Decision supersession

Never silently rewrite history.

When a decision changes:
- retain the old decision;
- mark superseded;
- state replacement;
- explain why evidence/intent changed;
- identify affected requirements/architecture/work;
- link the new decision.

A new agent should be able to distinguish "we never considered this" from "we considered and rejected this for reason X."

## Documentation graph

Docs themselves should participate in traceability:

```text
raw intent/source
  → Genesis / Intent synthesis
  → product decision / ADR
  → architecture / ontology
  → requirement
  → implementation
  → verification/evidence
```

Reverse traversal should explain why an implementation/requirement exists.

## Recoverability tests

Before a repository can reach specification/design completion, run fresh-context recovery reviews.

### Human-oriented test

Given only the dossier and linked repo/registry docs, can a technically capable product owner who remembers only the big picture explain:
- what the product is and is not;
- why it exists;
- its key historical pivots;
- current architecture and why;
- major alternatives and why not;
- current maturity and blockers;
- what to do next?

### Agent-oriented test

A fresh agent receives the dossier without conversation history and must:
- reconstruct the product boundary;
- identify accepted vs proposed architecture;
- avoid a known rejected approach;
- locate the next valid work frontier;
- cite the evidence/rationale supporting those conclusions.

Failures become documentation defects.

## Freshness and drift

Each important document should expose:
- status: accepted/proposed/historical/superseded/incomplete;
- last substantive review date;
- source revision where applicable;
- dependencies/related docs;
- known stale sections if any.

Automated checks can detect broken links/schema/staleness indicators, but semantic freshness requires review.

## Registry vs repository

### Product repository
Owns detailed canonical product contract, architecture, requirements, implementation mappings and executable verification assets.

### PhenoRegistry
Owns cross-repository identity, lineage/indexing, portfolio context, high-level dossier navigation, shared research and durable pointers into repo-local truth.

Do not duplicate huge mutable specifications in both places.

## Completion gate

No repository reaches 100% specification/design completion while:
- major user intent exists only in chat;
- Genesis is materially incomplete;
- important aliases/history are unrecovered;
- key decisions lack rationale;
- specialized dossier documents are placeholders;
- current architecture cannot be traced to intent/research/decisions;
- a fresh human/agent recovery exercise fails;
- superseded decisions are indistinguishable from current ones.

Documentation completeness is not measured by file count. It is measured by **recoverability and fidelity**.
