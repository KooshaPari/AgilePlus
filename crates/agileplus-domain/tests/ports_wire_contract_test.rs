//! Wire-contract tests for the adapter-facing port payload types.
//!
//! `ports::{agent, observability, review, vcs, events}` define the data that
//! crosses the adapter boundary: agent dispatch/status, telemetry spans,
//! metrics and logs, code-review and CI state, git worktree/merge results, and
//! the post-persistence domain events.
//!
//! Before this file none of those 26 types had a single test reference in this
//! crate, so the `Serialize`/`Deserialize` impls, the `#[serde(rename_all)]`
//! wire names, the external tagging of data-carrying variants, and the
//! optional-field handling were entirely unexercised. Adapters (`agileplus-git`,
//! `agileplus-telemetry`, `agileplus-api`, `agileplus-grpc`) and the dashboard
//! JSON API read this format, so a silent rename or a dropped payload field is
//! a cross-crate breaking change rather than a local refactor.
//!
//! Traceability: FR-004, FR-010, FR-011, FR-012, FR-013, FR-014, FR-OBSERVE-*
//! / WP05-T026, WP05-T027, WP05-T028, WP05-T029

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::agent::{AgentConfig, AgentKind, AgentResult, AgentStatus, AgentTask};
use agileplus_domain::ports::events::{DomainEvent, DomainEventPublisher};
use agileplus_domain::ports::observability::{
    LogEntry, LogLevel, MetricValue, ObservabilityPort, SpanContext,
};
use agileplus_domain::ports::review::{
    CiStatus, CommentSeverity, PrInfo, ReviewComment, ReviewStatus,
};
use agileplus_domain::ports::vcs::{
    BranchInfo, ConflictInfo, FeatureArtifacts, MergeResult, WorktreeInfo,
};

fn to_json<T: serde::Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).expect("port payload must serialize")
}

fn agent_result(success: bool, pr_url: Option<&str>) -> AgentResult {
    AgentResult {
        success,
        pr_url: pr_url.map(str::to_string),
        commits: vec!["abc1234".to_string()],
        stdout: "done".to_string(),
        stderr: String::new(),
        exit_code: if success { 0 } else { 1 },
    }
}

fn review_comment() -> ReviewComment {
    ReviewComment {
        author: "coderabbit".to_string(),
        body: "nil check missing".to_string(),
        file_path: Some("src/main.rs".to_string()),
        line: Some(42),
        severity: CommentSeverity::Major,
        actionable: true,
    }
}

// ---------------------------------------------------------------- agent port

#[test]
fn agent_kind_uses_snake_case_wire_names() {
    assert_eq!(
        to_json(&AgentKind::ClaudeCode),
        serde_json::json!("claude_code")
    );
    assert_eq!(to_json(&AgentKind::Codex), serde_json::json!("codex"));
    assert_eq!(
        serde_json::from_value::<AgentKind>(serde_json::json!("claude_code")).unwrap(),
        AgentKind::ClaudeCode
    );

    // The wire spelling is a contract: the Rust variant name and kebab case are
    // rejected rather than silently coerced.
    for wrong in ["ClaudeCode", "claudecode", "claude-code", "CODEX"] {
        assert!(
            serde_json::from_value::<AgentKind>(serde_json::json!(wrong)).is_err(),
            "{wrong} must not deserialize into an AgentKind"
        );
    }
}

#[test]
fn agent_config_round_trips_every_limit_and_arg() {
    let config = AgentConfig {
        kind: AgentKind::Codex,
        max_review_cycles: 3,
        timeout_secs: 1800,
        extra_args: vec!["--model".to_string(), "gpt-5".to_string()],
    };

    let value = to_json(&config);
    assert_eq!(value["kind"], "codex");
    assert_eq!(value["max_review_cycles"], 3);
    assert_eq!(value["timeout_secs"], 1800);
    assert_eq!(value["extra_args"], serde_json::json!(["--model", "gpt-5"]));

    let back: AgentConfig = serde_json::from_value(value).unwrap();
    assert_eq!(back.kind, AgentKind::Codex);
    assert_eq!(back.max_review_cycles, 3);
    assert_eq!(back.timeout_secs, 1800);
    assert_eq!(back.extra_args, config.extra_args);

    // A dispatch with no review loop and no extra args is a valid config.
    let bare: AgentConfig = serde_json::from_value(serde_json::json!({
        "kind": "claude_code",
        "max_review_cycles": 0,
        "timeout_secs": 0,
        "extra_args": []
    }))
    .unwrap();
    assert_eq!(bare.kind, AgentKind::ClaudeCode);
    assert_eq!(bare.max_review_cycles, 0);
    assert!(bare.extra_args.is_empty());
}

#[test]
fn agent_task_round_trips_paths_and_context_files() {
    let task = AgentTask {
        wp_id: "WP05-T026".to_string(),
        feature_slug: "auth-overhaul".to_string(),
        prompt_path: PathBuf::from("/repo/prompts/wp05.md"),
        worktree_path: PathBuf::from("/repo/AgilePlus-wtrees/auth"),
        context_files: vec![
            PathBuf::from("/repo/specs/auth.md"),
            PathBuf::from("/repo/plan.md"),
        ],
    };

    let value = to_json(&task);
    assert_eq!(value["wp_id"], "WP05-T026");
    assert_eq!(value["feature_slug"], "auth-overhaul");
    assert_eq!(value["prompt_path"], "/repo/prompts/wp05.md");
    assert_eq!(value["worktree_path"], "/repo/AgilePlus-wtrees/auth");
    assert_eq!(
        value["context_files"],
        serde_json::json!(["/repo/specs/auth.md", "/repo/plan.md"])
    );

    let back: AgentTask = serde_json::from_value(value).unwrap();
    assert_eq!(back.wp_id, task.wp_id);
    assert_eq!(back.feature_slug, task.feature_slug);
    assert_eq!(back.prompt_path, task.prompt_path);
    assert_eq!(back.worktree_path, task.worktree_path);
    assert_eq!(back.context_files, task.context_files);
}

#[test]
fn agent_result_distinguishes_no_pr_from_a_reported_pr() {
    let with_pr = agent_result(true, Some("https://github.com/o/r/pull/1"));
    let value = to_json(&with_pr);
    assert_eq!(value["success"], true);
    assert_eq!(value["pr_url"], "https://github.com/o/r/pull/1");
    assert_eq!(value["commits"], serde_json::json!(["abc1234"]));
    assert_eq!(value["stderr"], "");
    assert_eq!(value["exit_code"], 0);

    // `pr_url` is an explicit null, not an omitted key: consumers can tell the
    // agent ran and produced no PR.
    let no_pr = AgentResult {
        pr_url: None,
        ..with_pr.clone()
    };
    let value = to_json(&no_pr);
    assert!(value.get("pr_url").is_some(), "pr_url key must be present");
    assert_eq!(value["pr_url"], serde_json::Value::Null);

    let failed = AgentResult {
        success: false,
        exit_code: 1,
        stderr: "boom".to_string(),
        ..with_pr.clone()
    };
    let back: AgentResult = serde_json::from_value(to_json(&failed)).unwrap();
    assert!(!back.success);
    assert_eq!(back.exit_code, 1);
    assert_eq!(back.stderr, "boom");
    assert_eq!(back.stdout, with_pr.stdout);
    // A failure that still produced a PR URL keeps it.
    assert_eq!(
        back.pr_url.as_deref(),
        Some("https://github.com/o/r/pull/1")
    );

    // A failure that never opened a PR round trips as an explicit null.
    let failed_without_pr = AgentResult {
        pr_url: None,
        ..failed.clone()
    };
    let back: AgentResult = serde_json::from_value(to_json(&failed_without_pr)).unwrap();
    assert!(!back.success);
    assert!(back.pr_url.is_none());
}

#[test]
fn agent_status_tags_every_variant_externally_with_its_payload() {
    assert_eq!(to_json(&AgentStatus::Pending), serde_json::json!("pending"));
    assert_eq!(
        to_json(&AgentStatus::Running { pid: 4242 }),
        serde_json::json!({"running": {"pid": 4242}})
    );
    assert_eq!(
        to_json(&AgentStatus::WaitingForReview {
            pr_url: "https://github.com/o/r/pull/7".to_string()
        }),
        serde_json::json!({"waiting_for_review": {"pr_url": "https://github.com/o/r/pull/7"}})
    );
    assert_eq!(
        to_json(&AgentStatus::Failed {
            error: "exit 1".to_string()
        }),
        serde_json::json!({"failed": {"error": "exit 1"}})
    );

    let completed = AgentStatus::Completed {
        result: agent_result(true, None),
    };
    let value = to_json(&completed);
    assert_eq!(value["completed"]["result"]["success"], true);
    assert_eq!(
        value["completed"]["result"]["pr_url"],
        serde_json::Value::Null
    );

    let all = [
        AgentStatus::Pending,
        AgentStatus::Running { pid: 7 },
        AgentStatus::WaitingForReview {
            pr_url: "u".to_string(),
        },
        AgentStatus::Failed {
            error: "e".to_string(),
        },
        completed,
    ];
    for status in all {
        let encoded = serde_json::to_string(&status).unwrap();
        let back: AgentStatus = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            to_json(&back),
            to_json(&status),
            "round trip drifted for {encoded}"
        );
    }
}

#[test]
fn agent_status_rejects_unknown_and_malformed_variants() {
    for bad in [
        // Rust variant spelling / wrong case is not the wire format.
        r#""Pending""#,
        r#""unknown""#,
        // Payload-bearing variants require their exact field set.
        r#"{"running":{}}"#,
        r#"{"running":4242}"#,
        r#"{"waiting_for_review":{"pr_url":9}}"#,
        r#"{"failed":{}}"#,
        r#"{"completed":{}}"#,
        // `pid` is a u32.
        r#"{"running":{"pid":-1}}"#,
    ] {
        assert!(
            serde_json::from_str::<AgentStatus>(bad).is_err(),
            "{bad} must not deserialize into an AgentStatus"
        );
    }
}

// -------------------------------------------------------- observability port

#[test]
fn span_context_round_trips_with_and_without_a_parent() {
    let root = SpanContext {
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        parent_span_id: None,
    };
    let value = to_json(&root);
    assert_eq!(
        value,
        serde_json::json!({"trace_id": "trace-1", "span_id": "span-1", "parent_span_id": null})
    );
    let back: SpanContext = serde_json::from_value(value).unwrap();
    assert_eq!(back.trace_id, "trace-1");
    assert_eq!(back.span_id, "span-1");
    assert!(back.parent_span_id.is_none());

    let child = SpanContext {
        parent_span_id: Some("span-1".to_string()),
        ..root
    };
    let value = to_json(&child);
    assert_eq!(value["parent_span_id"], "span-1");
    let back: SpanContext = serde_json::from_value(value).unwrap();
    assert_eq!(back.parent_span_id.as_deref(), Some("span-1"));
}

#[test]
fn metric_value_tags_distinguish_counter_histogram_and_gauge() {
    assert_eq!(
        to_json(&MetricValue::Counter(7)),
        serde_json::json!({"counter": 7})
    );
    assert_eq!(
        to_json(&MetricValue::Histogram(1.5)),
        serde_json::json!({"histogram": 1.5})
    );
    assert_eq!(
        to_json(&MetricValue::Gauge(-0.25)),
        serde_json::json!({"gauge": -0.25})
    );

    // The tag carries the metric family, so the three must stay distinguishable
    // after a round trip (a shared tag would collapse counter/gauge/etc).
    for case in [
        MetricValue::Counter(0),
        MetricValue::Histogram(99.5),
        MetricValue::Gauge(-3.0),
    ] {
        let back: MetricValue = serde_json::from_value(to_json(&case)).unwrap();
        assert_eq!(to_json(&back), to_json(&case));
    }
    assert!(serde_json::from_str::<MetricValue>(r#"{"counter":-1}"#).is_err());
    assert!(serde_json::from_str::<MetricValue>(r#"{"counter":1.5}"#).is_err());
    assert!(serde_json::from_str::<MetricValue>(r#"{"histogram":"fast"}"#).is_err());
}

#[test]
fn log_level_wire_names_are_snake_case_and_strict() {
    for (level, wire) in [
        (LogLevel::Trace, "trace"),
        (LogLevel::Debug, "debug"),
        (LogLevel::Info, "info"),
        (LogLevel::Warn, "warn"),
        (LogLevel::Error, "error"),
    ] {
        assert_eq!(to_json(&level), serde_json::json!(wire));
        assert_eq!(
            serde_json::from_value::<LogLevel>(serde_json::json!(wire)).unwrap(),
            level
        );
    }

    for bad in ["TRACE", "Info", "warning", "err", ""] {
        assert!(
            serde_json::from_value::<LogLevel>(serde_json::json!(bad)).is_err(),
            "{bad} must not deserialize into a LogLevel"
        );
    }
}

#[test]
fn log_entry_round_trips_fields_and_optional_span_context() {
    let mut fields = HashMap::new();
    fields.insert("wp_id".to_string(), "WP05-T029".to_string());
    let entry = LogEntry {
        level: LogLevel::Warn,
        message: "retrying".to_string(),
        fields,
        span_context: Some(SpanContext {
            trace_id: "t".to_string(),
            span_id: "s".to_string(),
            parent_span_id: None,
        }),
    };

    let value = to_json(&entry);
    assert_eq!(value["level"], "warn");
    assert_eq!(value["message"], "retrying");
    assert_eq!(value["fields"], serde_json::json!({"wp_id": "WP05-T029"}));
    assert_eq!(value["span_context"]["trace_id"], "t");

    let back: LogEntry = serde_json::from_value(value).unwrap();
    assert_eq!(back.level, LogLevel::Warn);
    assert_eq!(back.message, "retrying");
    assert_eq!(
        back.fields.get("wp_id").map(String::as_str),
        Some("WP05-T029")
    );
    assert!(back.span_context.is_some());

    // A log emitted outside any span and without structured fields is valid.
    let bare: LogEntry =
        serde_json::from_str(r#"{"level":"info","message":"up","fields":{},"span_context":null}"#)
            .unwrap();
    assert_eq!(bare.level, LogLevel::Info);
    assert!(bare.fields.is_empty());
    assert!(bare.span_context.is_none());
}

// --------------------------------------------------------------- review port

#[test]
fn comment_severity_wire_names_cover_every_variant() {
    for (severity, wire) in [
        (CommentSeverity::Critical, "critical"),
        (CommentSeverity::Major, "major"),
        (CommentSeverity::Minor, "minor"),
        (CommentSeverity::Informational, "informational"),
    ] {
        assert_eq!(to_json(&severity), serde_json::json!(wire));
        assert_eq!(
            serde_json::from_value::<CommentSeverity>(serde_json::json!(wire)).unwrap(),
            severity
        );
    }

    for bad in ["CRITICAL", "Major", "info", "blocker"] {
        assert!(
            serde_json::from_value::<CommentSeverity>(serde_json::json!(bad)).is_err(),
            "{bad} must not deserialize into a CommentSeverity"
        );
    }
}

#[test]
fn review_comment_round_trips_optional_anchor_fields() {
    let anchored = review_comment();
    let value = to_json(&anchored);
    assert_eq!(value["author"], "coderabbit");
    assert_eq!(value["body"], "nil check missing");
    assert_eq!(value["file_path"], "src/main.rs");
    assert_eq!(value["line"], 42);
    assert_eq!(value["severity"], "major");
    assert_eq!(value["actionable"], true);

    let back: ReviewComment = serde_json::from_value(value).unwrap();
    assert_eq!(back.file_path.as_deref(), Some("src/main.rs"));
    assert_eq!(back.line, Some(42));
    assert!(back.actionable);

    // A PR-level comment has no file or line anchor.
    let unanchored: ReviewComment = serde_json::from_str(
        r#"{"author":"a","body":"b","file_path":null,"line":null,"severity":"informational","actionable":false}"#,
    )
    .unwrap();
    assert!(unanchored.file_path.is_none());
    assert!(unanchored.line.is_none());
    assert!(!unanchored.actionable);
    assert_eq!(unanchored.severity, CommentSeverity::Informational);
}

#[test]
fn review_status_tags_distinguish_every_outcome() {
    assert_eq!(
        to_json(&ReviewStatus::Pending),
        serde_json::json!("pending")
    );
    assert_eq!(
        to_json(&ReviewStatus::InProgress),
        serde_json::json!("in_progress")
    );
    assert_eq!(
        to_json(&ReviewStatus::Approved),
        serde_json::json!("approved")
    );
    assert_eq!(
        to_json(&ReviewStatus::Rejected {
            reason: "policy".to_string()
        }),
        serde_json::json!({"rejected": {"reason": "policy"}})
    );

    let changes = ReviewStatus::ChangesRequested {
        comments: vec![review_comment()],
    };
    let value = to_json(&changes);
    assert_eq!(
        value["changes_requested"]["comments"][0]["body"],
        "nil check missing"
    );
    assert_eq!(value["changes_requested"]["comments"][0]["line"], 42);

    let back: ReviewStatus = serde_json::from_value(value).unwrap();
    assert_eq!(to_json(&back), to_json(&changes));

    // Requested changes with an empty comment list is still a valid state.
    let empty: ReviewStatus =
        serde_json::from_str(r#"{"changes_requested":{"comments":[]}}"#).unwrap();
    assert_eq!(
        to_json(&empty),
        serde_json::json!({"changes_requested": {"comments": []}})
    );

    // The comment payload is required for this variant.
    assert!(serde_json::from_str::<ReviewStatus>(r#"{"changes_requested":{}}"#).is_err());
    assert!(serde_json::from_str::<ReviewStatus>(r#""changes_requested""#).is_err());
}

#[test]
fn ci_status_failed_carries_the_logs_url() {
    assert_eq!(to_json(&CiStatus::Pending), serde_json::json!("pending"));
    assert_eq!(to_json(&CiStatus::Running), serde_json::json!("running"));
    assert_eq!(to_json(&CiStatus::Passed), serde_json::json!("passed"));
    assert_eq!(
        to_json(&CiStatus::Cancelled),
        serde_json::json!("cancelled")
    );
    assert_eq!(
        to_json(&CiStatus::Failed {
            logs_url: "https://ci.example/9".to_string()
        }),
        serde_json::json!({"failed": {"logs_url": "https://ci.example/9"}})
    );

    let back: CiStatus =
        serde_json::from_str(r#"{"failed":{"logs_url":"https://ci.example/9"}}"#).unwrap();
    assert_eq!(
        to_json(&back),
        serde_json::json!({"failed": {"logs_url": "https://ci.example/9"}})
    );

    // `failed` without its log URL cannot be reconstructed.
    assert!(serde_json::from_str::<CiStatus>(r#"{"failed":"boom"}"#).is_err());
    assert!(serde_json::from_str::<CiStatus>(r#"{"failed":{}}"#).is_err());
}

#[test]
fn pr_info_round_trips_nested_review_and_ci_state() {
    let info = PrInfo {
        url: "https://github.com/o/r/pull/9".to_string(),
        number: 9,
        title: "Add auth".to_string(),
        state: "open".to_string(),
        review_status: ReviewStatus::ChangesRequested { comments: vec![] },
        ci_status: CiStatus::Failed {
            logs_url: "https://ci.example/9".to_string(),
        },
    };

    let value = to_json(&info);
    assert_eq!(value["number"], 9);
    assert_eq!(value["state"], "open");
    assert_eq!(
        value["review_status"],
        serde_json::json!({"changes_requested": {"comments": []}})
    );
    assert_eq!(
        value["ci_status"],
        serde_json::json!({"failed": {"logs_url": "https://ci.example/9"}})
    );

    let back: PrInfo = serde_json::from_value(value).unwrap();
    assert_eq!(back.number, 9);
    assert_eq!(back.title, "Add auth");
    assert_eq!(back.url, info.url);
    assert_eq!(to_json(&back.ci_status), to_json(&info.ci_status));
    assert_eq!(to_json(&back.review_status), to_json(&info.review_status));
}

// ------------------------------------------------------------------ vcs port

#[test]
fn worktree_and_branch_info_round_trip() {
    let worktree = WorktreeInfo {
        path: PathBuf::from("/repo/AgilePlus-wtrees/auth"),
        commit: "abc1234".to_string(),
        branch: "feat/auth".to_string(),
        feature_slug: "auth".to_string(),
        wp_id: "WP01".to_string(),
    };
    let value = to_json(&worktree);
    assert_eq!(value["path"], "/repo/AgilePlus-wtrees/auth");
    assert_eq!(value["branch"], "feat/auth");
    assert_eq!(value["feature_slug"], "auth");
    let back: WorktreeInfo = serde_json::from_value(value).unwrap();
    assert_eq!(back.path, worktree.path);
    assert_eq!(back.commit, "abc1234");
    assert_eq!(back.wp_id, "WP01");

    let remote = BranchInfo {
        name: "main".to_string(),
        commit: "deadbee".to_string(),
        is_remote: true,
    };
    let value = to_json(&remote);
    assert_eq!(value["is_remote"], true);
    let back: BranchInfo = serde_json::from_value(value).unwrap();
    assert_eq!(back.name, "main");
    assert!(back.is_remote);

    let local = BranchInfo {
        is_remote: false,
        ..remote
    };
    assert_eq!(to_json(&local)["is_remote"], false);
}

#[test]
fn merge_result_round_trips_conflicts_and_optional_commit() {
    let merged = MergeResult {
        success: true,
        conflicts: vec![],
        merged_commit: Some("cafe123".to_string()),
        commit: Some("cafe123".to_string()),
        message: Some("merged cleanly".to_string()),
    };
    let value = to_json(&merged);
    assert_eq!(value["success"], true);
    assert_eq!(value["merged_commit"], "cafe123");
    assert_eq!(value["commit"], "cafe123");
    assert_eq!(value["conflicts"], serde_json::json!([]));

    let conflicted = MergeResult {
        success: false,
        conflicts: vec![ConflictInfo {
            path: "src/main.rs".to_string(),
            file_path: "src/main.rs".to_string(),
            conflict_type: "content".to_string(),
            ours: Some("ours".to_string()),
            theirs: Some("theirs".to_string()),
        }],
        merged_commit: None,
        commit: None,
        message: None,
    };
    let value = to_json(&conflicted);
    assert_eq!(value["conflicts"][0]["conflict_type"], "content");
    assert!(value.get("merged_commit").is_some());
    assert_eq!(value["merged_commit"], serde_json::Value::Null);
    assert_eq!(value["message"], serde_json::Value::Null);

    let back: MergeResult = serde_json::from_value(value).unwrap();
    assert!(!back.success);
    assert_eq!(back.conflicts.len(), 1);
    assert_eq!(back.conflicts[0].file_path, "src/main.rs");
    assert!(back.merged_commit.is_none());
    assert!(back.commit.is_none());
    assert!(back.message.is_none());
}

#[test]
fn conflict_info_allows_either_side_to_be_absent() {
    let delete_modify = ConflictInfo {
        path: "gone.rs".to_string(),
        file_path: "gone.rs".to_string(),
        conflict_type: "delete/modify".to_string(),
        ours: None,
        theirs: Some("content".to_string()),
    };
    let value = to_json(&delete_modify);
    assert_eq!(value["ours"], serde_json::Value::Null);
    assert_eq!(value["theirs"], "content");
    assert_eq!(value["conflict_type"], "delete/modify");

    let back: ConflictInfo = serde_json::from_value(value).unwrap();
    assert!(back.ours.is_none());
    assert_eq!(back.theirs.as_deref(), Some("content"));
    assert_eq!(back.path, "gone.rs");

    let add_add: ConflictInfo = serde_json::from_str(
        r#"{"path":"p.rs","file_path":"p.rs","conflict_type":"add/add","ours":null,"theirs":null}"#,
    )
    .unwrap();
    assert!(add_add.ours.is_none());
    assert!(add_add.theirs.is_none());
}

#[test]
fn feature_artifacts_round_trips_optional_artifacts_and_extra_list() {
    let full = FeatureArtifacts {
        spec: Some("specs/auth.md".to_string()),
        research: Some("research.md".to_string()),
        plan: Some("plan.md".to_string()),
        other: vec!["tasks.md".to_string(), "data-model.md".to_string()],
        meta_json: Some("{}".to_string()),
        audit_chain: Some("chain.jsonl".to_string()),
        evidence_paths: vec!["evidence/1.json".to_string()],
    };
    let value = to_json(&full);
    assert_eq!(value["spec"], "specs/auth.md");
    assert_eq!(
        value["other"],
        serde_json::json!(["tasks.md", "data-model.md"])
    );
    assert_eq!(
        value["evidence_paths"],
        serde_json::json!(["evidence/1.json"])
    );
    assert_eq!(value["meta_json"], "{}");

    let back: FeatureArtifacts = serde_json::from_value(value).unwrap();
    assert_eq!(back.other, full.other);
    assert_eq!(back.evidence_paths, full.evidence_paths);
    assert_eq!(back.audit_chain.as_deref(), Some("chain.jsonl"));

    // Nothing discovered yet: every optional artifact is null, not missing.
    let empty: FeatureArtifacts = serde_json::from_str(
        r#"{"spec":null,"research":null,"plan":null,"other":[],"meta_json":null,"audit_chain":null,"evidence_paths":[]}"#,
    )
    .unwrap();
    assert!(empty.spec.is_none());
    assert!(empty.plan.is_none());
    assert!(empty.meta_json.is_none());
    assert!(empty.other.is_empty());
    assert!(empty.evidence_paths.is_empty());
}

// --------------------------------------------------------------- events port

#[test]
fn domain_event_variants_clone_and_debug_with_their_payloads() {
    let events = vec![
        DomainEvent::FeatureCreated {
            id: 1,
            slug: "auth".to_string(),
        },
        DomainEvent::FeatureStateAdvanced {
            id: 1,
            from: "planned".to_string(),
            to: "implementing".to_string(),
        },
        DomainEvent::StoryCreated {
            id: 2,
            epic_id: 3,
            title: "Login".to_string(),
        },
        DomainEvent::StoryStatusChanged {
            id: 2,
            from: "todo".to_string(),
            to: "done".to_string(),
        },
        DomainEvent::EpicCreated {
            id: 3,
            project_id: 4,
            title: "Auth".to_string(),
        },
    ];
    let tags = [
        "FeatureCreated",
        "FeatureStateAdvanced",
        "StoryCreated",
        "StoryStatusChanged",
        "EpicCreated",
    ];

    for (event, tag) in events.iter().zip(tags) {
        let debug = format!("{event:?}");
        assert!(
            debug.starts_with(tag),
            "debug {debug} should start with {tag}"
        );
        assert_eq!(
            format!("{:?}", event.clone()),
            debug,
            "Clone must preserve every payload field"
        );
    }

    match events[1].clone() {
        DomainEvent::FeatureStateAdvanced { id, from, to } => {
            assert_eq!(
                (id, from.as_str(), to.as_str()),
                (1, "planned", "implementing")
            );
        }
        other => panic!("Clone changed the variant: {other:?}"),
    }
    match events[3].clone() {
        DomainEvent::StoryStatusChanged { id, from, to } => {
            assert_eq!((id, from.as_str(), to.as_str()), (2, "todo", "done"));
        }
        other => panic!("Clone changed the variant: {other:?}"),
    }
}

#[derive(Default)]
struct RecordingPublisher {
    events: Mutex<Vec<DomainEvent>>,
    fail: bool,
}

impl DomainEventPublisher for RecordingPublisher {
    fn publish(&self, event: DomainEvent) -> Result<(), DomainError> {
        if self.fail {
            return Err(DomainError::Storage("event bus down".to_string()));
        }
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

#[test]
fn domain_event_publisher_dispatches_through_a_trait_object() {
    let publisher = RecordingPublisher::default();
    let port: &dyn DomainEventPublisher = &publisher;

    port.publish(DomainEvent::FeatureCreated {
        id: 11,
        slug: "auth".to_string(),
    })
    .unwrap();
    port.publish(DomainEvent::EpicCreated {
        id: 22,
        project_id: 1,
        title: "Auth".to_string(),
    })
    .unwrap();

    let recorded = publisher.events.lock().unwrap();
    assert_eq!(recorded.len(), 2, "both events must reach the publisher");
    match &recorded[0] {
        DomainEvent::FeatureCreated { id, slug } => {
            assert_eq!((*id, slug.as_str()), (11, "auth"));
        }
        other => panic!("first event was {other:?}"),
    }
    match &recorded[1] {
        DomainEvent::EpicCreated {
            id,
            project_id,
            title,
        } => assert_eq!((*id, *project_id, title.as_str()), (22, 1, "Auth")),
        other => panic!("second event was {other:?}"),
    }
    drop(recorded);

    // A failing bus must surface the error rather than swallow the event.
    let failing = RecordingPublisher {
        fail: true,
        ..RecordingPublisher::default()
    };
    let port: &dyn DomainEventPublisher = &failing;
    assert!(matches!(
        port.publish(DomainEvent::StoryCreated {
            id: 1,
            epic_id: 2,
            title: "S".to_string(),
        }),
        Err(DomainError::Storage(message)) if message == "event bus down"
    ));
    assert!(failing.events.lock().unwrap().is_empty());
}

/// Compile-time API guard: these ports are consumed as trait objects by the
/// API, telemetry, sqlite, and gRPC crates. Changing any of their signatures to
/// bare `impl Future` in trait position would silently break those consumers.
#[test]
fn ports_consumed_as_trait_objects_stay_object_safe() {
    // Naming each trait object type is the assertion: a signature change to
    // `impl Future` in trait position would stop this from compiling.
    let _: Option<&'static dyn ObservabilityPort> = None;
    let _: Option<&'static dyn DomainEventPublisher> = None;
    let _: Option<&'static dyn agileplus_domain::ports::VcsPort> = None;
    let _: Option<&'static dyn agileplus_domain::ports::TriagePort> = None;
    let _: Option<&'static dyn agileplus_domain::ports::StoragePort> = None;
    let _: Option<&'static dyn agileplus_domain::ports::ContentStoragePort> = None;
    let _: Option<&'static dyn agileplus_domain::ports::StoryRepository> = None;
    let _: Option<&'static dyn agileplus_domain::ports::EpicRepository> = None;

    // The sync telemetry port is reached through a trait object at runtime, not
    // only at compile time: dispatch must land on the implementation.
    #[derive(Default)]
    struct CountingObservability {
        spans: Mutex<usize>,
        gauges: Mutex<Vec<f64>>,
    }
    impl ObservabilityPort for CountingObservability {
        fn start_span(&self, _: &str, _: Option<&SpanContext>) -> SpanContext {
            *self.spans.lock().unwrap() += 1;
            SpanContext {
                trace_id: String::new(),
                span_id: String::new(),
                parent_span_id: None,
            }
        }
        fn end_span(&self, _: &SpanContext) {}
        fn add_span_event(&self, _: &SpanContext, _: &str, _: &[(&str, &str)]) {}
        fn set_span_error(&self, _: &SpanContext, _: &str) {}
        fn record_counter(&self, _: &str, _: u64, _: &[(&str, &str)]) {}
        fn record_histogram(&self, _: &str, _: f64, _: &[(&str, &str)]) {}
        fn record_gauge(&self, _: &str, value: f64, _: &[(&str, &str)]) {
            self.gauges.lock().unwrap().push(value);
        }
        fn log(&self, _: &LogEntry) {}
        fn log_info(&self, _: &str) {}
        fn log_warn(&self, _: &str) {}
        fn log_error(&self, _: &str) {}
    }

    let counting = CountingObservability::default();
    let port: &dyn ObservabilityPort = &counting;
    port.record_gauge("queue.depth", 12.5, &[]);
    port.start_span("dispatch", None);
    assert_eq!(*counting.spans.lock().unwrap(), 1);
    assert_eq!(*counting.gauges.lock().unwrap(), vec![12.5]);
}
