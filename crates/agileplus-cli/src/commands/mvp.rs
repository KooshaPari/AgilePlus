//! MVP project/epic/story/work-package command surface.

use std::str::FromStr;

use anyhow::{Context, Result, anyhow, bail};
use chrono::NaiveDate;
use clap::{Args, Subcommand, ValueEnum};

// ── Top-level Mvp command surface ────────────────────────────────────────────

/// MVP project/epic/story/work-package management.
#[derive(Debug, Args)]
pub struct MvpArgs {
    #[command(subcommand)]
    pub cmd: MvpCmd,
}

#[derive(Debug, Subcommand)]
pub enum MvpCmd {
    /// Create a project.
    /// Manage projects (CRUD).
    #[command(subcommand)]
    Project(ProjectCmd),

    /// Manage epics (CRUD).
    #[command(subcommand)]
    Epic(EpicCmd),

    /// Manage stories (CRUD).
    #[command(subcommand)]
    Story(StoryCmd),

    /// Manage work packages (CRUD).
    #[command(subcommand)]
    Wp(WpCmd),

    /// Manage dependencies.
    #[command(subcommand)]
    Dep(DepCmd),
    CycleCreate(CycleCreateArgs),
    /// Add an epic or story to a cycle.
    CycleAdd(CycleAddArgs),
    /// Transition the state of a work package or story.
    Transition(TransitionArgs),
    /// List the next-ready work packages.
    NextReady(NextReadyArgs),
}

/// Top-level entry point for `ap mvp`.
pub async fn run_mvp<S: StoragePort>(args: MvpArgs, storage: &S) -> Result<()> {
    match args.cmd {
        MvpCmd::Project(ProjectCmd::Create(a)) => project_create(&a, storage).await,
        MvpCmd::Epic(EpicCmd::Create(a)) => epic_create(&a, storage).await,
        MvpCmd::Story(StoryCmd::Create(a)) => story_create(&a, storage).await,
        MvpCmd::Wp(WpCmd::Create(a)) => wp_create(&a, storage).await,
        MvpCmd::Dep(DepCmd::Add(a)) => dep_add(&a, storage).await,
        MvpCmd::CycleCreate(a) => cycle_create(&a, storage).await,
        MvpCmd::CycleAdd(a) => cycle_add(&a, storage).await,
        MvpCmd::Transition(a) => transition(&a, storage).await,
        MvpCmd::NextReady(a) => next_ready(&a, storage).await,
    }
}

use agileplus_domain::{
    domain::{
        cycle::{Cycle, CycleFeature},
        epic::Epic,
        project::Project,
        story::{Story, StoryStatus},
        work_package::{DependencyType, WorkPackage, WpDependency, WpState},
    },
    ports::StoragePort,
};

#[derive(Debug, Subcommand)]
pub enum ProjectCmd {
    /// Create a project.
    Create(ProjectCreateArgs),
}

#[derive(Debug, Args)]
pub struct ProjectCreateArgs {
    #[arg(long)]
    pub slug: String,

    #[arg(long)]
    pub name: String,

    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum EpicCmd {
    /// Create an epic.
    Create(EpicCreateArgs),
}

#[derive(Debug, Args)]
pub struct EpicCreateArgs {
    #[arg(long)]
    pub project: i64,

    #[arg(long)]
    pub title: String,

    #[arg(long)]
    pub description: String,

    #[arg(long)]
    pub requirement: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum StoryCmd {
    /// Create a story.
    Create(StoryCreateArgs),
}

#[derive(Debug, Args)]
pub struct StoryCreateArgs {
    #[arg(long)]
    pub epic: i64,

    #[arg(long)]
    pub title: String,

    #[arg(long)]
    pub description: String,

    #[arg(long)]
    pub points: Option<u32>,

    #[arg(long)]
    pub requirement: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum WpCmd {
    /// Create a work package for a story.
    Create(WpCreateArgs),
}

#[derive(Debug, Args)]
pub struct WpCreateArgs {
    #[arg(long)]
    pub story: i64,

    #[arg(long)]
    pub title: String,

    #[arg(long = "file-scope")]
    pub file_scope: String,

    #[arg(long)]
    pub acceptance: String,

    #[arg(long)]
    pub seq: Option<i32>,
}

#[derive(Debug, Subcommand)]
pub enum DepCmd {
    /// Add a work-package dependency.
    Add(DepAddArgs),
}

#[derive(Debug, Args)]
pub struct DepAddArgs {
    #[arg(long)]
    pub wp: i64,

    #[arg(long = "depends-on")]
    pub depends_on: i64,

    #[arg(long = "type", value_enum)]
    pub dep_type: DepTypeArg,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DepTypeArg {
    Explicit,
    FileOverlap,
    Data,
}

#[derive(Debug, Args)]
pub struct CycleCreateArgs {
    #[arg(long)]
    pub name: String,

    #[arg(long)]
    pub start: String,

    #[arg(long)]
    pub end: String,
}

#[derive(Debug, Args)]
pub struct CycleAddArgs {
    #[arg(long)]
    pub cycle: i64,

    #[arg(long, conflicts_with = "story")]
    pub epic: Option<i64>,

    #[arg(long, conflicts_with = "epic")]
    pub story: Option<i64>,
}

#[derive(Debug, Args)]
pub struct TransitionArgs {
    #[arg(long, conflicts_with = "story")]
    pub wp: Option<i64>,

    #[arg(long, conflicts_with = "wp")]
    pub story: Option<i64>,

    #[arg(long)]
    pub to: String,
}

#[derive(Debug, Args)]
pub struct NextReadyArgs {
    #[arg(long)]
    pub cycle: Option<i64>,

    #[arg(long)]
    pub json: bool,
}

pub async fn project_create<S: StoragePort>(args: &ProjectCreateArgs, storage: &S) -> Result<()> {
    let mut project = Project::new(&args.name, &args.slug)?;
    project.description = args.description.clone();
    let id = storage
        .create_project(&project)
        .await
        .context("creating project")?;
    println!("project_id: {id}");
    Ok(())
}

pub async fn epic_create<S: StoragePort>(args: &EpicCreateArgs, storage: &S) -> Result<()> {
    let mut epic = Epic::new(args.project, &args.title)?;
    epic.description = Some(args.description.clone());
    epic.requirement_id = args.requirement.clone();
    let id = storage.create_epic(&epic).await.context("creating epic")?;
    println!("epic_id: {id}");
    Ok(())
}

pub async fn story_create<S: StoragePort>(args: &StoryCreateArgs, storage: &S) -> Result<()> {
    let epic = storage
        .get_epic(args.epic)
        .await
        .context("loading epic")?
        .ok_or_else(|| anyhow!("epic {} not found", args.epic))?;
    let mut story = Story::new(args.epic, epic.project_id, &args.title, args.points)?;
    story.description = Some(args.description.clone());
    story.requirement_id = args.requirement.clone();
    let id = storage
        .create_story(&story)
        .await
        .context("creating story")?;
    println!("story_id: {id}");
    Ok(())
}

pub async fn wp_create<S: StoragePort>(args: &WpCreateArgs, storage: &S) -> Result<()> {
    let seq = args.seq.unwrap_or(1);
    let mut wp = WorkPackage::new(0, &args.title, seq, &args.acceptance);
    wp.file_scope = parse_csv(&args.file_scope);
    let id = storage
        .create_work_package_for_story(args.story, &wp)
        .await
        .context("creating work package")?;
    println!("wp_id: {id}");
    Ok(())
}

pub async fn dep_add<S: StoragePort>(args: &DepAddArgs, storage: &S) -> Result<()> {
    let dep = WpDependency {
        wp_id: args.wp,
        depends_on: args.depends_on,
        dep_type: args.dep_type.into(),
    };
    storage
        .add_wp_dependency(&dep)
        .await
        .context("adding work package dependency")?;
    println!("dependency: {} -> {}", args.wp, args.depends_on);
    Ok(())
}

pub async fn cycle_create<S: StoragePort>(args: &CycleCreateArgs, storage: &S) -> Result<()> {
    let start = parse_date(&args.start)?;
    let end = parse_date(&args.end)?;
    let cycle = Cycle::new(&args.name, start, end, None).map_err(anyhow::Error::msg)?;
    let id = storage
        .create_cycle(&cycle)
        .await
        .context("creating cycle")?;
    println!("cycle_id: {id}");
    Ok(())
}

pub async fn cycle_add<S: StoragePort>(args: &CycleAddArgs, storage: &S) -> Result<()> {
    match (args.epic, args.story) {
        (Some(epic_id), None) => {
            let stories = storage
                .list_stories_by_epic(epic_id)
                .await
                .context("listing epic stories")?;
            if stories.is_empty() {
                bail!("epic {epic_id} has no stories to add");
            }
            for story in stories {
                storage
                    .add_story_to_cycle(args.cycle, story.id)
                    .await
                    .with_context(|| format!("adding story {} to cycle", story.id))?;
            }
            println!("cycle_story_count: added epic {epic_id}");
        }
        (None, Some(story_id)) => {
            storage
                .add_story_to_cycle(args.cycle, story_id)
                .await
                .context("adding story to cycle")?;
            println!("cycle_story: {} -> {}", args.cycle, story_id);
        }
        _ => bail!("provide exactly one of --epic or --story"),
    }
    Ok(())
}

pub async fn cycle_add_feature<S: StoragePort>(
    cycle_id: i64,
    feature_id: i64,
    storage: &S,
) -> Result<()> {
    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .context("adding feature to cycle")?;
    println!("cycle_feature: {cycle_id} -> {feature_id}");
    Ok(())
}

pub async fn transition<S: StoragePort>(args: &TransitionArgs, storage: &S) -> Result<()> {
    match (args.wp, args.story) {
        (Some(wp_id), None) => {
            let target = parse_wp_state(&args.to)?;
            let wp = storage
                .get_work_package(wp_id)
                .await
                .context("loading work package")?
                .ok_or_else(|| anyhow!("work package {wp_id} not found"))?;
            if !wp.state.can_transition_to(target) {
                bail!("illegal wp transition: {:?} -> {:?}", wp.state, target);
            }
            if target == WpState::Doing {
                for dependency in storage
                    .get_wp_dependencies(wp_id)
                    .await
                    .context("loading work-package dependencies")?
                {
                    let prerequisite = storage
                        .get_work_package(dependency.depends_on)
                        .await
                        .context("loading prerequisite work package")?
                        .ok_or_else(|| {
                            anyhow!("prerequisite WP {} not found", dependency.depends_on)
                        })?;
                    if prerequisite.state != WpState::Done {
                        bail!(
                            "WP{:02} is blocked by WP{:02} ({:?})",
                            wp.sequence,
                            prerequisite.sequence,
                            prerequisite.state
                        );
                    }
                }
            }
            if let Some(contract) = storage
                .get_latest_governance_contract(wp.feature_id)
                .await
                .context("loading governance contract for work-package transition")?
            {
                let transition = format!("WP{:02}: {:?} -> {:?}", wp.sequence, wp.state, target);
                let items = storage
                    .get_evidence_by_wp(wp.id)
                    .await
                    .context("loading work-package evidence")?;
                for rule in contract
                    .rules
                    .iter()
                    .filter(|rule| rule.transition == transition)
                {
                    for requirement in &rule.required_evidence {
                        if !items.iter().any(|item| {
                            crate::commands::evidence::matches_requirement(requirement, item)
                        }) {
                            bail!(
                                "{} requires valid {} evidence before the state changes",
                                transition,
                                requirement
                            );
                        }
                    }
                }
            }
            storage
                .update_wp_state(wp_id, target)
                .await
                .context("updating work package state")?;
            let prev_hash = storage
                .get_latest_audit_entry(wp.feature_id)
                .await
                .context("loading prior work-package audit")?
                .map_or([0u8; 32], |entry| entry.hash);
            let mut audit = agileplus_domain::domain::audit::AuditEntry {
                id: 0,
                feature_id: wp.feature_id,
                wp_id: Some(wp.id),
                timestamp: chrono::Utc::now(),
                actor: "user".into(),
                transition: format!("WP{:02}: {:?} -> {:?}", wp.sequence, wp.state, target),
                evidence_refs: vec![],
                prev_hash,
                hash: [0u8; 32],
                event_id: None,
                archived_to: None,
            };
            audit.hash = agileplus_domain::domain::audit::hash_entry(&audit);
            storage
                .append_audit_entry(&audit)
                .await
                .context("recording work-package transition")?;
            println!("wp_state: {wp_id} -> {}", wp_state_label(target));
        }
        (None, Some(story_id)) => {
            let target = StoryStatus::from_str(&args.to)?;
            let mut story = storage
                .get_story(story_id)
                .await
                .context("loading story")?
                .ok_or_else(|| anyhow!("story {story_id} not found"))?;
            story.transition_status(target)?;
            storage
                .update_story_status(story_id, target)
                .await
                .context("updating story status")?;
            println!("story_status: {story_id} -> {target}");
        }
        _ => bail!("provide exactly one of --wp or --story"),
    }
    Ok(())
}

pub async fn next_ready<S: StoragePort>(args: &NextReadyArgs, storage: &S) -> Result<()> {
    let wps = storage
        .get_next_ready_wps(args.cycle)
        .await
        .context("listing next-ready work packages")?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&wps)?);
        return Ok(());
    }

    if wps.is_empty() {
        println!("No next-ready work packages.");
        return Ok(());
    }

    println!("{:<6}  {:<8}  {:<8}  TITLE", "ID", "FEATURE", "STATE");
    println!("{}", "-".repeat(70));
    for wp in &wps {
        println!(
            "{:<6}  {:<8}  {:<8}  {}",
            wp.id,
            wp.feature_id,
            wp_state_label(wp.state),
            wp.title
        );
    }
    Ok(())
}

fn parse_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_date(raw: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .with_context(|| format!("invalid date '{raw}', expected YYYY-MM-DD"))
}

fn parse_wp_state(raw: &str) -> Result<WpState> {
    match raw {
        "planned" => Ok(WpState::Planned),
        "doing" => Ok(WpState::Doing),
        "review" => Ok(WpState::Review),
        "done" => Ok(WpState::Done),
        "blocked" => Ok(WpState::Blocked),
        _ => bail!("unknown WpState: {raw}"),
    }
}

fn wp_state_label(state: WpState) -> &'static str {
    match state {
        WpState::Planned => "planned",
        WpState::Doing => "doing",
        WpState::Review => "review",
        WpState::Done => "done",
        WpState::Blocked => "blocked",
    }
}

impl From<DepTypeArg> for DependencyType {
    fn from(value: DepTypeArg) -> Self {
        match value {
            DepTypeArg::Explicit => DependencyType::Explicit,
            DepTypeArg::FileOverlap => DependencyType::FileOverlap,
            DepTypeArg::Data => DependencyType::Data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn wp_transition_requires_its_contract_evidence_before_mutation() {
        use agileplus_domain::domain::feature::Feature;
        use agileplus_domain::domain::governance::{
            Evidence, EvidenceType, GovernanceContract, GovernanceRule,
        };
        use agileplus_sqlite::SqliteStorageAdapter;
        use chrono::Utc;

        let db = SqliteStorageAdapter::in_memory().unwrap();
        let feature_id = db
            .create_feature(&Feature::new("gate", "Gate", [0; 32], None))
            .await
            .unwrap();
        let wp_id = db
            .create_work_package(&WorkPackage::new(
                feature_id,
                "First",
                1,
                "- FR-001 -- task",
            ))
            .await
            .unwrap();
        db.update_wp_state(wp_id, WpState::Doing).await.unwrap();
        db.create_governance_contract(&GovernanceContract {
            id: 0,
            feature_id,
            version: 1,
            bound_at: Utc::now(),
            rules: vec![GovernanceRule {
                transition: "WP01: Doing -> Review".into(),
                required_evidence: vec!["FR-CI:ci_output".into(), "FR-001:test_result".into()],
                policy_refs: vec![],
            }],
        })
        .await
        .unwrap();
        let args = TransitionArgs {
            wp: Some(wp_id),
            story: None,
            to: "review".into(),
        };
        assert!(
            transition(&args, &db)
                .await
                .unwrap_err()
                .to_string()
                .contains("FR-CI")
        );
        assert_eq!(
            db.get_work_package(wp_id).await.unwrap().unwrap().state,
            WpState::Doing
        );
        let artifact = tempfile::NamedTempFile::new().unwrap();
        let digest = crate::commands::evidence::digest_file(artifact.path()).unwrap();
        db.create_evidence(&Evidence {
            id: 0,
            wp_id,
            fr_id: "FR-CI".into(),
            evidence_type: EvidenceType::CiOutput,
            artifact_path: artifact.path().display().to_string(),
            metadata: Some(serde_json::json!({"result":"pass", "sha256":digest})),
            created_at: Utc::now(),
        })
        .await
        .unwrap();
        let receipts = tempfile::tempdir().unwrap();
        crate::commands::evidence::run_evidence(
            crate::commands::evidence::EvidenceArgs {
                command: crate::commands::evidence::EvidenceCommand::RunTest(
                    crate::commands::evidence::RunTestArgs {
                        feature: "gate".into(),
                        wp: "WP01".into(),
                        fr: "FR-001".into(),
                        artifact: receipts.path().join("fr001.json"),
                        command: vec!["/bin/true".into()],
                    },
                ),
            },
            &db,
        )
        .await
        .unwrap();
        transition(&args, &db).await.unwrap();
        assert_eq!(
            db.get_work_package(wp_id).await.unwrap().unwrap().state,
            WpState::Review
        );
        assert_eq!(
            db.get_latest_audit_entry(feature_id)
                .await
                .unwrap()
                .unwrap()
                .transition,
            "WP01: Doing -> Review"
        );
    }

    // ── parse_csv ────────────────────────────────────────────────────────────

    #[test]
    fn parse_csv_single_element() {
        assert_eq!(parse_csv("src/main.rs"), vec!["src/main.rs"]);
    }

    #[test]
    fn parse_csv_multiple_elements() {
        assert_eq!(parse_csv("a.rs,b.rs,c.rs"), vec!["a.rs", "b.rs", "c.rs"]);
    }

    #[test]
    fn parse_csv_trims_whitespace() {
        assert_eq!(
            parse_csv(" a.rs , b.rs , c.rs "),
            vec!["a.rs", "b.rs", "c.rs"]
        );
    }

    #[test]
    fn parse_csv_filters_empty_segments() {
        assert_eq!(parse_csv("a.rs,,b.rs,,"), vec!["a.rs", "b.rs"]);
    }

    #[test]
    fn parse_csv_empty_string() {
        assert!(parse_csv("").is_empty());
    }

    #[test]
    fn parse_csv_commas_only() {
        assert!(parse_csv(",,,,").is_empty());
    }

    // ── parse_date ───────────────────────────────────────────────────────────

    #[test]
    fn parse_date_valid() {
        let d = parse_date("2025-01-15").unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    }

    #[test]
    fn parse_date_invalid_format() {
        assert!(parse_date("01/15/2025").is_err());
    }

    #[test]
    fn parse_date_invalid_day() {
        assert!(parse_date("2025-02-30").is_err());
    }

    #[test]
    fn parse_date_garbage() {
        assert!(parse_date("not-a-date").is_err());
    }

    #[test]
    fn parse_date_empty() {
        assert!(parse_date("").is_err());
    }

    // ── parse_wp_state ───────────────────────────────────────────────────────

    #[test]
    fn parse_wp_state_all_variants() {
        assert!(matches!(
            parse_wp_state("planned").unwrap(),
            WpState::Planned
        ));
        assert!(matches!(parse_wp_state("doing").unwrap(), WpState::Doing));
        assert!(matches!(parse_wp_state("review").unwrap(), WpState::Review));
        assert!(matches!(parse_wp_state("done").unwrap(), WpState::Done));
        assert!(matches!(
            parse_wp_state("blocked").unwrap(),
            WpState::Blocked
        ));
    }

    #[test]
    fn parse_wp_state_unknown() {
        assert!(parse_wp_state("unknown").is_err());
    }

    #[test]
    fn parse_wp_state_case_sensitive() {
        assert!(parse_wp_state("Planned").is_err());
        assert!(parse_wp_state("DOING").is_err());
    }

    #[test]
    fn parse_wp_state_empty_string() {
        assert!(parse_wp_state("").is_err());
    }

    // ── wp_state_label additional ──────────────────────────────────────────

    #[test]
    fn wp_state_label_all_unique() {
        let labels: Vec<&str> = vec![
            wp_state_label(WpState::Planned),
            wp_state_label(WpState::Doing),
            wp_state_label(WpState::Review),
            wp_state_label(WpState::Done),
            wp_state_label(WpState::Blocked),
        ];
        let unique: std::collections::HashSet<&str> = labels.into_iter().collect();
        assert_eq!(unique.len(), 5, "all state labels should be unique");
    }

    // ── wp_state_label ───────────────────────────────────────────────────────

    #[test]
    fn wp_state_label_roundtrip() {
        assert_eq!(wp_state_label(WpState::Planned), "planned");
        assert_eq!(wp_state_label(WpState::Doing), "doing");
        assert_eq!(wp_state_label(WpState::Review), "review");
        assert_eq!(wp_state_label(WpState::Done), "done");
        assert_eq!(wp_state_label(WpState::Blocked), "blocked");
    }

    // ── DepTypeArg → DependencyType conversion ───────────────────────────────

    #[test]
    fn dep_type_arg_explicit_conversion() {
        let dt: DependencyType = DepTypeArg::Explicit.into();
        assert!(matches!(dt, DependencyType::Explicit));
    }

    #[test]
    fn dep_type_arg_file_overlap_conversion() {
        let dt: DependencyType = DepTypeArg::FileOverlap.into();
        assert!(matches!(dt, DependencyType::FileOverlap));
    }

    #[test]
    fn dep_type_arg_data_conversion() {
        let dt: DependencyType = DepTypeArg::Data.into();
        assert!(matches!(dt, DependencyType::Data));
    }
}
