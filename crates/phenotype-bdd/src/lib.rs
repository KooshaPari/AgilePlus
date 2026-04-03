//! BDD testing framework

pub mod error;
pub mod parser;

use std::collections::HashMap;

pub use error::{BddError, Result};
pub use parser::{Feature, Scenario, Step, StepKind};

/// Step argument types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepArg {
    String(String),
    Table(Vec<Vec<String>>),
    DocString(String),
}

impl StepArg {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            StepArg::String(ref s) => Some(s),
            _ => None,
        }
    }
}

/// Step context
#[derive(Debug, Default)]
pub struct StepContext {
    pub args: Vec<StepArg>,
    pub table: Option<Vec<Vec<String>>>,
}

impl StepContext {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_arg(&mut self, arg: StepArg) {
        self.args.push(arg);
    }
}

/// Step result
#[derive(Debug, Clone)]
pub struct StepResult {
    pub passed: bool,
    pub error: Option<String>,
}

impl StepResult {
    pub fn passed() -> Self {
        Self {
            passed: true,
            error: None,
        }
    }
    
    pub fn failed(error: impl Into<String>) -> Self {
        Self {
            passed: false,
            error: Some(error.into()),
        }
    }
}

/// Scenario result
#[derive(Debug, Clone)]
pub struct ScenarioResult {
    pub name: String,
    pub passed: bool,
    pub steps: Vec<StepResult>,
}

/// Feature result
#[derive(Debug, Clone)]
pub struct FeatureResult {
    pub name: String,
    pub passed: bool,
    pub scenarios: Vec<ScenarioResult>,
}

/// BDD Runner
pub struct BddRunner {
    steps: HashMap<String, Box<dyn Fn(&StepContext) -> StepResult + Send + Sync>>,
}

impl BddRunner {
    pub fn new() -> Self {
        Self {
            steps: HashMap::new(),
        }
    }
    
    pub fn register_step<F>(&mut self, pattern: impl Into<String>, handler: F)
    where
        F: Fn(&StepContext) -> StepResult + Send + Sync + 'static,
    {
        self.steps.insert(pattern.into(), Box::new(handler));
    }
    
    pub fn run_feature(&self, feature_text: &str) -> Result<FeatureResult> {
        let feature = Feature::parse(feature_text)?;
        let mut scenarios = Vec::new();
        let mut feature_passed = true;

        for scenario in feature.scenarios {
            let mut steps = Vec::new();
            let mut scenario_passed = true;

            for step in scenario.steps {
                let result = if let Some(handler) = self.steps.get(&step.text) {
                    handler(&StepContext::new())
                } else {
                    StepResult::failed(format!("Step not found: {}", step.text))
                };

                if !result.passed {
                    scenario_passed = false;
                }
                steps.push(result);
            }

            if !scenario_passed {
                feature_passed = false;
            }
            scenarios.push(ScenarioResult {
                name: scenario.name,
                passed: scenario_passed,
                steps,
            });
        }

        Ok(FeatureResult {
            name: feature.name,
            passed: feature_passed,
            scenarios,
        })
    }
}

impl Default for BddRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bdd_runner_success() {
        let mut runner = BddRunner::new();
        runner.register_step("a valid config", |_| StepResult::passed());
        runner.register_step("I load it", |_| StepResult::passed());
        runner.register_step("it should be valid", |_| StepResult::passed());

        let feature = r#"
Feature: Config Loading
  Scenario: Load it
    Given a valid config
    When I load it
    Then it should be valid
"#;
        let result = runner.run_feature(feature).unwrap();
        assert!(result.passed);
        assert_eq!(result.scenarios.len(), 1);
        assert!(result.scenarios[0].passed);
    }
}
