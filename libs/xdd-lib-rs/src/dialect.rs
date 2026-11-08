//! Dialect trait and types

use serde_json::Value;

use super::error::{XddError, XddResult};

/// Supported dialect types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialectType {
    Json,
    Toml,
    Yaml,
}

impl DialectType {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            "yaml" | "yml" => Some(Self::Yaml),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Yaml => "yaml",
        }
    }
}

/// Dialect trait for format-specific parsing and serialization
pub trait Dialect: Send + Sync {
    /// Get the dialect type
    fn dialect_type(&self) -> DialectType;

    /// Parse a string into a generic JSON Value
    fn parse(&self, input: &str) -> XddResult<Value>;

    /// Serialize a Value back to string
    fn serialize(&self, value: &Value) -> XddResult<String>;
}

/// JSON dialect implementation
#[derive(Debug, Clone, Default)]
pub struct JsonDialect;

impl JsonDialect {
    pub fn new() -> Self {
        Self
    }
}

impl Dialect for JsonDialect {
    fn dialect_type(&self) -> DialectType {
        DialectType::Json
    }

    fn parse(&self, input: &str) -> XddResult<Value> {
        serde_json::from_str(input).map_err(|e| XddError::ParseError(e.to_string()))
    }

    fn serialize(&self, value: &Value) -> XddResult<String> {
        serde_json::to_string_pretty(value).map_err(|e| XddError::SerializationError(e.to_string()))
    }
}

/// TOML dialect implementation
#[derive(Debug, Clone, Default)]
pub struct TomlDialect;

impl TomlDialect {
    pub fn new() -> Self {
        Self
    }
}

impl Dialect for TomlDialect {
    fn dialect_type(&self) -> DialectType {
        DialectType::Toml
    }

    fn parse(&self, input: &str) -> XddResult<Value> {
        let value: toml::Value =
            toml::from_str(input).map_err(|e| XddError::ParseError(e.to_string()))?;
        Ok(toml_to_json(value))
    }

    fn serialize(&self, value: &Value) -> XddResult<String> {
        let toml_value = json_to_toml(value)?;
        toml::to_string_pretty(&toml_value).map_err(|e| XddError::SerializationError(e.to_string()))
    }
}

/// YAML dialect implementation
#[derive(Debug, Clone, Default)]
pub struct YamlDialect;

impl YamlDialect {
    pub fn new() -> Self {
        Self
    }
}

impl Dialect for YamlDialect {
    fn dialect_type(&self) -> DialectType {
        DialectType::Yaml
    }

    fn parse(&self, input: &str) -> XddResult<Value> {
        serde_yaml::from_str(input).map_err(|e| XddError::ParseError(e.to_string()))
    }

    fn serialize(&self, value: &Value) -> XddResult<String> {
        serde_yaml::to_string(value).map_err(|e| XddError::SerializationError(e.to_string()))
    }
}

// Helper: TOML to JSON conversion
fn toml_to_json(toml: toml::Value) -> Value {
    match toml {
        toml::Value::String(s) => Value::String(s),
        toml::Value::Integer(i) => Value::Number(i.into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        toml::Value::Boolean(b) => Value::Bool(b),
        toml::Value::Datetime(dt) => Value::String(dt.to_string()),
        toml::Value::Array(arr) => Value::Array(arr.into_iter().map(toml_to_json).collect()),
        toml::Value::Table(table) => Value::Object(
            table
                .into_iter()
                .map(|(k, v)| (k, toml_to_json(v)))
                .collect(),
        ),
    }
}

// Helper: JSON to TOML conversion
fn json_to_toml(json: &Value) -> XddResult<toml::Value> {
    match json {
        Value::Null => Ok(toml::Value::String("null".to_string())),
        Value::Bool(b) => Ok(toml::Value::Boolean(*b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                Ok(toml::Value::Integer(0))
            }
        }
        Value::String(s) => Ok(toml::Value::String(s.clone())),
        Value::Array(arr) => Ok(toml::Value::Array(
            arr.iter()
                .map(json_to_toml)
                .collect::<XddResult<Vec<_>>>()?,
        )),
        Value::Object(obj) => {
            let mut table = toml::map::Map::new();
            for (k, v) in obj {
                table.insert(k.clone(), json_to_toml(v)?);
            }
            Ok(toml::Value::Table(table))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_dialect_parse() {
        let dialect = JsonDialect::new();
        let value = dialect.parse(r#"{"key": "value"}"#).unwrap();
        assert_eq!(value["key"], "value");
    }

    #[test]
    fn json_dialect_roundtrip() {
        let dialect = JsonDialect::new();
        let original = r#"{"name": "test", "count": 42}"#;
        let parsed = dialect.parse(original).unwrap();
        let serialized = dialect.serialize(&parsed).unwrap();
        assert!(serialized.contains("test"));
    }

    #[test]
    fn toml_dialect_parse() {
        let dialect = TomlDialect::new();
        let value = dialect
            .parse(
                r#"name = "test"
count = 42"#,
            )
            .unwrap();
        assert_eq!(value["name"], "test");
        assert_eq!(value["count"], 42);
    }

    #[test]
    fn yaml_dialect_parse() {
        let dialect = YamlDialect::new();
        let value = dialect.parse("name: test\ncount: 42").unwrap();
        assert_eq!(value["name"], "test");
    }
}
