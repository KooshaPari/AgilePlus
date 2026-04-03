//! Gherkin parser

pub struct Parser;

impl Parser {
    pub fn parse_feature(content: &str) -> Result<Feature, BddError> {
        let mut scenarios = Vec::new();
        let mut steps = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Scenario:") {
                scenarios.push(Scenario::new(trimmed.trim_start_matches("Scenario:").trim().to_string()));
            } else if trimmed.starts_with("Given ") || trimmed.starts_with("When ") || trimmed.starts_with("Then ") || trimmed.starts_with("And ") {
                steps.push(Step::new(trimmed.to_string()));
            }
        }
        
        Ok(Feature { scenarios, steps })
    }
}

pub struct Feature {
    pub scenarios: Vec<Scenario>,
    pub steps: Vec<Step>,
}

pub struct Scenario {
    pub name: String,
}

impl Scenario {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

pub struct Step {
    pub text: String,
}

impl Step {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

use crate::error::BddError;
