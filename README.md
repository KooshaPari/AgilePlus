# agileplus-spec-harmonizer

Harmonize work packages from **GSD**, **OpenSpec**, **BMAD-Method**, and **Spec Kitty** into a single normalized shape, then emit as NDJSON (TRAC-aligned) or a Markdown index.

Designed to be the front of the AgilePlus SDD pipeline:

```
gsd/openspec/bmad/kitty specs  →  harmonize  →  WorkPackage[]  →  seed-requirements (Tracera)  →  phenodag tasks
```

## What it does

Each spec format has its own quirks (GSD uses `## Task N:` + checkboxes, OpenSpec uses `## Spec <id> — <title>` + `## Acceptance`, etc.). This crate parses all four into one `WorkPackage`:

```rust
pub struct WorkPackage {
    pub id: String,             // "gsd-1", "openspec-ABC-1", "bmad-S1", "kitty-K-1"
    pub title: String,
    pub description: String,
    pub acceptance: Vec<AcceptanceCriterion>,
    pub source_format: String,  // "gsd" | "openspec" | "bmad" | "kitty"
    pub source_anchor: String,  // original spec anchor (e.g. "1", "ABC-1")
}
```

Then `normalize` (slug, stable 16-char hash, merge) + `emit` (NDJSON or Markdown) round out the pipeline.

## Quick start

```rust
use agileplus_spec_harmonizer::{parsers::Parser, parsers::gsd::GsdParser};

let text = std::fs::read_to_string("spec.md").unwrap();
let pkgs = GsdParser.parse(&text).unwrap();
println!("parsed {} GSD tasks", pkgs.len());
```

Or as a binary:

```bash
$ cargo run -- harmonize fixtures/gsd_sample.md --format gsd
parsed 3 GSD tasks:
  gsd-1: Bootstrap repo (3 acceptance, 1 done)
  gsd-2: Add CLI entrypoint (4 acceptance, 2 done)
  gsd-3: Persist state (2 acceptance, 2 done)
```

## Format details

| Format | Heading | Acceptance block | Source anchor |
|---|---|---|---|
| GSD | `## Task N: <title>` | `- [ ]` / `- [x]` bullets | `N` |
| OpenSpec | `## Spec <id> — <title>` (or `:`, `-`) | `## Acceptance` block | `<id>` |
| BMAD | `## Story <id>: <title>` | `## Criteria` block | `<id>` |
| Spec Kitty | `## Spec <id> - <title>` (hyphen) | `## Acceptance` block | `<id>` |

## Why "harmonize"?

Four formats, four sets of toolchains, four sets of "what does done mean". The harmonizer is the choke point: once you have `WorkPackage[]`, the rest of the SDD pipeline (Tracera seed-requirements → phenodag tasks) is format-agnostic.

## Tests

```bash
$ cargo test
running 11 tests
test src::parsers::gsd::tests::errors_when_no_heading ... ok
test src::parsers::gsd::tests::parses_two_gsd_tasks_with_acceptance ... ok
test src::parsers::openspec::tests::parses_openspec_with_acceptance ... ok
test src::parsers::bmad::tests::parses_bmad_story_with_criteria ... ok
test src::parsers::kitty::tests::parses_kitty_spec_hyphen_separator ... ok
test src::emit::tests::ndjson_one_line_per_package ... ok
test src::emit::tests::markdown_groups_by_format ... ok
test src::normalize::tests::slug_strips_separators ... ok
test src::normalize::tests::stable_hash_is_deterministic ... ok
test src::normalize::tests::merge_picks_more_acceptance ... ok
test tests::integration::tests::parses_fixture_gsd ... ok
test result: ok. 11 passed; 0 failed
```

## License

MIT
