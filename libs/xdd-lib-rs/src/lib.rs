// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cross-dialect data conversion library for JSON, TOML, and YAML.
//!
//! Provides a uniform [`Dialect`] trait and implementations for the three
//! most common spec / config file formats used in AgilePlus.  The
//! [`DialectConverter`] and [`DialectRegistry`] allow format-agnostic loading
//! and round-trip conversion.
//!
//! # Quick start
//!
//! ```
//! use xdd_lib_rs::{DialectRegistry, JsonDialect, TomlDialect, YamlDialect};
//!
//! let reg = DialectRegistry::default();
//! let json = reg.get_by_name("json").unwrap();
//! let value = json.parse(r#"{"status": "ok"}"#).unwrap();
//! ```

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during dialect parsing or serialization.
#[derive(Debug, Error)]
pub enum DialectError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("TOML error: {0}")]
    Toml(String),
    #[error("YAML error: {0}")]
    Yaml(String),
    #[error("Unknown dialect: {0}")]
    UnknownDialect(String),
    #[error("No dialect registered for extension: {0}")]
    UnknownExtension(String),
    #[error("Conversion error: {0}")]
    Conversion(String),
}

impl From<toml::de::Error> for DialectError {
    fn from(e: toml::de::Error) -> Self {
        DialectError::Toml(e.to_string())
    }
}

impl From<toml::ser::Error> for DialectError {
    fn from(e: toml::ser::Error) -> Self {
        DialectError::Toml(e.to_string())
    }
}

impl From<serde_yaml::Error> for DialectError {
    fn from(e: serde_yaml::Error) -> Self {
        DialectError::Yaml(e.to_string())
    }
}

/// A single data format (dialect) that can be parsed into and serialized from
/// a generic JSON value tree.
pub trait Dialect: Send + Sync {
    /// Human-readable name, e.g. `"json"`.
    fn name(&self) -> &str;
    /// File extensions this dialect is commonly associated with.
    fn file_extensions(&self) -> &[&str];
    /// Parse raw text into a [`serde_json::Value`].
    fn parse(&self, input: &str) -> Result<Value, DialectError>;
    /// Serialize a [`serde_json::Value`] into this dialect's text format.
    fn serialize(&self, value: &Value) -> Result<String, DialectError>;
}

/// JSON dialect implementation.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonDialect;

impl Dialect for JsonDialect {
    fn name(&self) -> &str {
        "json"
    }
    fn file_extensions(&self) -> &[&str] {
        &["json"]
    }
    fn parse(&self, input: &str) -> Result<Value, DialectError> {
        Ok(serde_json::from_str(input)?)
    }
    fn serialize(&self, value: &Value) -> Result<String, DialectError> {
        Ok(serde_json::to_string_pretty(value)?)
    }
}

/// TOML dialect implementation.
#[derive(Debug, Clone, Copy, Default)]
pub struct TomlDialect;

impl Dialect for TomlDialect {
    fn name(&self) -> &str {
        "toml"
    }
    fn file_extensions(&self) -> &[&str] {
        &["toml"]
    }
    fn parse(&self, input: &str) -> Result<Value, DialectError> {
        let v: Value = toml::from_str(input)?;
        Ok(v)
    }
    fn serialize(&self, value: &Value) -> Result<String, DialectError> {
        Ok(toml::to_string_pretty(value)?)
    }
}

/// YAML dialect implementation.
#[derive(Debug, Clone, Copy, Default)]
pub struct YamlDialect;

impl Dialect for YamlDialect {
    fn name(&self) -> &str {
        "yaml"
    }
    fn file_extensions(&self) -> &[&str] {
        &["yaml", "yml"]
    }
    fn parse(&self, input: &str) -> Result<Value, DialectError> {
        Ok(serde_yaml::from_str(input)?)
    }
    fn serialize(&self, value: &Value) -> Result<String, DialectError> {
        Ok(serde_yaml::to_string(value)?)
    }
}

/// Converts data between any two registered dialects.
#[derive(Debug, Default)]
pub struct DialectConverter;

impl DialectConverter {
    pub fn new() -> Self {
        Self
    }

    /// Convert raw text from `from` dialect into `to` dialect.
    pub fn convert(
        &self,
        input: &str,
        from: &dyn Dialect,
        to: &dyn Dialect,
    ) -> Result<String, DialectError> {
        let value = from.parse(input)?;
        to.serialize(&value)
    }

    /// Convenience: convert by name (uses a [`DialectRegistry`] internally).
    pub fn convert_by_name(
        &self,
        input: &str,
        from_name: &str,
        to_name: &str,
        registry: &DialectRegistry,
    ) -> Result<String, DialectError> {
        let from = registry
            .get_by_name(from_name)
            .ok_or_else(|| DialectError::UnknownDialect(from_name.to_string()))?;
        let to = registry
            .get_by_name(to_name)
            .ok_or_else(|| DialectError::UnknownDialect(to_name.to_string()))?;
        self.convert(input, from.as_ref(), to.as_ref())
    }
}

/// Registry of known dialects, indexed by name and file extension.
#[derive(Default)]
pub struct DialectRegistry {
    by_name: HashMap<String, Arc<dyn Dialect>>,
    by_ext: HashMap<String, Arc<dyn Dialect>>,
}

impl std::fmt::Debug for DialectRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut names: Vec<&str> = self.by_name.keys().map(|s| s.as_str()).collect();
        names.sort();
        f.debug_struct("DialectRegistry")
            .field("dialects", &names)
            .finish()
    }
}

impl DialectRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a dialect.  Overwrites any previous entry with the same name
    /// or extensions.
    pub fn register<D: Dialect + 'static>(&mut self, dialect: D) {
        let name = dialect.name().to_string();
        let arc: Arc<dyn Dialect> = Arc::new(dialect);
        for ext in arc.file_extensions() {
            self.by_ext.insert(ext.to_string(), arc.clone());
        }
        self.by_name.insert(name, arc);
    }

    /// Look up a dialect by its canonical name.
    pub fn get_by_name(&self, name: &str) -> Option<Arc<dyn Dialect>> {
        self.by_name.get(name).cloned()
    }

    /// Look up a dialect by file extension (e.g. `"json"`).
    pub fn get_by_extension(&self, ext: &str) -> Option<Arc<dyn Dialect>> {
        self.by_ext.get(ext).cloned()
    }

    /// Returns an iterator over all registered dialect names.
    pub fn dialect_names(&self) -> impl Iterator<Item = &str> {
        self.by_name.keys().map(|s| s.as_str())
    }
}

impl DialectRegistry {
    /// Convenience constructor pre-populated with the three built-in dialects.
    pub fn default() -> Self {
        let mut reg = Self {
            by_name: HashMap::new(),
            by_ext: HashMap::new(),
        };
        reg.register(JsonDialect);
        reg.register(TomlDialect);
        reg.register(YamlDialect);
        reg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── Smoke tests for each dialect ───────────────────────────────────────

    #[test]
    fn json_dialect_name_and_extensions() {
        let d = JsonDialect;
        assert_eq!(d.name(), "json");
        assert_eq!(d.file_extensions(), &["json"]);
    }

    #[test]
    fn toml_dialect_name_and_extensions() {
        let d = TomlDialect;
        assert_eq!(d.name(), "toml");
        assert_eq!(d.file_extensions(), &["toml"]);
    }

    #[test]
    fn yaml_dialect_name_and_extensions() {
        let d = YamlDialect;
        assert_eq!(d.name(), "yaml");
        assert_eq!(d.file_extensions(), &["yaml", "yml"]);
    }

    // ── Round-trip tests ─────────────────────────────────────────────────

    #[test]
    fn json_roundtrip_simple_object() {
        let d = JsonDialect;
        let value = json!({"name": "AgilePlus", "version": 1});
        let serialized = d.serialize(&value).unwrap();
        let parsed = d.parse(&serialized).unwrap();
        assert_eq!(parsed, value);
    }

    #[test]
    fn toml_roundtrip_simple_object() {
        let d = TomlDialect;
        let value = json!({"name": "AgilePlus", "version": 1});
        let serialized = d.serialize(&value).unwrap();
        let parsed = d.parse(&serialized).unwrap();
        assert_eq!(parsed, value);
    }

    #[test]
    fn yaml_roundtrip_simple_object() {
        let d = YamlDialect;
        let value = json!({"name": "AgilePlus", "version": 1});
        let serialized = d.serialize(&value).unwrap();
        let parsed = d.parse(&serialized).unwrap();
        assert_eq!(parsed, value);
    }

    // ── AgilePlus trace record (JSON ↔ YAML ↔ TOML) ───────────────────────

    fn agileplus_trace_json() -> &'static str {
        r##"{
  "schema_version": "1",
  "fr_id": "FR-024-1",
  "spec_slug": "eco-024-traceability",
  "spec_anchor": "#fr-1",
  "docs_pages": ["AgilePlus/docs/traceability.md"],
  "tests": ["tooling/trace-validator/tests/spec.rs::test_fr1_trace_required"],
  "code_modules": ["tooling/trace-validator/src/main.rs"],
  "journeys": ["docs/operations/journeys/FR-024-1.md"],
  "status": "proposed",
  "last_validated": "2026-06-05T00:00:00Z"
}"##
    }

    #[test]
    fn json_to_yaml_agileplus_trace() {
        let reg = DialectRegistry::default();
        let conv = DialectConverter::new();
        let yaml = conv
            .convert_by_name(
                agileplus_trace_json(),
                "json",
                "yaml",
                &reg,
            )
            .unwrap();
        assert!(yaml.contains("schema_version:"));
        assert!(yaml.contains("FR-024-1"));
        assert!(yaml.contains("last_validated:"));

        // Verify round-trip back to JSON
        let json_back = conv
            .convert_by_name(&yaml, "yaml", "json", &reg)
            .unwrap();
        let v1: Value = serde_json::from_str(agileplus_trace_json()).unwrap();
        let v2: Value = serde_json::from_str(&json_back).unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn json_to_toml_agileplus_trace() {
        let reg = DialectRegistry::default();
        let conv = DialectConverter::new();
        let toml = conv
            .convert_by_name(
                agileplus_trace_json(),
                "json",
                "toml",
                &reg,
            )
            .unwrap();
        assert!(toml.contains("schema_version ="));
        assert!(toml.contains("fr_id ="));
        assert!(toml.contains("FR-024-1"));

        // Verify round-trip back to JSON
        let json_back = conv
            .convert_by_name(&toml, "toml", "json", &reg)
            .unwrap();
        let v1: Value = serde_json::from_str(agileplus_trace_json()).unwrap();
        let v2: Value = serde_json::from_str(&json_back).unwrap();
        assert_eq!(v1, v2);
    }

    // ── AgilePlus worklog record (YAML ↔ JSON) ─────────────────────────────

    fn agileplus_worklog_yaml() -> &'static str {
        r#"worklog_id: "L1-001-2026-06-11"
project: "AgilePlus"
status: "completed"
tasks:
  - id: "T-001"
    description: "Define core domain models"
    owner: "domain-team"
  - id: "T-002"
    description: "Add snapshot aggregate tests"
    owner: "domain-team"
metadata:
  schema_version: "1"
  last_updated: "2026-06-11T12:00:00Z"
"#
    }

    #[test]
    fn yaml_to_json_agileplus_worklog() {
        let reg = DialectRegistry::default();
        let conv = DialectConverter::new();
        let json = conv
            .convert_by_name(
                agileplus_worklog_yaml(),
                "yaml",
                "json",
                &reg,
            )
            .unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["worklog_id"], "L1-001-2026-06-11");
        assert_eq!(parsed["project"], "AgilePlus");
        assert_eq!(parsed["status"], "completed");
        assert_eq!(parsed["tasks"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["metadata"]["schema_version"], "1");
    }

    #[test]
    fn yaml_to_toml_agileplus_worklog() {
        let reg = DialectRegistry::default();
        let conv = DialectConverter::new();
        let toml = conv
            .convert_by_name(
                agileplus_worklog_yaml(),
                "yaml",
                "toml",
                &reg,
            )
            .unwrap();
        assert!(toml.contains("worklog_id ="));
        assert!(toml.contains("L1-001-2026-06-11"));
        assert!(toml.contains("[[tasks]]"));
    }

    // ── AgilePlus spec config (TOML ↔ JSON) ──────────────────────────────

    fn agileplus_spec_config_toml() -> &'static str {
        r#"[spec]
id = "FR-CORE-001"
title = "Domain Models"
status = "accepted"

[spec.criteria]
ac1 = "Preserve entity identity"
ac2 = "Initialize safe defaults"

[spec.traceability]
code = "crates/agileplus-domain/src/domain/event.rs"
spec_file = "specs/001-agileplus-core/FR-CORE-001-DOMAIN-MODELS.md"
"#
    }

    #[test]
    fn toml_to_json_agileplus_spec_config() {
        let reg = DialectRegistry::default();
        let conv = DialectConverter::new();
        let json = conv
            .convert_by_name(
                agileplus_spec_config_toml(),
                "toml",
                "json",
                &reg,
            )
            .unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["spec"]["id"], "FR-CORE-001");
        assert_eq!(parsed["spec"]["title"], "Domain Models");
        assert_eq!(parsed["spec"]["status"], "accepted");
        assert_eq!(parsed["spec"]["criteria"]["ac1"], "Preserve entity identity");
    }

    // ── Registry tests ─────────────────────────────────────────────────────

    #[test]
    fn registry_default_has_all_dialects() {
        let reg = DialectRegistry::default();
        let mut names: Vec<&str> = reg.dialect_names().collect();
        names.sort();
        assert_eq!(names, vec!["json", "toml", "yaml"]);
    }

    #[test]
    fn registry_lookup_by_extension() {
        let reg = DialectRegistry::default();
        assert!(reg.get_by_extension("json").is_some());
        assert!(reg.get_by_extension("toml").is_some());
        assert!(reg.get_by_extension("yaml").is_some());
        assert!(reg.get_by_extension("yml").is_some());
        assert!(reg.get_by_extension("xml").is_none());
    }

    #[test]
    fn registry_lookup_by_name() {
        let reg = DialectRegistry::default();
        assert!(reg.get_by_name("json").is_some());
        assert!(reg.get_by_name("toml").is_some());
        assert!(reg.get_by_name("yaml").is_some());
        assert!(reg.get_by_name("xml").is_none());
    }

    // ── DialectConverter direct API tests ──────────────────────────────────

    #[test]
    fn converter_direct_json_to_yaml() {
        let conv = DialectConverter::new();
        let json = JsonDialect;
        let yaml = YamlDialect;
        let input = r#"{"foo": "bar"}"#;
        let out = conv.convert(input, &json, &yaml).unwrap();
        assert!(out.contains("foo:"));
    }

    #[test]
    fn converter_unknown_dialect_errors() {
        let reg = DialectRegistry::default();
        let conv = DialectConverter::new();
        let err = conv
            .convert_by_name("", "xml", "json", &reg)
            .unwrap_err();
        assert!(matches!(err, DialectError::UnknownDialect(_)));
        assert!(err.to_string().contains("xml"));
    }

    #[test]
    fn json_parse_error_returns_dialect_error() {
        let json = JsonDialect;
        let err = json.parse("not json").unwrap_err();
        assert!(matches!(err, DialectError::Json(_)));
    }

    #[test]
    fn toml_parse_error_returns_dialect_error() {
        let toml = TomlDialect;
        let err = toml.parse("not toml [[").unwrap_err();
        assert!(matches!(err, DialectError::Toml(_)));
    }

    #[test]
    fn yaml_parse_error_returns_dialect_error() {
        let yaml = YamlDialect;
        let err = yaml.parse("\t\t: bad").unwrap_err();
        assert!(matches!(err, DialectError::Yaml(_)));
    }

    // ── Edge case: nested structures ───────────────────────────────────────

    #[test]
    fn nested_structures_roundtrip_all_dialects() {
        let value = json!({
            "project": {
                "name": "AgilePlus",
                "crates": [
                    {"name": "agileplus-domain", "path": "crates/agileplus-domain"},
                    {"name": "agileplus-api", "path": "crates/agileplus-api"},
                ]
            },
            "workspace": {
                "resolver": "3",
                "members": ["crates/*", "libs/*"]
            }
        });

        for dialect in [&JsonDialect as &dyn Dialect, &TomlDialect, &YamlDialect] {
            let serialized = dialect.serialize(&value).unwrap();
            let parsed = dialect.parse(&serialized).unwrap();
            assert_eq!(
                parsed, value,
                "{} round-trip failed",
                dialect.name()
            );
        }
    }

    // ── Edge case: empty array / object ───────────────────────────────────

    #[test]
    fn empty_array_and_object_roundtrip() {
        let value = json!({"empty_array": [], "empty_object": {}});
        for dialect in [&JsonDialect as &dyn Dialect, &TomlDialect, &YamlDialect] {
            let serialized = dialect.serialize(&value).unwrap();
            let parsed = dialect.parse(&serialized).unwrap();
            assert_eq!(parsed, value, "{} round-trip failed", dialect.name());
        }
    }

    // ── Edge case: null handling ──────────────────────────────────────────

    #[test]
    fn null_value_json() {
        let json = JsonDialect;
        let value = json!(null);
        let serialized = json.serialize(&value).unwrap();
        assert_eq!(serialized, "null");
        let parsed = json.parse(&serialized).unwrap();
        assert_eq!(parsed, Value::Null);
    }

    #[test]
    fn null_value_yaml() {
        let yaml = YamlDialect;
        let value = json!(null);
        let serialized = yaml.serialize(&value).unwrap();
        let parsed = yaml.parse(&serialized).unwrap();
        assert_eq!(parsed, Value::Null);
    }
}
