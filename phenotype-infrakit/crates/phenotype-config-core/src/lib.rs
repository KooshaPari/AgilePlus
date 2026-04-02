//! Core configuration types

pub mod error;

pub use error::{ConfigError, ConfigResult};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
    Null,
}

impl ConfigValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn from_json(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::String(s) => ConfigValue::String(s),
            serde_json::Value::Number(n) => ConfigValue::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::Bool(b) => ConfigValue::Boolean(b),
            serde_json::Value::Array(a) => {
                ConfigValue::Array(a.into_iter().map(ConfigValue::from_json).collect())
            }
            serde_json::Value::Object(o) => {
                let mut map = HashMap::new();
                for (k, v) in o {
                    map.insert(k, ConfigValue::from_json(v));
                }
                ConfigValue::Object(map)
            }
            serde_json::Value::Null => ConfigValue::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_value_as_str() {
        let val = ConfigValue::String("test".to_string());
        assert_eq!(val.as_str(), Some("test"));

        let val = ConfigValue::Number(42.0);
        assert_eq!(val.as_str(), None);
    }

    #[test]
    fn test_from_json() {
        let json = serde_json::json!({
            "key": "value",
            "num": 42,
            "bool": true,
            "list": ["a", 1],
            "nested": { "a": "b" }
        });

        let val = ConfigValue::from_json(json);
        match val {
            ConfigValue::Object(m) => {
                assert_eq!(m.get("key").unwrap().as_str(), Some("value"));
                assert!(matches!(m.get("num").unwrap(), ConfigValue::Number(_)));
                assert!(matches!(m.get("bool").unwrap(), ConfigValue::Boolean(true)));
                assert!(matches!(m.get("list").unwrap(), ConfigValue::Array(_)));
                assert!(matches!(m.get("nested").unwrap(), ConfigValue::Object(_)));
            }
            _ => panic!("Expected Object"),
        }
    }
}
