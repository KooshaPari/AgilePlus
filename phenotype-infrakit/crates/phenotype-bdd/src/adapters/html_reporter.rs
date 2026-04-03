use crate::domain::entities::{ExecutionStatus, FeatureResult, ScenarioResult, StepResult};
use crate::domain::ports::ReportWriterPort;
use crate::BddError;

pub struct HtmlReporter;

impl HtmlReporter {
    pub fn new() -> Self {
        Self
    }

    fn status_badge_class(status: ExecutionStatus) -> &'static str {
        match status {
            ExecutionStatus::Passed => "badge-passed",
            ExecutionStatus::Failed => "badge-failed",
            ExecutionStatus::Skipped => "badge-skipped",
            ExecutionStatus::Pending => "badge-pending",
        }
    }

    fn status_text(status: ExecutionStatus) -> &'static str {
        match status {
            ExecutionStatus::Passed => "Passed",
            ExecutionStatus::Failed => "Failed",
            ExecutionStatus::Skipped => "Skipped",
            ExecutionStatus::Pending => "Pending",
        }
    }

    fn step_badge_class(result: &StepResult) -> &'static str {
        match result {
            StepResult::Passed => "step-passed",
            StepResult::Failed { .. } => "step-failed",
            StepResult::Skipped { .. } => "step-skipped",
            StepResult::Pending { .. } => "step-pending",
            StepResult::Ambiguous { .. } => "step-ambiguous",
        }
    }

    fn step_text(result: &StepResult) -> String {
        match result {
            StepResult::Passed => "✓ Passed".to_string(),
            StepResult::Failed { error, .. } => format!("✗ Failed: {}", error),
            StepResult::Skipped { reason } => format!("⊘ Skipped: {}", reason),
            StepResult::Pending { reason } => format!("�.pending: {}", reason),
            StepResult::Ambiguous { matches } => {
                format!("? Ambiguous (matched: {})", matches.len())
            }
        }
    }

    fn scenario_status(&self, scenario: &ScenarioResult) -> ExecutionStatus {
        scenario.status()
    }

    fn generate_html(&self, result: &FeatureResult) -> String {
        let passed = result
            .scenario_results
            .iter()
            .filter(|s| s.status() == ExecutionStatus::Passed)
            .count();
        let failed = result
            .scenario_results
            .iter()
            .filter(|s| s.status() == ExecutionStatus::Failed)
            .count();
        let skipped = result
            .scenario_results
            .iter()
            .filter(|s| s.status() == ExecutionStatus::Skipped)
            .count();
        let total = result.scenario_results.len();
        let pass_rate = if total > 0 {
            (passed as f64 / total as f64 * 100.0).round()
        } else {
            0.0
        };
        let total_duration: u64 = result.scenario_results.iter().map(|s| s.duration_ms).sum();

        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str(
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str("<title>BDD Test Report</title>\n");
        html.push_str("<style>\n");
        html.push_str(self.get_css());
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");

        html.push_str("<div class=\"container\">\n");
        html.push_str("<header>\n");
        html.push_str(&format!(
            "<h1>{}</h1>\n",
            self.escape_html(&result.feature_name)
        ));
        html.push_str(&format!(
            "<p class=\"timestamp\">Executed: {}</p>\n",
            result.started_at.format("%Y-%m-%d %H:%M:%S")
        ));
        html.push_str("</header>\n");

        html.push_str("<div class=\"summary\">\n");
        html.push_str("<div class=\"summary-card\">\n");
        html.push_str(&format!("<div class=\"summary-value\">{}</div>\n", total));
        html.push_str("<div class=\"summary-label\">Total Scenarios</div>\n");
        html.push_str("</div>\n");
        html.push_str("<div class=\"summary-card passed\">\n");
        html.push_str(&format!("<div class=\"summary-value\">{}</div>\n", passed));
        html.push_str("<div class=\"summary-label\">Passed</div>\n");
        html.push_str("</div>\n");
        html.push_str("<div class=\"summary-card failed\">\n");
        html.push_str(&format!("<div class=\"summary-value\">{}</div>\n", failed));
        html.push_str("<div class=\"summary-label\">Failed</div>\n");
        html.push_str("</div>\n");
        html.push_str("<div class=\"summary-card skipped\">\n");
        html.push_str(&format!("<div class=\"summary-value\">{}</div>\n", skipped));
        html.push_str("<div class=\"summary-label\">Skipped</div>\n");
        html.push_str("</div>\n");
        html.push_str("</div>\n");

        html.push_str("<div class=\"progress-section\">\n");
        html.push_str("<h2>Pass Rate</h2>\n");
        html.push_str(&format!(
            "<div class=\"progress-bar\">\n\
             <div class=\"progress-fill\" style=\"width:{}%\"></div>\n\
             </div>\n",
            pass_rate
        ));
        html.push_str(&format!(
            "<p class=\"pass-rate-text\">{:.1}% ({}/{})</p>\n",
            pass_rate, passed, total
        ));
        html.push_str(&format!(
            "<p class=\"duration\">Total duration: {}ms</p>\n",
            total_duration
        ));
        html.push_str("</div>\n");

        html.push_str("<div class=\"features\">\n");
        html.push_str("<h2>Scenarios</h2>\n");

        for scenario in &result.scenario_results {
            let status = scenario.status();
            let badge_class = Self::status_badge_class(status);
            let status_text = Self::status_text(status);

            html.push_str("<div class=\"scenario\">\n");
            html.push_str("<div class=\"scenario-header\">\n");
            html.push_str(&format!(
                "<span class=\"scenario-name\">{}</span>\n",
                self.escape_html(&scenario.scenario_name)
            ));
            html.push_str(&format!(
                "<span class=\"badge {}\">{}</span>\n",
                badge_class, status_text
            ));
            html.push_str(&format!(
                "<span class=\"duration\">{}ms</span>\n",
                scenario.duration_ms
            ));
            html.push_str("</div>\n");

            html.push_str("<div class=\"steps\">\n");
            for (id, step_result) in &scenario.step_results {
                html.push_str("<div class=\"step\">\n");
                html.push_str(&format!(
                    "<span class=\"step-id\">{}</span>\n",
                    id.to_string().chars().take(8).collect::<String>()
                ));
                html.push_str(&format!(
                    "<span class=\"{}\">{}</span>\n",
                    Self::step_badge_class(step_result),
                    Self::step_text(step_result)
                ));
                html.push_str("</div>\n");
            }
            html.push_str("</div>\n");
            html.push_str("</div>\n");
        }

        html.push_str("</div>\n");
        html.push_str("</div>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        html
    }

    fn escape_html(&self, s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    fn get_css(&self) -> &'static str {
        r#"
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: #f5f5f5;
            color: #333;
            line-height: 1.6;
        }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        header { background: #2c3e50; color: white; padding: 30px; border-radius: 8px; margin-bottom: 20px; }
        header h1 { font-size: 2em; margin-bottom: 10px; }
        .timestamp { color: #bdc3c7; font-size: 0.9em; }
        .summary { display: grid; grid-template-columns: repeat(4, 1fr); gap: 15px; margin-bottom: 20px; }
        .summary-card { background: white; padding: 20px; border-radius: 8px; text-align: center; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .summary-value { font-size: 2.5em; font-weight: bold; }
        .summary-label { color: #666; font-size: 0.9em; margin-top: 5px; }
        .summary-card.passed .summary-value { color: #27ae60; }
        .summary-card.failed .summary-value { color: #e74c3c; }
        .summary-card.skipped .summary-value { color: #f39c12; }
        .progress-section { background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .progress-section h2 { margin-bottom: 15px; font-size: 1.2em; }
        .progress-bar { background: #ecf0f1; height: 30px; border-radius: 15px; overflow: hidden; }
        .progress-fill { background: linear-gradient(90deg, #27ae60, #2ecc71); height: 100%; transition: width 0.3s ease; }
        .pass-rate-text { text-align: center; margin-top: 10px; font-weight: bold; color: #27ae60; }
        .duration { color: #7f8c8d; font-size: 0.9em; }
        .features { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .features h2 { margin-bottom: 15px; font-size: 1.2em; }
        .scenario { border: 1px solid #ecf0f1; border-radius: 8px; margin-bottom: 15px; overflow: hidden; }
        .scenario-header { display: flex; align-items: center; gap: 10px; padding: 15px; background: #fafafa; border-bottom: 1px solid #ecf0f1; }
        .scenario-name { flex: 1; font-weight: 600; font-size: 1.1em; }
        .badge { padding: 4px 12px; border-radius: 12px; font-size: 0.85em; font-weight: 600; }
        .badge-passed { background: #d5f4e6; color: #27ae60; }
        .badge-failed { background: #fadbd8; color: #e74c3c; }
        .badge-skipped { background: #fef5e7; color: #f39c12; }
        .badge-pending { background: #eaeaea; color: #7f8c8d; }
        .steps { padding: 15px; }
        .step { display: flex; align-items: center; gap: 10px; padding: 8px; border-radius: 4px; margin-bottom: 5px; }
        .step-id { font-family: monospace; font-size: 0.8em; color: #95a5a6; }
        .step-passed { color: #27ae60; }
        .step-failed { color: #e74c3c; background: #fadbd8; }
        .step-skipped { color: #f39c12; background: #fef5e7; }
        .step-pending { color: #7f8c8d; background: #eaeaea; }
        .step-ambiguous { color: #9b59b6; background: #f4ecf7; }
        "#
    }
}

impl Default for HtmlReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportWriterPort for HtmlReporter {
    fn write_report(&self, result: &FeatureResult) -> Result<String, BddError> {
        Ok(self.generate_html(result))
    }

    fn write_report_to_file(&self, result: &FeatureResult, path: &str) -> Result<(), BddError> {
        let html = self.generate_html(result);
        std::fs::write(path, html).map_err(|e| BddError::IoError(e.to_string()))
    }

    fn format(&self) -> &str {
        "html"
    }

    fn flush(&self) -> Result<(), BddError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Feature, Scenario, Step, StepType};
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_feature_result() -> FeatureResult {
        let scenario1 = ScenarioResult {
            scenario_id: Uuid::new_v4(),
            scenario_name: "Successful login".to_string(),
            step_results: vec![(Uuid::new_v4(), StepResult::Passed)],
            duration_ms: 150,
            started_at: Utc::now(),
            completed_at: Utc::now(),
        };
        let scenario2 = ScenarioResult {
            scenario_id: Uuid::new_v4(),
            scenario_name: "Failed login".to_string(),
            step_results: vec![(
                Uuid::new_v4(),
                StepResult::Failed {
                    error: "Invalid credentials".to_string(),
                    location: "auth.rs:42".to_string(),
                },
            )],
            duration_ms: 50,
            started_at: Utc::now(),
            completed_at: Utc::now(),
        };
        FeatureResult {
            feature_id: Uuid::new_v4(),
            feature_name: "User Authentication".to_string(),
            scenario_results: vec![scenario1, scenario2],
            status: ExecutionStatus::Passed,
            started_at: Utc::now(),
            completed_at: Utc::now(),
        }
    }

    #[test]
    fn test_html_reporter_generates_valid_html() {
        let result = create_test_feature_result();
        let reporter = HtmlReporter::new();
        let html = reporter.generate_html(&result);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("User Authentication"));
        assert!(html.contains("Passed"));
        assert!(html.contains("Failed"));
    }

    #[test]
    fn test_html_escape() {
        let mut result = create_test_feature_result();
        result.feature_name = "Test <script>alert('xss')</script>".to_string();
        let reporter = HtmlReporter::new();
        let html = reporter.generate_html(&result);

        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_format_returns_html() {
        let reporter = HtmlReporter::new();
        assert_eq!(reporter.format(), "html");
    }

    #[test]
    fn test_pass_rate_calculation() {
        let result = create_test_feature_result();
        let reporter = HtmlReporter::new();
        let html = reporter.generate_html(&result);

        assert!(html.contains("50.0%"));
    }

    #[test]
    fn test_empty_feature() {
        let result = FeatureResult {
            feature_id: Uuid::new_v4(),
            feature_name: "Empty".to_string(),
            scenario_results: vec![],
            status: ExecutionStatus::Passed,
            started_at: Utc::now(),
            completed_at: Utc::now(),
        };
        let reporter = HtmlReporter::new();
        let html = reporter.generate_html(&result);

        assert!(html.contains("0.0%"));
    }
}
