use super::metrics::{
    FeatureMetrics, format_duration, generate_constitution_suggestions, generate_insights,
};

pub(crate) fn generate_retro_markdown(
    feature_slug: &str,
    feature_name: &str,
    metrics: &FeatureMetrics,
    verbose: bool,
) -> String {
    let insights = generate_insights(metrics);
    let suggestions = generate_constitution_suggestions(metrics);

    let mut lines = vec![
        format!("# Retrospective: {feature_name}"),
        format!(
            "**Feature**: `{feature_slug}` | **Generated**: {}",
            chrono::Utc::now().format("%Y-%m-%d")
        ),
        String::new(),
        "## Summary".to_string(),
        String::new(),
        format!(
            "- **Total duration**: {}",
            format_duration(metrics.total_duration_ms)
        ),
        format!("- **Work packages**: {}", metrics.wp_count),
        format!(
            "- **Total agent invocations**: {}",
            metrics.total_agent_runs
        ),
        format!(
            "- **Total review cycles**: {} (avg {:.1} per WP)",
            metrics.total_review_cycles, metrics.avg_review_cycles_per_wp
        ),
        format!(
            "- **Governance exceptions**: {}",
            metrics.governance_exceptions.len()
        ),
        String::new(),
    ];

    if !metrics.state_transition_durations.is_empty() {
        lines.push("## Phase Breakdown".to_string());
        lines.push(String::new());
        lines.push("| Phase | Duration |".to_string());
        lines.push("|-------|----------|".to_string());
        for (phase, dur_ms) in &metrics.state_transition_durations {
            lines.push(format!("| {} | {} |", phase, format_duration(*dur_ms)));
        }
        lines.push(String::new());
    }

    if !metrics.wp_metrics.is_empty() {
        lines.push("## WP Performance".to_string());
        lines.push(String::new());
        lines.push("| WP | Title | Agent Runs | Review Cycles | Duration |".to_string());
        lines.push("|----|-------|------------|---------------|----------|".to_string());
        for wp in &metrics.wp_metrics {
            lines.push(format!(
                "| WP{:02} | {} | {} | {} | {} |",
                wp.sequence,
                &wp.title[..wp.title.len().min(40)],
                wp.agent_runs,
                wp.review_cycles,
                format_duration(wp.duration_ms),
            ));
        }
        lines.push(String::new());
    }

    lines.push("## Insights".to_string());
    lines.push(String::new());
    for insight in &insights {
        lines.push(format!("- {insight}"));
    }
    lines.push(String::new());

    lines.push("## Suggested Constitution Amendments".to_string());
    lines.push(String::new());
    for suggestion in &suggestions {
        lines.push(suggestion.clone());
        lines.push(String::new());
    }

    if verbose && !metrics.governance_exceptions.is_empty() {
        lines.push("## Governance Exceptions".to_string());
        lines.push(String::new());
        for exc in &metrics.governance_exceptions {
            lines.push(format!("- {exc}"));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::metrics::{FeatureMetrics, WpMetrics};
    use super::generate_retro_markdown;

    fn sample_metrics() -> FeatureMetrics {
        FeatureMetrics {
            total_duration_ms: 3600000,
            wp_count: 2,
            total_agent_runs: 4,
            total_review_cycles: 2,
            avg_review_cycles_per_wp: 1.0,
            state_transition_durations: vec![],
            governance_exceptions: vec![],
            high_review_wps: vec![],
            wp_metrics: vec![],
        }
    }

    #[test]
    fn contains_header_and_summary() {
        let report = generate_retro_markdown("my-feat", "My Feature", &sample_metrics(), false);
        assert!(report.contains("# Retrospective: My Feature"));
        assert!(report.contains("## Summary"));
        assert!(report.contains("`my-feat`"));
    }

    #[test]
    fn contains_summary_metrics() {
        let m = sample_metrics();
        let report = generate_retro_markdown("f", "F", &m, false);
        assert!(report.contains("Work packages**: 2"));
        assert!(report.contains("agent invocations**: 4"));
        assert!(report.contains("review cycles**: 2"));
        assert!(report.contains("Governance exceptions**: 0"));
    }

    #[test]
    fn phase_breakdown_shown_when_transitions_present() {
        let mut m = sample_metrics();
        m.state_transition_durations = vec![
            ("Created -> Specified".to_string(), 3600000),
            ("Specified -> Researched".to_string(), 7200000),
        ];
        let report = generate_retro_markdown("f", "F", &m, false);
        assert!(report.contains("## Phase Breakdown"));
        assert!(report.contains("Created -> Specified"));
    }

    #[test]
    fn phase_breakdown_absent_when_empty() {
        let report = generate_retro_markdown("f", "F", &sample_metrics(), false);
        assert!(!report.contains("## Phase Breakdown"));
    }

    #[test]
    fn wp_performance_shown_when_wps_present() {
        let mut m = sample_metrics();
        m.wp_metrics = vec![WpMetrics {
            sequence: 1,
            title: "Auth".to_string(),
            agent_runs: 2,
            review_cycles: 1,
            duration_ms: 1800000,
        }];
        let report = generate_retro_markdown("f", "F", &m, false);
        assert!(report.contains("## WP Performance"));
        assert!(report.contains("WP01"));
        assert!(report.contains("Auth"));
    }

    #[test]
    fn wp_performance_absent_when_empty() {
        let report = generate_retro_markdown("f", "F", &sample_metrics(), false);
        assert!(!report.contains("## WP Performance"));
    }

    #[test]
    fn long_wp_title_truncated() {
        let mut m = sample_metrics();
        m.wp_metrics = vec![WpMetrics {
            sequence: 1,
            title: "A".repeat(80),
            agent_runs: 1,
            review_cycles: 0,
            duration_ms: 0,
        }];
        let report = generate_retro_markdown("f", "F", &m, false);
        assert!(report.contains("## WP Performance"));
    }

    #[test]
    fn insights_section_always_present() {
        let report = generate_retro_markdown("f", "F", &sample_metrics(), false);
        assert!(report.contains("## Insights"));
    }

    #[test]
    fn constitution_amendments_always_present() {
        let report = generate_retro_markdown("f", "F", &sample_metrics(), false);
        assert!(report.contains("## Suggested Constitution Amendments"));
    }

    #[test]
    fn verbose_shows_governance_exceptions() {
        let mut m = sample_metrics();
        m.governance_exceptions = vec!["skipped review".to_string()];
        let report = generate_retro_markdown("f", "F", &m, true);
        assert!(report.contains("## Governance Exceptions"));
        assert!(report.contains("skipped review"));
    }

    #[test]
    fn non_verbose_hides_governance_exceptions() {
        let mut m = sample_metrics();
        m.governance_exceptions = vec!["skipped review".to_string()];
        let report = generate_retro_markdown("f", "F", &m, false);
        assert!(!report.contains("## Governance Exceptions"));
    }

    #[test]
    fn verbose_without_exceptions_no_section() {
        let report = generate_retro_markdown("f", "F", &sample_metrics(), true);
        assert!(!report.contains("## Governance Exceptions"));
    }

    #[test]
    fn empty_metrics_healthy() {
        let m = FeatureMetrics {
            total_duration_ms: 0,
            wp_count: 0,
            total_agent_runs: 0,
            total_review_cycles: 0,
            avg_review_cycles_per_wp: 0.0,
            state_transition_durations: vec![],
            governance_exceptions: vec![],
            high_review_wps: vec![],
            wp_metrics: vec![],
        };
        let report = generate_retro_markdown("feat-z", "Z Feature", &m, false);
        assert!(report.contains("0s"));
        assert!(report.contains("Work packages**: 0"));
        assert!(report.contains("# Retrospective: Z Feature"));
    }

    #[test]
    fn high_review_wps_in_insights() {
        let mut m = sample_metrics();
        m.avg_review_cycles_per_wp = 5.0;
        m.high_review_wps = vec![(1, "Auth".to_string(), 7)];
        m.wp_metrics = vec![WpMetrics {
            sequence: 1,
            title: "Auth".to_string(),
            agent_runs: 5,
            review_cycles: 7,
            duration_ms: 3600000,
        }];
        let report = generate_retro_markdown("f", "F", &m, false);
        assert!(report.contains("WP01"));
        assert!(report.contains("7 cycles"));
    }

    #[test]
    fn multiple_wp_performance_rows() {
        let mut m = sample_metrics();
        m.wp_metrics = vec![
            WpMetrics {
                sequence: 1,
                title: "Auth".to_string(),
                agent_runs: 2,
                review_cycles: 1,
                duration_ms: 1000000,
            },
            WpMetrics {
                sequence: 2,
                title: "DB".to_string(),
                agent_runs: 3,
                review_cycles: 2,
                duration_ms: 2000000,
            },
            WpMetrics {
                sequence: 3,
                title: "API".to_string(),
                agent_runs: 1,
                review_cycles: 0,
                duration_ms: 500000,
            },
        ];
        let report = generate_retro_markdown("f", "F", &m, false);
        assert!(report.contains("WP01"));
        assert!(report.contains("WP02"));
        assert!(report.contains("WP03"));
    }
}
