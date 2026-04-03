//! Gherkin-lite parser for .feature files

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepKind {
    Given,
    When,
    Then,
    And,
    But,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub kind: StepKind,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone)]
pub struct Feature {
    pub name: String,
    pub scenarios: Vec<Scenario>,
}

impl Feature {
    pub fn parse(input: &str) -> crate::Result<Self> {
        let mut name = String::new();
        let mut scenarios = Vec::new();
        let mut current_scenario: Option<Scenario> = None;

        for line in input.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(rest) = line.strip_prefix("Feature:") {
                name = rest.trim().to_string();
            } else if let Some(rest) = line.strip_prefix("Scenario:") {
                if let Some(s) = current_scenario.take() {
                    scenarios.push(s);
                }
                current_scenario = Some(Scenario {
                    name: rest.trim().to_string(),
                    steps: Vec::new(),
                });
            } else if let Some(s) = current_scenario.as_mut() {
                if let Some(rest) = line.strip_prefix("Given") {
                    s.steps.push(Step { kind: StepKind::Given, text: rest.trim().to_string() });
                } else if let Some(rest) = line.strip_prefix("When") {
                    s.steps.push(Step { kind: StepKind::When, text: rest.trim().to_string() });
                } else if let Some(rest) = line.strip_prefix("Then") {
                    s.steps.push(Step { kind: StepKind::Then, text: rest.trim().to_string() });
                } else if let Some(rest) = line.strip_prefix("And") {
                    s.steps.push(Step { kind: StepKind::And, text: rest.trim().to_string() });
                } else if let Some(rest) = line.strip_prefix("But") {
                    s.steps.push(Step { kind: StepKind::But, text: rest.trim().to_string() });
                }
            }
        }

        if let Some(s) = current_scenario {
            scenarios.push(s);
        }

        Ok(Feature { name, scenarios })
    }
}
