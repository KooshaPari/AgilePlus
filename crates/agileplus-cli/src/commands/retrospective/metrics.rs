use chrono::{DateTime, Utc};

use agileplus_domain::domain::audit::AuditEntry;
use agileplus_domain::domain::metric::Metric;
use agileplus_domain::domain::work_package::WorkPackage;

/// Per-WP performance data.
#[derive(Debug, Clone)]
pub(crate) struct WpMetrics {
    pub(crate) sequence: i32,
    pub(crate) title: String,
    pub(crate) agent_runs: i32,
    pub(crate) review_cycles: i32,
    pub(crate) duration_ms: i64,
}

/// Aggregated feature metrics for the retrospective.
#[derive(Debug)]
pub(crate) struct FeatureMetrics {
    pub(crate) total_duration_ms: i64,
    pub(crate) wp_count: usize,
    pub(crate) total_agent_runs: i32,
    pub(crate) total_review_cycles: i32,
    pub(crate) avg_review_cycles_per_wp: f64,
    pub(crate) state_transition_durations: Vec<(String, i64)>,
    pub(crate) governance_exceptions: Vec<String>,
    pub(crate) high_review_wps: Vec<(i32, String, i32)>,
    pub(crate) wp_metrics: Vec<WpMetrics>,
}

pub(crate) fn collect_feature_metrics(
    feature_id: i64,
    created_at: &DateTime<Utc>,
    wps: &[WorkPackage],
    audit_trail: &[AuditEntry],
    metrics_data: &[Metric],
) -> FeatureMetrics {
    let (total_duration_ms, state_transition_durations) =
        compute_durations_from_audit(audit_trail, created_at);

    let governance_exceptions: Vec<String> = audit_trail
        .iter()
        .filter(|e| e.transition.contains("skipped") || e.transition.contains("exception"))
        .map(|e| format!("{}: {}", e.timestamp.format("%Y-%m-%d"), e.transition))
        .collect();

    let wp_metrics: Vec<WpMetrics> = wps
        .iter()
        .map(|wp| {
            let wp_metric = metrics_data.iter().find(|m| {
                m.feature_id == Some(feature_id)
                    && m.command.contains(&format!("WP{:02}", wp.sequence))
            });

            WpMetrics {
                sequence: wp.sequence,
                title: wp.title.clone(),
                agent_runs: wp_metric.map(|m| m.agent_runs).unwrap_or(0),
                review_cycles: wp_metric.map(|m| m.review_cycles).unwrap_or(0),
                duration_ms: wp_metric.map(|m| m.duration_ms).unwrap_or(0),
            }
        })
        .collect();

    let total_agent_runs: i32 = wp_metrics.iter().map(|w| w.agent_runs).sum::<i32>()
        + metrics_data.iter().map(|m| m.agent_runs).sum::<i32>();

    let total_review_cycles: i32 = wp_metrics.iter().map(|w| w.review_cycles).sum::<i32>()
        + metrics_data.iter().map(|m| m.review_cycles).sum::<i32>();

    let avg_review_cycles = if !wps.is_empty() {
        total_review_cycles as f64 / wps.len() as f64
    } else {
        0.0
    };

    let high_review_wps: Vec<(i32, String, i32)> = wp_metrics
        .iter()
        .filter(|w| w.review_cycles > 3)
        .map(|w| (w.sequence, w.title.clone(), w.review_cycles))
        .collect();

    FeatureMetrics {
        total_duration_ms,
        wp_count: wps.len(),
        total_agent_runs,
        total_review_cycles,
        avg_review_cycles_per_wp: avg_review_cycles,
        state_transition_durations,
        governance_exceptions,
        high_review_wps,
        wp_metrics,
    }
}

pub(crate) fn generate_insights(metrics: &FeatureMetrics) -> Vec<String> {
    let mut insights = Vec::new();

    if metrics.avg_review_cycles_per_wp > 3.0 {
        insights.push(format!(
            "High average review cycles ({:.1} per WP) suggest acceptance criteria may be unclear \
            or code areas are complex. Consider breaking future WPs into smaller units.",
            metrics.avg_review_cycles_per_wp
        ));
    }

    if !metrics.high_review_wps.is_empty() {
        let wp_list: Vec<String> = metrics
            .high_review_wps
            .iter()
            .map(|(seq, title, cycles)| format!("WP{:02} '{}' ({} cycles)", seq, title, cycles))
            .collect();
        insights.push(format!(
            "WPs with >3 review cycles (potential bottlenecks): {}",
            wp_list.join(", ")
        ));
    }

    if metrics.wp_count > 0 {
        let avg_agent_per_wp = metrics.total_agent_runs as f64 / metrics.wp_count as f64;
        if avg_agent_per_wp > 5.0 {
            insights.push(format!(
                "High agent invocation rate ({:.1} per WP) suggests agent failures or restarts. \
                Consider improving prompts or adding pre-flight checks.",
                avg_agent_per_wp
            ));
        }
    }

    if !metrics.governance_exceptions.is_empty() {
        insights.push(format!(
            "{} governance exception(s) occurred. Review whether contracts are too strict \
            or process is unclear.",
            metrics.governance_exceptions.len()
        ));
    }

    if metrics.total_duration_ms > 0 {
        let implement_fraction = metrics
            .wp_metrics
            .iter()
            .map(|w| w.duration_ms)
            .sum::<i64>() as f64
            / metrics.total_duration_ms as f64;
        if implement_fraction > 0.5 {
            insights.push(format!(
                "{:.0}% of total time was spent in implementation/review phases. \
                Consider splitting WPs into smaller units to improve flow.",
                implement_fraction * 100.0
            ));
        }
    }

    if insights.is_empty() {
        insights
            .push("No significant issues detected. Development process is healthy.".to_string());
    }

    insights
}

pub(crate) fn generate_constitution_suggestions(metrics: &FeatureMetrics) -> Vec<String> {
    let mut suggestions = Vec::new();

    if metrics.avg_review_cycles_per_wp > 3.0 {
        suggestions.push(
            "Consider adding a pre-review self-check rule to the governance constitution:\n\
            ```toml\n\
            [[rules]]\n\
            name = \"pre-review-self-check\"\n\
            description = \"Agent must verify acceptance criteria before requesting review\"\n\
            trigger = \"doing -> review\"\n\
            ```"
            .to_string(),
        );
    }

    if !metrics.governance_exceptions.is_empty() {
        suggestions.push(
            "Governance exceptions occurred. Consider adding a fast-track path:\n\
            ```toml\n\
            [[rules]]\n\
            name = \"fast-track\"\n\
            description = \"Allow expedited transitions with documented rationale\"\n\
            allow_skip = true\n\
            require_justification = true\n\
            ```"
            .to_string(),
        );
    }

    if suggestions.is_empty() {
        suggestions.push(
            "No constitution amendments suggested. Current governance is appropriate.".to_string(),
        );
    }

    suggestions
}

pub(crate) fn format_duration(ms: i64) -> String {
    if ms < 0 {
        return "N/A".to_string();
    }
    let secs = ms / 1000;
    let minutes = secs / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if days > 0 {
        format!("{}d {}h", days, hours % 24)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes % 60)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

pub(crate) fn compute_durations_from_audit(
    trail: &[AuditEntry],
    created_at: &DateTime<Utc>,
) -> (i64, Vec<(String, i64)>) {
    if trail.is_empty() {
        return (0, vec![]);
    }

    let last_ts = trail.last().map(|e| e.timestamp).unwrap_or_else(Utc::now);
    let total_ms = (last_ts - *created_at).num_milliseconds().max(0);

    let mut phase_durations = Vec::new();
    for window in trail.windows(2) {
        let from = &window[0];
        let to = &window[1];
        let dur_ms = (to.timestamp - from.timestamp).num_milliseconds().max(0);
        phase_durations.push((from.transition.clone(), dur_ms));
    }

    (total_ms, phase_durations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn make_audit(transition: &str, ts: DateTime<Utc>) -> AuditEntry {
        let mut e = AuditEntry {
            id: 0,
            feature_id: 1,
            wp_id: None,
            timestamp: ts,
            actor: "user".into(),
            transition: transition.into(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        e.hash = agileplus_domain::domain::audit::hash_entry(&e);
        e
    }

    fn make_wp(feature_id: i64, title: &str, sequence: i32) -> WorkPackage {
        WorkPackage::new(feature_id, title, sequence, "- criteria")
    }

    fn make_metric(
        feature_id: i64,
        command: &str,
        agent_runs: i32,
        review_cycles: i32,
        duration_ms: i64,
    ) -> Metric {
        Metric {
            id: 0,
            feature_id: Some(feature_id),
            command: command.to_string(),
            duration_ms,
            agent_runs,
            review_cycles,
            metadata: None,
            timestamp: Utc::now(),
        }
    }

    fn sample_metrics() -> FeatureMetrics {
        FeatureMetrics {
            total_duration_ms: 3600000,
            wp_count: 3,
            total_agent_runs: 6,
            total_review_cycles: 3,
            avg_review_cycles_per_wp: 1.0,
            state_transition_durations: vec![],
            governance_exceptions: vec![],
            high_review_wps: vec![],
            wp_metrics: vec![],
        }
    }

    // ── format_duration ───────────────────────────────────────────────

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(0), "0s");
    }

    #[test]
    fn format_duration_secs() {
        assert_eq!(format_duration(5000), "5s");
    }

    #[test]
    fn format_duration_subsecond() {
        assert_eq!(format_duration(500), "0s");
    }

    #[test]
    fn format_duration_minutes() {
        assert_eq!(format_duration(65000), "1m 5s");
    }

    #[test]
    fn format_duration_exact_minute() {
        assert_eq!(format_duration(60000), "1m 0s");
    }

    #[test]
    fn format_duration_hours() {
        assert_eq!(format_duration(3660000), "1h 1m");
    }

    #[test]
    fn format_duration_exact_hour() {
        assert_eq!(format_duration(3600000), "1h 0m");
    }

    #[test]
    fn format_duration_days() {
        let ms = (2 * 24 * 3600 + 3 * 3600) * 1000i64;
        assert_eq!(format_duration(ms), "2d 3h");
    }

    #[test]
    fn format_duration_multiple_days() {
        let ms = (5 * 24 * 3600 + 3 * 3600) * 1000i64;
        assert_eq!(format_duration(ms), "5d 3h");
    }

    #[test]
    fn format_duration_negative() {
        assert_eq!(format_duration(-100), "N/A");
    }

    // ── compute_durations_from_audit ──────────────────────────────────

    #[test]
    fn compute_durations_empty_trail() {
        let now = Utc::now();
        let (total, phases) = compute_durations_from_audit(&[], &now);
        assert_eq!(total, 0);
        assert!(phases.is_empty());
    }

    #[test]
    fn compute_durations_single_entry() {
        let created = Utc::now() - Duration::hours(2);
        let e = make_audit("Created -> Specified", Utc::now());
        let (total, phases) = compute_durations_from_audit(&[e], &created);
        assert!(total > 0);
        assert!(phases.is_empty());
    }

    #[test]
    fn compute_durations_two_entries() {
        let base = Utc::now();
        let t0 = base - Duration::hours(3);
        let t1 = base - Duration::hours(1);
        let e0 = make_audit("Created -> Specified", t0);
        let e1 = make_audit("Specified -> Researched", t1);
        let (total, phases) = compute_durations_from_audit(&[e0, e1], &t0);
        assert!(total > 0);
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].0, "Created -> Specified");
        assert!(phases[0].1 > 0);
    }

    #[test]
    fn compute_durations_three_entries() {
        let base = Utc::now();
        let t0 = base - Duration::hours(10);
        let t1 = base - Duration::hours(6);
        let t2 = base - Duration::hours(1);
        let e0 = make_audit("Created -> Specified", t0);
        let e1 = make_audit("Specified -> Researched", t1);
        let e2 = make_audit("Researched -> Implemented", t2);
        let (total, phases) = compute_durations_from_audit(&[e0, e1, e2], &t0);
        assert!(total > 0);
        assert_eq!(phases.len(), 2);
        assert_eq!(phases[0].0, "Created -> Specified");
        assert_eq!(phases[1].0, "Specified -> Researched");
    }

    #[test]
    fn compute_durations_negative_clamped_to_zero() {
        let base = Utc::now();
        let t0 = base - Duration::hours(1);
        let e0 = make_audit("A -> B", base); // later
        let e1 = make_audit("B -> C", t0); // earlier — inverted
        let (_, phases) = compute_durations_from_audit(&[e0, e1], &base);
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].1, 0);
    }

    // ── generate_insights ─────────────────────────────────────────────

    #[test]
    fn generate_insights_healthy() {
        let insights = generate_insights(&sample_metrics());
        assert_eq!(insights.len(), 1);
        assert!(insights[0].contains("healthy") || insights[0].contains("No significant"));
    }

    #[test]
    fn generate_insights_high_review_cycles() {
        let mut m = sample_metrics();
        m.avg_review_cycles_per_wp = 5.0;
        let insights = generate_insights(&m);
        let combined = insights.join(" ");
        assert!(combined.contains("review cycles"));
    }

    #[test]
    fn generate_insights_high_review_wps() {
        let mut m = sample_metrics();
        m.high_review_wps = vec![(1, "Auth".to_string(), 5), (3, "DB".to_string(), 4)];
        let insights = generate_insights(&m);
        let combined = insights.join(" ");
        assert!(combined.contains("WP01"));
        assert!(combined.contains("WP03"));
        assert!(combined.contains("bottleneck"));
    }

    #[test]
    fn generate_insights_agent_rate_below_threshold() {
        let mut m = sample_metrics();
        m.total_agent_runs = 12; // 12/3 = 4 per wp, < 5
        let insights = generate_insights(&m);
        let combined = insights.join(" ");
        assert!(!combined.contains("agent invocation rate"));
    }

    #[test]
    fn generate_insights_agent_rate_above_threshold() {
        let mut m = sample_metrics();
        m.total_agent_runs = 18; // 18/3 = 6 per WP, > 5
        let insights = generate_insights(&m);
        let combined = insights.join(" ");
        assert!(combined.contains("agent invocation rate"));
    }

    #[test]
    fn generate_insights_governance_exceptions() {
        let mut m = sample_metrics();
        m.governance_exceptions = vec!["skipped review".into(), "exception".into()];
        let insights = generate_insights(&m);
        let combined = insights.join(" ");
        assert!(combined.contains("2 governance exception"));
    }

    #[test]
    fn generate_insights_implementation_fraction_high() {
        let mut m = sample_metrics();
        m.total_duration_ms = 10000;
        m.wp_metrics = vec![WpMetrics {
            sequence: 1,
            title: "WP01".into(),
            agent_runs: 1,
            review_cycles: 0,
            duration_ms: 8000,
        }];
        let insights = generate_insights(&m);
        let combined = insights.join(" ");
        assert!(combined.contains("80%"));
    }

    #[test]
    fn generate_insights_implementation_fraction_low_no_warning() {
        let mut m = sample_metrics();
        m.total_duration_ms = 10000;
        m.wp_metrics = vec![WpMetrics {
            sequence: 1,
            title: "WP01".into(),
            agent_runs: 1,
            review_cycles: 0,
            duration_ms: 3000,
        }];
        let insights = generate_insights(&m);
        let combined = insights.join(" ");
        assert!(!combined.contains("% of total time"));
    }

    #[test]
    fn generate_insights_zero_wp_count_skips_agent_rate() {
        let mut m = sample_metrics();
        m.wp_count = 0;
        m.total_agent_runs = 100;
        let insights = generate_insights(&m);
        let combined = insights.join(" ");
        assert!(!combined.contains("agent invocation"));
    }

    // ── generate_constitution_suggestions ─────────────────────────────

    #[test]
    fn generate_constitution_suggestions_healthy() {
        let suggestions = generate_constitution_suggestions(&sample_metrics());
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].contains("No constitution amendments"));
    }

    #[test]
    fn generate_constitution_suggestions_high_review() {
        let mut m = sample_metrics();
        m.avg_review_cycles_per_wp = 5.0;
        let suggestions = generate_constitution_suggestions(&m);
        let combined = suggestions.join(" ");
        assert!(combined.contains("pre-review"));
        assert!(combined.contains("doing -> review"));
    }

    #[test]
    fn generate_constitution_suggestions_governance() {
        let mut m = sample_metrics();
        m.governance_exceptions = vec!["skipped".into()];
        let suggestions = generate_constitution_suggestions(&m);
        let combined = suggestions.join(" ");
        assert!(combined.contains("fast-track"));
    }

    #[test]
    fn generate_constitution_suggestions_both() {
        let mut m = sample_metrics();
        m.avg_review_cycles_per_wp = 4.0;
        m.governance_exceptions = vec!["exception".into()];
        let suggestions = generate_constitution_suggestions(&m);
        assert_eq!(suggestions.len(), 2);
    }

    // ── collect_feature_metrics ───────────────────────────────────────

    #[test]
    fn collect_feature_metrics_empty_data() {
        let now = Utc::now();
        let metrics = collect_feature_metrics(1, &now, &[], &[], &[]);
        assert_eq!(metrics.wp_count, 0);
        assert_eq!(metrics.total_duration_ms, 0);
        assert!(metrics.wp_metrics.is_empty());
        assert!(metrics.governance_exceptions.is_empty());
    }

    #[test]
    fn collect_feature_metrics_with_wps_no_metrics() {
        let created = Utc::now() - Duration::hours(1);
        let wp1 = make_wp(1, "Auth (WP01)", 1);
        let wp2 = make_wp(1, "API (WP02)", 2);
        let metrics = collect_feature_metrics(1, &created, &[wp1, wp2], &[], &[]);
        assert_eq!(metrics.wp_count, 2);
        assert_eq!(metrics.total_agent_runs, 0);
        assert_eq!(metrics.total_review_cycles, 0);
        assert_eq!(metrics.avg_review_cycles_per_wp, 0.0);
    }

    #[test]
    fn collect_feature_metrics_with_matching_metrics() {
        let created = Utc::now() - Duration::hours(2);
        let wp1 = make_wp(1, "Auth (WP01)", 1);
        let audit = make_audit("Created -> Done", Utc::now());
        let metric = make_metric(1, "WP01 plan", 3, 2, 5000);
        let metrics = collect_feature_metrics(1, &created, &[wp1], &[audit], &[metric]);
        assert_eq!(metrics.wp_count, 1);
        assert_eq!(metrics.total_agent_runs, 3);
        assert_eq!(metrics.total_review_cycles, 2);
        assert_eq!(metrics.avg_review_cycles_per_wp, 2.0);
        assert_eq!(metrics.wp_metrics[0].agent_runs, 3);
    }

    #[test]
    fn collect_feature_metrics_governance_exceptions_detected() {
        let created = Utc::now() - Duration::hours(1);
        let e1 = make_audit("Created -> Skipped review", Utc::now());
        let e2 = make_audit("Doing -> exception: fast-track", Utc::now());
        let metrics = collect_feature_metrics(1, &created, &[], &[e1, e2], &[]);
        assert_eq!(metrics.governance_exceptions.len(), 2);
    }

    #[test]
    fn collect_feature_metrics_high_review_wps_detected() {
        let created = Utc::now() - Duration::hours(1);
        let wp1 = make_wp(1, "Auth (WP01)", 1);
        let metric = make_metric(1, "WP01 plan", 2, 5, 3000);
        let metrics = collect_feature_metrics(1, &created, &[wp1], &[], &[metric]);
        assert_eq!(metrics.high_review_wps.len(), 1);
        assert_eq!(metrics.high_review_wps[0].0, 1);
        assert_eq!(metrics.high_review_wps[0].2, 5);
    }

    #[test]
    fn collect_feature_metrics_state_transitions() {
        let base = Utc::now();
        let t0 = base - Duration::hours(5);
        let t1 = base - Duration::hours(3);
        let t2 = base - Duration::hours(1);
        let e0 = make_audit("Created -> Specified", t0);
        let e1 = make_audit("Specified -> Researched", t1);
        let e2 = make_audit("Researched -> Done", t2);
        let metrics = collect_feature_metrics(1, &t0, &[], &[e0, e1, e2], &[]);
        assert_eq!(metrics.state_transition_durations.len(), 2);
        assert!(metrics.state_transition_durations[0].1 > 0);
        assert!(metrics.state_transition_durations[1].1 > 0);
    }

    #[test]
    fn collect_feature_metrics_agent_runs_summed() {
        let created = Utc::now() - Duration::hours(1);
        let wp1 = make_wp(1, "A (WP01)", 1);
        let wp2 = make_wp(1, "B (WP02)", 2);
        let m1 = make_metric(1, "WP01 plan", 3, 1, 1000);
        let m2 = make_metric(1, "WP02 plan", 5, 2, 2000);
        let m3 = make_metric(1, "overall", 2, 0, 500);
        let metrics = collect_feature_metrics(1, &created, &[wp1, wp2], &[], &[m1, m2, m3]);
        assert_eq!(metrics.total_agent_runs, 10);
        assert_eq!(metrics.total_review_cycles, 3);
    }

    #[test]
    fn collect_feature_metrics_no_matching_metric_for_wp() {
        let created = Utc::now() - Duration::hours(1);
        let wp1 = make_wp(1, "Auth (WP01)", 1);
        let metric = make_metric(1, "WP99 plan", 5, 5, 9000);
        let metrics = collect_feature_metrics(1, &created, &[wp1], &[], &[metric]);
        assert_eq!(metrics.wp_metrics[0].agent_runs, 0);
        assert_eq!(metrics.wp_metrics[0].review_cycles, 0);
    }
}
