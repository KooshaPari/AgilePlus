# traces/ (relocated)

FR trace metadata folded into [`docs/journeys/`](../docs/journeys/) YAML frontmatter.
Archived JSON lives under [`docs/_archive/traces-json/`](../docs/_archive/traces-json/).
Coverage is derived from in-code `#[trace_fr(spec = ..., fr = ...)]` annotations (see
[`docs/adr/0004-json-to-frontmatter-decorators.md`](../docs/adr/0004-json-to-frontmatter-decorators.md)).

Hand-maintained `FR-*.json` trace files were folded into journey frontmatter under
[`docs/journeys/`](../docs/journeys/). Archived JSON copies:
[`docs/_archive/traces-json/`](../docs/_archive/traces-json/).

Schema and matrix docs: [`docs/requirements/traceability/`](../docs/requirements/traceability/).

Machine-generated trace matrices belong in build output (e.g. `target/traceability/`).
