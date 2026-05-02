# AGENTS.md — AgilePlus

## Project Overview
- **Name**: AgilePlus
- **Description**: Rust workspace — work tracking CLI, dashboard, and service infrastructure
- **Location**: KooshaPari/AgilePlus
- **Language Stack**: Rust
- **Status**: Active development

## AgilePlus Mandate
All work MUST be tracked in AgilePlus:
- Reference: /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
- CLI: \`cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus && agileplus <command>\`
- No code without corresponding AgilePlus spec.

## Stack & Commands
\`\`\`bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt
\`\`\`

## Quality Checks
- \`cargo build --workspace\` — compile check
- \`cargo test --workspace\` — unit tests
- \`cargo clippy --workspace -- -D warnings\` — lint
- \`cargo fmt\` — formatting

## Git & Branch Discipline
- Feature branches: \`AgilePlus-wtrees/<topic>/\`
- Canonical: \`main\` (canonical is bare — use worktree for all authoring)
- Never commit directly to \`main\`

## References
- Parent workspace: /Users/kooshapari/CodeProjects/Phenotype/repos/CLAUDE.md
