<!-- work-state: integration/consolidate | 2026-06-15 | gaps=8/8 CLOSED | build=PASS -->
[██████████] 10/10 — All w35 audit gaps closed; PR-ready for main

<!-- AI-DD-META:START -->
<!-- This repository is planned, maintained, and managed by AI Agents only. -->
<!-- Slop issues are expected and intentionally present as part of an HITL-less -->
<!-- /minimized AI-DD metaproject of learning, refining, and building brute-force -->
<!-- training for both agents and the human operator. -->
![Downloads](https://img.shields.io/github/downloads/KooshaPari/AgilePlus/total?style=flat-square&label=downloads&color=blue)
![GitHub release](https://img.shields.io/github/v/release/KooshaPari/AgilePlus?style=flat-square&label=release)
![CI](https://img.shields.io/github/actions/workflow/status/KooshaPari/AgilePlus/ci.yml?branch=main&style=flat-square&label=CI)
![License](https://img.shields.io/github/license/KooshaPari/AgilePlus?style=flat-square)
![AI-Slop](https://img.shields.io/badge/AI--DD-Slop%20Expected-orange?style=flat-square)
![AI-Only-Maintained](https://img.shields.io/badge/Planned%20%26%20Maintained%20by-AI%20Agents%20Only-red?style=flat-square)
![HITL-less](https://img.shields.io/badge/HITL--less%20AI--DD-metaproject-yellow?style=flat-square)

> ⚠️ **AI-Agent-Only Repository**
>
> This repo is **planned, maintained, and managed exclusively by AI Agents**.
> Slop issues, rough edges, and AI artifacts are **expected and intentionally
> present** as part of an **HITL-less / minimized AI-DD** metaproject focused
> on learning, refining, and brute-force training both the agents and the
> human operator. Bug reports and contributions are still welcome, but please
> expect AI-generated code, comments, and documentation throughout.
<!-- AI-DD-META:END -->
> **Work state:** ACTIVE · **Progress:** `███████░░░ 70%`
> AI-native spec-driven PM platform (Rust workspace + React/TS dashboard + Electrobun desktop); frontend candidate #1. Core domain/api/dashboard implemented; CI partially red. · updated 2026-06-02

# AgilePlus

![Scorecard](https://api.securityscorecards.dev/projects/github.com/KooshaPari/AgilePlus/badge)

> **Pinned references (Phenotype-org)**
> - MSRV: see `rust-toolchain.toml`
> - cargo-deny config: see `deny.toml`
> - cargo-audit: `rustsec/audit-check@v2` weekly
> - Branch protection: 1 reviewer required, no force-push
> - Branching baseline: canonical checkout stays on `main` unless doing merge/pull
> - Governance authority: `phenotype-org-governance/SUPERSEDED.md` when present

**Local-first, AI-native, spec-driven project management for agent + human teams.**

AgilePlus manages feature specs, work packages, and acceptance criteria with a hexagonal
Rust core, optional GitHub/Plane sync, P2P merge, a web dashboard, and a desktop app. It is
one of the Phenotype org's three project-management frontend candidates (alongside Tracera and
Planify).

## Architecture

AgilePlus is a Cargo workspace following hexagonal (ports-and-adapters) architecture: the
`domain` and `application` crates have no framework dependencies; everything else is an adapter.

| Crate | Role |
|-------|------|
| `agileplus-domain` | Core entities, invariants (no framework deps) |
| `agileplus-application` | Use-case layer (no framework deps) |
| `agileplus-api` | HTTP API surface |
| `agileplus-grpc` / `agileplus-proto` | gRPC layer + compiled tonic types |
| `agileplus-cli` / `agileplus-subcmds` | `agileplus` command-line client |
| `agileplus-dashboard` | Web dashboard (Askama + React/TS under `web/`) + Electrobun desktop |
| `agileplus-sqlite` | SQLite persistence adapter |
| `agileplus-events` / `agileplus-nats` | Event model + NATS transport |
| `agileplus-sync` | Sync orchestrator — conflict detection/resolution + NATS |
| `agileplus-p2p` | Peer-to-peer merge |
| `agileplus-github` / `agileplus-plane` / `agileplus-import` | External integrations + import |
| `agileplus-git` | Git integration |
| `agileplus-governance` | Release channels, audit logging, policy enforcement |
| `agileplus-config` | Shared config-builder macro |
| `agileplus-cache` / `agileplus-telemetry` / `agileplus-triage` | Cache, telemetry, triage |
| `agileplus-graph` / `agileplus-artifacts` / `agileplus-fixtures` | Graph, artifacts, fixtures |
| `agileplus-benchmarks` | Criterion performance benchmarks |
| `agileplus-contract-tests` / `agileplus-integration-tests` | Cross-crate test suites |

The Python `agileplus-mcp/` directory is a separate FastMCP server. `python/phenotype_traceability/`
holds the traceability package.

## Getting Started

```bash
# Build the workspace
cargo build --workspace

# Install and run the CLI
cargo install --path crates/agileplus-cli
agileplus --help

# Create a spec / feature
agileplus specify --title "<feature>" --description "<desc>"

# Web dashboard frontend
cd crates/agileplus-dashboard/web
bun install && bun run dev
```

## Development

- `main` is protected — all changes via PR. Branch prefixes: `feat/ fix/ chore/ ci/ docs/`.
- Keep PRs small and focused; fix all CI failures on a PR, including pre-existing ones.
- All files UTF-8, no BOM. Never commit agent dirs (`.claude/`, `.codex/`, `.cursor/`).
- Spec work is tracked in AgilePlus itself (`agileplus specify` / `agileplus status`).

## Quality Standards

- `cargo clippy --workspace -- -D warnings` (zero warnings)
- `cargo fmt` before commit
- Tests for new features; reproduce a bug with a failing test before fixing
- cargo-deny advisories (`deny.toml`) + weekly cargo-audit

## License

See [LICENSE](LICENSE).

## Documentation

This repository includes the following cross-cutting documents:

| Project | Description |
|---|---|
| [heliosApp](heliosApp/) | Product application workspace for helios. |
| [HeliosLab](HeliosLab/) | Experimentation and analytics workspace for helios. |
| [heliosBench](heliosBench/) | Benchmarking project for helios execution performance. |
| [BytePort](BytePort/) | Network transport and endpoint-oriented product. |
| [Tokn](Tokn/) | Token operations and pricing governance. |
| [Tracera](Tracera/) | Traceability system for event and execution history. |
| [Observably](Observably/) | Product-level observability surface. |
| [hwLedger](hwLedger/) | Hardware and capacity ledger for fleet/operations planning. |
| [PolicyStack](PolicyStack/) | Governance/policy engine and compliance tooling. |
| [Planify](Planify/) | Planning utilities and work lifecycle helpers. |
| [Sidekick](Sidekick/) | Assistant-side operational helper. |
| [Eidolon](Eidolon/) | Phenotype's Eidolon domain project. |

### Tooling and infrastructure

| Project | Description |
|---|---|
| [FocalPoint](FocalPoint/) | Central operations tooling, including target-pruner utilities. |
| [Configra](Configra/) | Configuration management framework. |
| [Conft](Conft/) | Flag/config control service. |
| [PhenoObservability](PhenoObservability/) | Logging, tracing, telemetry, and monitoring stack. |
| [ObservabilityKit](ObservabilityKit/) | Reusable observability building blocks. |
| [PlatformKit](PlatformKit/) | Cross-platform platform utilities. |
| [HexaKit](HexaKit/) | Architecture and scaffolding utilities. |
| [phenotype-infra](phenotype-infra/) | Infrastructure-as-code and org automation. |
| [phenoAI](phenoAI/) | AI service integrations and helpers. |
| [ValidationKit](ValidationKit/) | Validation and policy checks. |
| [TestingKit](TestingKit/) | Test scaffolding and quality helpers. |
| [rich-cli-kit](rich-cli-kit/) | Rich terminal UX helpers. |
| [ResilienceKit](ResilienceKit/) | Resilience patterns library. |
| [Tracely](Tracely/) | Trace explorer. |
| [Metron](Metron/) | Metric collection service. |
| [Tasken](Tasken/) | Task scheduler. |
| [Civis](Civis/) | Civis project. |
| [Benchora](Benchora/) | Benchmarking service. |
| [QuadSGM](QuadSGM/) | QuadSGM project. |
| [localbase3](localbase3/) | Local DB / storage layer. |
| [KDesktopVirt](KDesktopVirt/) | Desktop virtualization. |
| [bare-cua](bare-cua/) | Bare computer-use-agent harness. |
| [PlayCua](PlayCua/) | Computer-use agent playground. |
| [AppGen](AppGen/) | App generator. |
| [DevHex](DevHex/) | DevHex product. |
| [Dino](Dino/) | Dinoforge core. |
| [dinoforge-packs](dinoforge-packs/) | Dinoforge governance packs. |
| [DINOForge-UnityDoorstop](DINOForge-UnityDoorstop/) | Unity Doorstop integration. |
| [KlipDot](KlipDot/) | KlipDot product. |
| [Pine](Pine/) | Pine project. |
| [Parpoura](Parpoura/) | Parpoura project. |
| [Paginary](Paginary/) | Pagination service. |
| [netweave-final2](netweave-final2/) | Netweave experiment. |
| [foqos-private](foqos-private/) | Foqos (private). |
| [argis-extensions](argis-extensions/) | Argis editor extensions. |
| [AtomsBot](AtomsBot/) | Atoms bot (archived; retained for history). |
| [chatta](chatta/) | Chatta workspace. |
| [portage](portage/) | Portage workspace. |
| [portage-adapter-core](portage-adapter-core/) | Portage adapter core. |
| [nanovms](nanovms/) | NanoVM integration. |
| [GDK](GDK/) | Game/graphics dev kit. |
| [DataKit](DataKit/) | Data-pipeline kit. |
| [AuthKit](AuthKit/) | Auth kit. |
| [kwality](kwality/) | Quality / scorecard tooling. |
| [phenoUtils](phenoUtils/) | Shared utilities. |
| [phenoDesign](phenoDesign/) | Design system primitives. |
| [phenodocs](phenodocs/) | Phenotype documentation site. |
| [phenodocs-scorecard-remediation](phenodocs-scorecard-remediation/) | Doc scorecard remediation harness. |
| [PhenoHandbook](PhenoHandbook/) | Phenotype handbook. |
| [phenotype-auth-ts](phenotype-auth-ts/) | TypeScript auth client. |
| [phenotype-bus](phenotype-bus/) | Message bus. |
| [phenotype-hub](phenotype-hub/) | Discovery / registry hub. |
| [phenotype-registry](phenotype-registry/) | Artifact registry. |
| [phenotype-journeys](phenotype-journeys/) | User-journey traceability. |
| [phenotype-omlx](phenotype-omlx/) | OMLX integration. |
| [phenotype-ops-mcp](phenotype-ops-mcp/) | Ops MCP surface. |
| [phenotype-org-audits](phenotype-org-audits/) | Org-wide audit reports. |
| [phenotype-tooling](phenotype-tooling/) | Cross-repo tool chest. |
| [phenotype-icons](phenotype-icons/) | Icon set. |
| [phenotype-previews-smoketest](phenotype-previews-smoketest/) | Preview/smoketest harness. |
| [phenotype-skills](phenotype-skills/) | Claude skill definitions. |
| [phenoXdd](phenoXdd/) | XDD (cross-document development) tooling. |
| [phenoResearchEngine](phenoResearchEngine/) | Research automation engine. |

### Landing pages and web surfaces

| Project | Description |
|---|---|
| [agileplus-landing](agileplus-landing/) | Marketing/landing site for AgilePlus. |
| [byteport-landing](byteport-landing/) | Marketing/landing site for BytePort. |
| [hwledger-landing](hwledger-landing/) | Marketing/landing site for hwLedger. |
| [phenokits-landing](phenokits-landing/) | Marketing/landing site for PhenoKits. |
| [projects-landing](projects-landing/) | Organization-wide projects landing page. |
| [frontend](frontend/) | Shelf-level frontend app surface. |

### Cross-cutting shelves and references

| Path | Purpose |
|---|---|
| `docs/worklogs/` | Shelf-wide worklog index by category (`ARCHITECTURE`, `RESEARCH`, `GOVERNANCE`, etc.). |
| `AgilePlus/kitty-specs/` | Feature specs and task packages used by AgilePlus. |
| `docs/` | Shared references, ADRs, and reusable docs. |
| `*-wtrees/` | Project worktree directories (e.g. `AgilePlus-wtrees/`). |
| `.github/scripts/` | Shared CI and maintenance scripts. |
| `AgilePlus/` | Main platform implementation. |

## Notes

- Some directories are placeholders/landing/utility trees and may not represent standalone canonical products.
- This README is intentionally a shelf-level index; source-of-truth setup for each project stays in that project's own `README.md`.
- This index does not remove information from previous versions; it reorganizes it for polyrepo orientation and easier navigation.

---

## Rich Media Stubs

<!-- RICH-MEDIA-STUB type="annotated-screenshot" subject="AgilePlus quickstart — agileplus status after first epic created" journey="quickstart-cli" status="TODO" -->
> **[RICH MEDIA PLACEHOLDER — blocked on CLI build]** *Terminal capture of `agileplus status` immediately after `agileplus init` + `agileplus epic create`.*
>
> **Blocked:** `agileplus-cli` is not yet published from this workspace (`cargo build -p agileplus-cli` is the unlock). Once the binary exists, record via `Capture` in a Playwright or `script` session and render with `phenotype-journeys/remotion/doc-embeds/bin/render.mjs`. **Workaround:** use the web dashboard captures above for onboarding visuals until the CLI journey lands.
<!-- END-RICH-MEDIA-STUB -->

<!-- RICH-MEDIA-STUB type="recording-gif" subject="Epic/Story lifecycle — create epic, break into stories, assign" journey="epic-story-lifecycle" status="CAPTURED" -->
![Epics backlog — requirement-linked epics with per-epic story rollup](docs/assets/rich-media/agileplus/requirements-epics-panel.png)

![Stories view — filterable story list tied to epics](docs/assets/rich-media/agileplus/stories-panel.png)

*Keyframe pair from the `epic-story-lifecycle` Playwright capture (`docs/embeds/journeys/epic-story-lifecycle.annotations.json`). Remotion mp4/gif render is queued once `API_PORT=4000` is up with seeded data; re-run `render.mjs` from `phenotype-journeys/remotion/doc-embeds` to publish the animated loop.*
<!-- END-RICH-MEDIA-STUB -->

<!-- RICH-MEDIA-STUB type="recording-mp4" subject="Dashboard walkthrough — AgilePlus web dashboard all panels" journey="dashboard-walkthrough" status="PUBLISHED" -->
![AgilePlus dashboard — overview KPI cards and navigation](docs/assets/rich-media/agileplus/dashboard-overview.png)

*Three-step dashboard walkthrough captured via `@phenotype/doc-embeds` Capture helper (Overview → Epics → Stories). EmbedSpec: `docs/embeds/journeys/dashboard-walkthrough.annotations.json`. Re-capture with `API_PORT=4000 DATABASE_PATH=agileplus.db` for live epic/story counts, then `npm run render -- --annotations …` in `phenotype-journeys/remotion/doc-embeds` to drop `dashboard-walkthrough.mp4` beside these stills.*
<!-- END-RICH-MEDIA-STUB -->
