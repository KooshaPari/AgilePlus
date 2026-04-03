# ADR-004: CLI Architecture

**Date**: 2026-04-02  
**Status**: Accepted  
**Deciders**: Agent  

## Context

AgilePlus is primarily a CLI-driven tool for developers and AI agents. The CLI is the primary interface for all operations - from specification to implementation to governance checks. The architecture must support:

- Human developers using the CLI interactively
- AI agents invoking commands programmatically
- CI/CD pipelines running commands non-interactively
- Shell completion for discoverability
- Consistent flag and argument patterns
- Extensibility for future subcommands

## Decision Drivers

- **Usability**: Commands must be intuitive for both humans and agents
- **Discoverability**: Users must find commands easily without memorization
- **Scriptability**: All commands must support non-interactive execution
- **Consistency**: Similar patterns across all subcommands
- **Extensibility**: New commands should follow established patterns
- **Performance**: Fast startup, responsive execution
- **Shell integration**: Native completion for bash, zsh, fish

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CLI Architecture                              │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                      Entry Point (agileplus)                     │ │
│  │  • Global flags: --config, --verbose, --format, --json           │ │
│  │  • Root command: version, help                                   │ │
│  └─────────────────────────────┬───────────────────────────────────┘ │
│                                │                                      │
│  ┌─────────────────────────────▼───────────────────────────────────┐ │
│  │                    Command Router (clap)                         │ │
│  │  • Subcommand matching and dispatch                              │ │
│  │  • Flag/argument parsing and validation                          │ │
│  │  • Help generation                                               │ │
│  └──────┬──────────┬──────────┬──────────┬──────────┬──────────────┘ │
│         │          │          │          │          │               │
│  ┌──────▼───┐ ┌────▼─────┐ ┌──▼───┐ ┌───▼────┐ ┌──▼─────┐        │
│  │ Planning │ │Implement │ │Verify│ │  Meta  │ │  Git   │        │
│  │ Commands │ │ Commands │ │ Cmds │ │Commands│ │Commands│        │
│  ├──────────┤ ├──────────┤ ├──────┤ ├────────┤ ├────────┤        │
│  │ specify  │ │ implement│ │check │ │status  │ │ branch │        │
│  │ research │ │ validate │ │audit │ │config  │ │commit  │        │
│  │ plan     │ │  ship    │ │verify│ │init    │ │  scan  │        │
│  │  triage  │ │retro     │ │      │ │        │ │        │        │
│  └──────────┘ └──────────┘ └──────┘ └────────┘ └────────┘        │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                    Shared Components                             │ │
│  │  • Output formatter (table, json, yaml)                          │ │
│  │  • Interactive prompts (dialoguer)                               │ │
│  │  • Progress indicators (indicatif)                               │ │
│  │  • Error reporter (miette)                                       │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                    Domain Services (gRPC)                        │ │
│  │  • CoreServiceClient                                             │ │
│  │  • AgentServiceClient                                            │ │
│  │  • IntegrationServiceClient                                      │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Command Structure

### Top-Level Commands (The "7 Command Workflow")

AgilePlus exposes exactly 7 primary commands that humans use:

| Command | Purpose | Typical Usage | Output |
|---------|---------|---------------|--------|
| `specify` | Create/refine a feature specification | Interactive discovery interview | `kitty-specs/FEAT-XXX/` |
| `research` | Technical analysis and feasibility | Pre- or post-specify research | `kitty-specs/FEAT-XXX/research.md` |
| `plan` | Generate work packages and tasks | After research complete | `kitty-specs/FEAT-XXX/plan.md`, WPs |
| `implement` | Dispatch agents to execute work | During implementation | Worktrees, PRs |
| `validate` | Check governance and quality gates | Pre-ship verification | Validation report |
| `ship` | Merge and archive feature | Final delivery | Merged branch, archive |
| `retro` | Generate learnings | Post-ship analysis | Retrospective report |

### Hidden Subcommands (Agent-Only)

Behind each primary command, ~25 hidden subcommands provide bmad-level depth:

```
specify
├── triage:classify        # Classify intent before specifying
├── triage:file-bug        # Auto-file bugs during specify
├── context:load-spec      # Load existing spec for context
└── governance:check-gates # Verify spec meets entry criteria

implement
├── git:create-worktree    # Isolate WP work
├── git:branch-from-wp     # Create feature branch
├── agent:dispatch         # Spawn subagent
├── agent:review-loop      # Handle Coderabbit feedback
├── devops:run-ci-checks   # Validate CI passes
└── git:merge-and-cleanup  # Finalize WP

validate
├── governance:verify-chain    # Check audit chain integrity
├── governance:evaluate-policy # Run quality gates
└── devops:lint-and-format     # Code quality checks
```

### Organization by Domain

```
agileplus
├── specify          # FR-001: Create specification
├── research         # FR-002: Technical analysis
├── plan             # FR-003: Work package generation
├── implement        # FR-004: Agent dispatch
├── validate         # FR-005: Quality gates
├── ship             # FR-006: Feature completion
├── retro            # FR-007: Post-hoc analysis
│
├── status           # FR-008: Project dashboard
├── config           # FR-009: Settings management
├── init             # FR-010: Project bootstrap
│
├── git              # Git integration subcommands
│   ├── scan         # Scan commits for correlations
│   ├── show         # Show git correlations
│   ├── link         # Manual commit linking
│   └── sync-notes   # Sync git notes
│
├── sync             # External sync subcommands
│   ├── push-plane   # Sync to Plane.so
│   ├── pull-plane   # Sync from Plane.so
│   ├── push-github  # Sync to GitHub
│   └── status       # Show sync state
│
├── triage           # Triage subcommands (hidden by default)
│   ├── classify
│   ├── file-bug
│   └── queue-idea
│
├── governance       # Governance subcommands (hidden)
│   ├── check-gates
│   ├── verify-chain
│   └── evaluate-policy
│
├── devops           # DevOps subcommands (hidden)
│   ├── lint-and-format
│   ├── run-ci-checks
│   └── conventional-commit
│
├── escape           # Quick-escape subcommands (hidden)
│   ├── hotfix
│   ├── quick-fix
│   └── skip-with-warning
│
└── agent            # Agent-only subcommands (hidden)
    ├── dispatch
    ├── review-loop
    └── monitor
```

## Interactive vs Batch Modes

### Mode Detection

```rust
// crates/agileplus-cli/src/mode.rs
pub enum ExecutionMode {
    /// Full interactive mode with prompts and progress bars
    Interactive,
    /// Batch mode for scripts and CI - no prompts, JSON output
    Batch,
    /// Agent mode - structured output, minimal formatting
    Agent,
}

impl ExecutionMode {
    pub fn detect() -> Self {
        if std::env::var("AGENT_MODE").is_ok() {
            ExecutionMode::Agent
        } else if !atty::is(atty::Stream::Stdin) {
            ExecutionMode::Batch
        } else {
            ExecutionMode::Interactive
        }
    }
}
```

### Interactive Mode

For human developers:

```bash
# Discovery interview with prompts
$ agileplus specify
? Feature title: User Authentication
? Priority: (Use arrow keys)
  > P1 - Critical
    P2 - High
    P3 - Medium
    P4 - Low
? Description: (Enter to open editor)

# Progress bars for long operations
$ agileplus implement FEAT-001
[████████░░░░░░░░░░░░] 40% (2/5 WPs complete)
WP-001: OAuth Core          [✓] Done
WP-002: SAML Integration  [▶] In Progress - Agent working...
WP-003: Token Refresh     [○] Queued
WP-004: Session Management[○] Queued
WP-005: Audit Logging     [○] Queued
```

### Batch Mode

For CI/CD and scripting:

```bash
# JSON output for programmatic consumption
$ agileplus status --format=json
{
  "features": [
    {
      "id": "FEAT-001",
      "title": "User Authentication",
      "status": "implementing",
      "progress": {
        "total_wps": 5,
        "completed": 2,
        "in_progress": 1,
        "blocked": 0
      }
    }
  ]
}

# Non-interactive with all flags provided
$ agileplus specify --title "Bug Fix" --priority p1 --yes
Created FEAT-042 in kitty-specs/FEAT-042-bug-fix/

# Exit codes for automation
$ agileplus validate FEAT-001; echo $?
0  # Success

$ agileplus validate FEAT-002; echo $?
1  # Validation failed - governance violations found
```

### Agent Mode

For AI agents via MCP:

```bash
# Structured output for agent parsing
$ AGENT_MODE=1 agileplus status
FEATURE|FEAT-001|User Authentication|implementing|40|2|5
WP|WP-001|OAuth Core|done|16|16
WP|WP-002|SAML Integration|doing|8|24

# Machine-readable errors
$ AGENT_MODE=1 agileplus validate FEAT-002
ERROR|GOVERNANCE|FR-003|Missing test evidence
ERROR|QUALITY|test_coverage|75%|Required: 80%
```

## Subcommand Organization

### Nested Subcommands (Multi-Level)

```
agileplus git <action>
agileplus sync <direction> <target>
agileplus triage <type>
agileplus governance <action>
agileplus devops <action>
agileplus escape <type>
```

### Command Groups

```rust
// crates/agileplus-cli/src/commands/mod.rs
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    /// Core workflow commands (the 7)
    #[command(subcommand)]
    Core(CoreCommands),
    
    /// Git integration
    #[command(subcommand)]
    Git(GitCommands),
    
    /// External sync
    #[command(subcommand)]
    Sync(SyncCommands),
    
    /// Triage and classification
    #[command(subcommand, hide = true)]
    Triage(TriageCommands),
    
    /// Governance and quality gates
    #[command(subcommand, hide = true)]
    Governance(GovernanceCommands),
    
    /// DevOps automation
    #[command(subcommand, hide = true)]
    Devops(DevopsCommands),
    
    /// Quick escapes
    #[command(subcommand, hide = true)]
    Escape(EscapeCommands),
    
    /// Agent operations
    #[command(subcommand, hide = true)]
    Agent(AgentCommands),
    
    /// Utility commands
    Status,
    Config,
    Init,
}

#[derive(Subcommand)]
pub enum CoreCommands {
    /// Create or refine a feature specification
    Specify(SpecifyArgs),
    /// Conduct technical research
    Research(ResearchArgs),
    /// Generate work packages
    Plan(PlanArgs),
    /// Execute implementation
    Implement(ImplementArgs),
    /// Validate against governance
    Validate(ValidateArgs),
    /// Ship completed feature
    Ship(ShipArgs),
    /// Generate retrospective
    Retro(RetroArgs),
}
```

## Shell Completion Strategy

### Generated Completions

```rust
// crates/agileplus-cli/src/completions.rs
use clap::Command;
use clap_complete::{generate, Generator, Shell};

pub fn generate_completions<G: Generator>(generator: G, cmd: &mut Command) {
    generate(generator, cmd, "agileplus", &mut std::io::stdout());
}

// Usage in build.rs or CLI
pub fn print_completions(shell: Shell) {
    let mut cmd = crate::cli::build_cli();
    generate_completions(shell, &mut cmd);
}
```

### Installation

```bash
# Bash (~/.bashrc)
eval "$(agileplus completions bash)"

# Zsh (~/.zshrc)
eval "$(agileplus completions zsh)"

# Fish (~/.config/fish/config.fish)
agileplus completions fish | source

# PowerShell ($PROFILE)
agileplus completions powershell | Out-String | Invoke-Expression
```

### Dynamic Completions

```rust
// Complete feature IDs
$ agileplus implement FE<TAB>
FEAT-001  FEAT-002  FEAT-003

// Complete work packages for a feature
$ agileplus implement FEAT-001 WP<TAB>
WP-001  WP-002  WP-003

// Complete git branches
$ agileplus git branch <TAB>
main  feat/FEAT-001-oauth  feat/FEAT-002-saml
```

## CLI Implementation

### Command Structure

```rust
// crates/agileplus-cli/src/commands/specify.rs
use clap::Parser;
use miette::Result;

#[derive(Parser)]
pub struct SpecifyArgs {
    /// Feature title (optional - prompts if not provided)
    #[arg(short, long)]
    title: Option<String>,
    
    /// Feature priority
    #[arg(short, long, value_enum)]
    priority: Option<Priority>,
    
    /// Skip discovery interview, use defaults
    #[arg(long)]
    quick: bool,
    
    /// Non-interactive mode
    #[arg(short, long)]
    yes: bool,
    
    /// Output format
    #[arg(short, long, value_enum, default_value = "table")]
    format: OutputFormat,
}

pub async fn run(args: SpecifyArgs, ctx: &CliContext) -> Result<()> {
    let mode = ExecutionMode::detect();
    
    // Gather inputs
    let input = match mode {
        ExecutionMode::Interactive if !args.yes => {
            // Interactive prompts
            gather_interactive_input().await?
        }
        _ => {
            // Batch/agent mode - use args or defaults
            SpecificationInput::from_args(&args)?
        }
    };
    
    // Call domain service
    let client = ctx.core_client();
    let feature = client.specify(input).await?;
    
    // Format output
    ctx.formatter().write(&feature, args.format)?;
    
    Ok(())
}
```

### Error Handling

```rust
// crates/agileplus-cli/src/error.rs
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum CliError {
    #[error("Feature not found: {id}")]
    #[diagnostic(
        code(cli::feature_not_found),
        help("Run 'agileplus status' to see available features")
    )]
    FeatureNotFound {
        id: String,
        #[label("here")]
        span: SourceSpan,
    },
    
    #[error("Governance violation: {message}")]
    #[diagnostic(
        code(cli::governance_violation),
        help("Run 'agileplus governance check-gates' for details")
    )]
    GovernanceViolation {
        message: String,
        feature_id: String,
        gate: String,
    },
    
    #[error("Connection failed: {0}")]
    #[diagnostic(
        code(cli::connection_error),
        help("Is the AgilePlus service running? Try 'agileplus status'")
    )]
    ConnectionError(#[from] tonic::transport::Error),
}
```

### Output Formatting

```rust
// crates/agileplus-cli/src/format.rs
use serde::Serialize;

pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Csv,
    Agent,  // Machine-readable for agents
}

pub trait Formatter {
    fn write<T: Serialize>(&self, data: &T, format: OutputFormat) -> Result<()>;
}

impl Formatter for ConsoleFormatter {
    fn write<T: Serialize>(&self, data: &T, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Table => {
                let table = to_table(data)?;
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(data)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(data)?);
            }
            OutputFormat::Agent => {
                // Pipe-delimited for easy parsing
                println!("{}", to_agent_format(data)?);
            }
            OutputFormat::Csv => {
                println!("{}", to_csv(data)?);
            }
        }
        Ok(())
    }
}
```

## Configuration Integration

### Config Hierarchy

```
1. CLI flags (highest priority)
2. Environment variables (AGILEPLUS_*)
3. Project config (.agileplus/config.toml)
4. User config (~/.config/agileplus/config.toml)
5. System defaults (lowest priority)
```

### Config Commands

```bash
# View current configuration
$ agileplus config
project.name = "MyProject"
sync.plane.enabled = true
sync.plane.url = "https://plane.example.com"
git.auto_scan = true

# Get specific value
$ agileplus config get sync.plane.url
https://plane.example.com

# Set value
$ agileplus config set sync.plane.enabled false

# Edit configuration
$ agileplus config edit
# Opens $EDITOR with config.toml
```

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| CLI startup | < 100ms | Time from command start to first output |
| Help generation | < 50ms | `agileplus --help` |
| Command dispatch | < 50ms | Parsing and routing |
| Tab completion | < 100ms | First suggestion displayed |
| JSON output | < 20ms | Serialization overhead |
| Progress refresh | 10Hz | Updates per second for long ops |

## References

- clap.rs: https://docs.rs/clap/latest/clap/
- clap_complete: https://docs.rs/clap_complete/latest/clap_complete/
- miette (diagnostics): https://docs.rs/miette/latest/miette/
- dialoguer (prompts): https://docs.rs/dialoguer/latest/dialoguer/
- indicatif (progress): https://docs.rs/indicatif/latest/indicatif/

---

*This ADR will be updated as CLI patterns evolve*
