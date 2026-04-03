//! BDD testing framework

#![allow(missing_docs)]

use std::collections::HashMap;

/// Step argument types
#[derive(Debug, Clone)]
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
#[derive(Debug)]
pub struct StepContext {
    pub args: Vec<StepArg>,
    pub table: Option<Vec<Vec<String>>>,
}

impl StepContext {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            table: None,
        }
    }
    
    pub fn add_arg(&mut self, arg: StepArg) {
        self.args.push(arg);
    }
}

/// Feature result
#[derive(Debug)]
pub struct FeatureResult {
    pub passed: bool,
    pub scenarios: Vec<ScenarioResult>,
}

impl FeatureResult {
    pub fn new() -> Self {
        Self {
            passed: true,
            scenarios: Vec::new(),
        }
    }
}

/// Scenario result
#[derive(Debug)]
pub struct ScenarioResult {
    pub passed: bool,
    pub steps: Vec<StepResult>,
}

impl ScenarioResult {
    pub fn new() -> Self {
        Self {
            passed: true,
            steps: Vec::new(),
        }
    }
}

/// Step result
#[derive(Debug)]
pub struct StepResult {
    pub passed: bool,
    pub error: Option<String>,
}

impl StepResult {
    pub fn new() -> Self {
        Self {
            passed: true,
            error: None,
        }
    }
    
    pub fn failed(mut self, error: String) -> Self {
        self.passed = false;
        self.error = Some(error);
        self
    }
}

/// BDD Runner
pub struct BddRunner {
    steps: HashMap<String, Box<dyn Fn(&StepContext) + Send + Sync>>,
}

impl BddRunner {
    pub fn new() -> Self {
        Self {
            steps: HashMap::new(),
        }
    }
    
    pub fn register_step<F>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(&StepContext) + Send + Sync + 'static,
    {
        self.steps.insert(pattern.to_string(), Box::new(handler));
    }
    
    pub fn run_feature(&self, feature: &str) -> FeatureResult {
        FeatureResult::new()
    }
}

impl Default for BddRunner {
    fn default() -> Self {
        Self::new()
    }
}
