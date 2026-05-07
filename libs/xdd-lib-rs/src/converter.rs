//! Dialect converter

use serde_json::Value;

use super::dialect::{Dialect, DialectType};
use super::error::XddResult;

/// Convert between dialects
pub struct DialectConverter;

impl DialectConverter {
    /// Convert a string from one dialect to another
    pub fn convert(input: &str, from: &dyn Dialect, to: &dyn Dialect) -> XddResult<String> {
        let value = from.parse(input)?;
        to.serialize(&value)
    }

    /// Convert between dialect types using their dialects
    pub fn convert_types(input: &str, from: DialectType, to: DialectType) -> XddResult<String> {
        let from_dialect = dialect_for_type(from)?;
        let to_dialect = dialect_for_type(to)?;
        Self::convert(input, from_dialect.as_ref(), to_dialect.as_ref())
    }

    /// Convert to JSON Value
    pub fn to_json<D: Dialect>(input: &str, dialect: &D) -> XddResult<Value> {
        dialect.parse(input)
    }

    /// Convert from JSON Value to string
    pub fn from_json<D: Dialect>(value: &Value, dialect: &D) -> XddResult<String> {
        dialect.serialize(value)
    }

    /// Parse and re-serialize (useful for formatting)
    pub fn reformat<D: Dialect>(input: &str, dialect: &D) -> XddResult<String> {
        let value = dialect.parse(input)?;
        dialect.serialize(&value)
    }
}

fn dialect_for_type(dialect_type: DialectType) -> XddResult<Box<dyn Dialect>> {
    match dialect_type {
        DialectType::Json => Ok(Box::new(super::dialect::JsonDialect::new())),
        DialectType::Toml => Ok(Box::new(super::dialect::TomlDialect::new())),
        DialectType::Yaml => Ok(Box::new(super::dialect::YamlDialect::new())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::{JsonDialect, YamlDialect};

    #[test]
    fn json_to_toml() {
        let input = r#"{"name": "test", "count": 42}"#;
        let result =
            DialectConverter::convert_types(input, DialectType::Json, DialectType::Toml).unwrap();
        assert!(result.contains("name"));
        assert!(result.contains("test"));
    }

    #[test]
    fn json_to_yaml() {
        let input = r#"{"name": "test"}"#;
        let result =
            DialectConverter::convert_types(input, DialectType::Json, DialectType::Yaml).unwrap();
        assert!(result.contains("name:"));
    }

    #[test]
    fn reformat_json() {
        let input = r#"{"a":1,"b":2}"#;
        let result = DialectConverter::reformat(input, &JsonDialect::new()).unwrap();
        assert!(result.contains('\n')); // Pretty printed
    }

    #[test]
    fn invalid_input_fails() {
        let input = "not a json";
        let res = DialectConverter::convert_types(input, DialectType::Json, DialectType::Toml);
        assert!(res.is_err());
    }

    #[test]
    fn dialect_to_json_to_string_roundtrip() {
        let val = serde_json::json!({"test": 42});
        let dialect = YamlDialect::new();
        let s = DialectConverter::from_json(&val, &dialect).unwrap();
        let val2 = DialectConverter::to_json(&s, &dialect).unwrap();
        assert_eq!(val, val2);
    }
}
