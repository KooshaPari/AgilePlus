use phenotype_error_core::ConfigError as Error;
type Result<T> = std::result::Result<T, Error>;

use phenotype_health::{HealthChecker, HealthStatus};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::future::Future;
use std::pin::Pin;
use async_trait::async_trait;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Finding {
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub file: Option<PathBuf>,
}

#[async_trait]
pub trait ComplianceRule: Send + Sync {
    fn id(&self) -> &str;
    async fn check(&self, root: &Path) -> Result<Vec<Finding>>;
}

pub struct ComplianceScanner {
    root: PathBuf,
    rules: Vec<Box<dyn ComplianceRule>>,
}

impl ComplianceScanner {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Box<dyn ComplianceRule>) {
        self.rules.push(rule);
    }

    pub async fn scan(&self) -> Result<Vec<Finding>> {
        let mut all_findings = Vec::new();
        for rule in &self.rules {
            let findings = rule.check(&self.root).await?;
            all_findings.extend(findings);
        }
        Ok(all_findings)
    }
}

impl HealthChecker for ComplianceScanner {
    fn name(&self) -> &str {
        "compliance-scanner"
    }

    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async move {
            match self.scan().await {
                Ok(findings) => {
                    if findings.is_empty() {
                        HealthStatus::Healthy
                    } else {
                        HealthStatus::Degraded
                    }
                }
                Err(_) => HealthStatus::Unhealthy,
            }
        })
    }
}

pub struct LicenseFileRule;

#[async_trait]
impl ComplianceRule for LicenseFileRule {
    fn id(&self) -> &str {
        "missing-license"
    }

    async fn check(&self, root: &Path) -> Result<Vec<Finding>> {
        let license_path = root.join("LICENSE");
        let license_md_path = root.join("LICENSE.md");

        if !license_path.exists() && !license_md_path.exists() {
            Ok(vec![Finding {
                rule_id: self.id().to_string(),
                severity: "High".to_string(),
                message: "Missing LICENSE file".to_string(),
                file: None,
            }])
        } else {
            Ok(vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_license_rule() {
        let dir = tempdir().unwrap();
        let rule = LicenseFileRule;
        
        let findings = rule.check(dir.path()).await.unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "missing-license");

        std::fs::write(dir.path().join("LICENSE"), "MIT").unwrap();
        let findings = rule.check(dir.path()).await.unwrap();
        assert_eq!(findings.len(), 0);
    }
}
