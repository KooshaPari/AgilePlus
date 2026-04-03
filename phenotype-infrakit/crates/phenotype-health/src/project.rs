//! Project-Level Health Types for Unified Health Dashboard
//!
//! This module provides types for tracking project health across the Phenotype ecosystem.
//! It enables unified compliance monitoring, security posture tracking, and dependency
//! freshness checking across all projects.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Language stack for a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageStack {
    Rust,
    TypeScript,
    Python,
    Go,
    Multi,
}

impl std::fmt::Display for LanguageStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageStack::Rust => write!(f, "Rust"),
            LanguageStack::TypeScript => write!(f, "TypeScript"),
            LanguageStack::Python => write!(f, "Python"),
            LanguageStack::Go => write!(f, "Go"),
            LanguageStack::Multi => write!(f, "Multi"),
        }
    }
}

/// Health band classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthBand {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl HealthBand {
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 90.0 => HealthBand::Excellent,
            s if s >= 75.0 => HealthBand::Good,
            s if s >= 60.0 => HealthBand::Fair,
            s if s >= 40.0 => HealthBand::Poor,
            _ => HealthBand::Critical,
        }
    }
}

/// Health dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDimension {
    pub name: String,
    pub score: f32,
    pub weight: f32,
    pub findings: Vec<HealthFinding>,
}

impl HealthDimension {
    pub fn new(name: &str, score: f32, weight: f32) -> Self {
        Self {
            name: name.to_string(),
            score: score.clamp(0.0, 100.0),
            weight,
            findings: Vec::new(),
        }
    }

    pub fn weighted_score(&self) -> f32 {
        self.score * self.weight / 100.0
    }
}

/// Health finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthFinding {
    pub severity: FindingSeverity,
    pub category: FindingCategory,
    pub message: String,
}

impl HealthFinding {
    pub fn new(severity: FindingSeverity, category: FindingCategory, message: &str) -> Self {
        Self {
            severity,
            category,
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingCategory {
    Documentation,
    TestCoverage,
    Security,
    Dependencies,
    Compliance,
    CodeQuality,
}

/// Complete project health score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHealth {
    pub project_name: String,
    pub owner: String,
    pub language: LanguageStack,
    pub overall_score: f32,
    pub band: HealthBand,
    pub dimensions: HashMap<String, HealthDimension>,
    pub findings: Vec<HealthFinding>,
    pub last_scan: DateTime<Utc>,
}

impl ProjectHealth {
    pub fn new(project_name: &str, owner: &str, language: LanguageStack) -> Self {
        Self {
            project_name: project_name.to_string(),
            owner: owner.to_string(),
            language,
            overall_score: 0.0,
            band: HealthBand::Critical,
            dimensions: HashMap::new(),
            findings: Vec::new(),
            last_scan: Utc::now(),
        }
    }

    pub fn add_dimension(mut self, dimension: HealthDimension) -> Self {
        self.overall_score += dimension.weighted_score();
        self.dimensions.insert(dimension.name.clone(), dimension);
        self
    }

    pub fn finalize(mut self) -> Self {
        self.overall_score = self.overall_score.min(100.0);
        self.band = HealthBand::from_score(self.overall_score);
        self
    }

    pub fn is_compliant(&self, threshold: f32) -> bool {
        self.overall_score >= threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_band_from_score() {
        assert_eq!(HealthBand::from_score(95.0), HealthBand::Excellent);
        assert_eq!(HealthBand::from_score(85.0), HealthBand::Good);
        assert_eq!(HealthBand::from_score(70.0), HealthBand::Fair);
        assert_eq!(HealthBand::from_score(50.0), HealthBand::Poor);
        assert_eq!(HealthBand::from_score(30.0), HealthBand::Critical);
    }

    #[test]
    fn test_project_health_calculation() {
        let health = ProjectHealth::new("test", "owner", LanguageStack::Rust)
            .add_dimension(HealthDimension::new("documentation", 90.0, 15.0))
            .add_dimension(HealthDimension::new("test_coverage", 80.0, 20.0))
            .add_dimension(HealthDimension::new("security", 100.0, 25.0))
            .add_dimension(HealthDimension::new("dependencies", 75.0, 15.0))
            .add_dimension(HealthDimension::new("compliance", 90.0, 15.0))
            .add_dimension(HealthDimension::new("code_quality", 88.0, 10.0))
            .finalize();

        // Expected: 90*0.15 + 80*0.20 + 100*0.25 + 75*0.15 + 90*0.15 + 88*0.10 = 88.05
        assert!(health.overall_score > 87.0 && health.overall_score < 89.0, 
            "Expected score ~88.05, got {}", health.overall_score);
        assert_eq!(health.band, HealthBand::Good);
    }
}
