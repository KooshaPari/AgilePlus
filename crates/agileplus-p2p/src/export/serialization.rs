use std::collections::BTreeMap;

use serde_json::Value;

pub(crate) fn to_sorted(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> = map
                .into_iter()
                .map(|(key, value)| (key, to_sorted(value)))
                .collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(to_sorted).collect()),
        other => other,
    }
}

pub(crate) fn to_sorted_pretty(value: Value) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&to_sorted(value))
}

pub(crate) fn to_sorted_line(value: Value) -> Result<String, serde_json::Error> {
    serde_json::to_string(&to_sorted(value))
}

#[cfg(test)]
mod deep_tests {
    use super::*;

    #[test]
    fn to_sorted_preserves_scalars() {
        assert_eq!(to_sorted(serde_json::json!(1)), serde_json::json!(1));
        assert_eq!(to_sorted(serde_json::json!("s")), serde_json::json!("s"));
        assert_eq!(to_sorted(serde_json::json!(true)), serde_json::json!(true));
        assert_eq!(to_sorted(serde_json::json!(null)), serde_json::json!(null));
    }

    #[test]
    fn to_sorted_sorts_top_level_object() {
        let v = serde_json::json!({"z": 1, "a": 2, "m": 3});
        let sorted = to_sorted(v);
        let keys: Vec<&String> = sorted.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["a", "m", "z"]);
    }

    #[test]
    fn to_sorted_recurses_into_nested_objects() {
        let v = serde_json::json!({"outer": {"z": 1, "a": 2}});
        let sorted = to_sorted(v);
        let inner_keys: Vec<&String> = sorted["outer"].as_object().unwrap().keys().collect();
        assert_eq!(inner_keys, vec!["a", "z"]);
    }

    #[test]
    fn to_sorted_recurses_into_arrays_of_objects() {
        let v = serde_json::json!([{"b": 1, "a": 2}]);
        let sorted = to_sorted(v);
        let keys: Vec<&String> = sorted[0].as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn to_sorted_empty_object() {
        let sorted = to_sorted(serde_json::json!({}));
        assert_eq!(sorted, serde_json::json!({}));
    }

    #[test]
    fn to_sorted_empty_array() {
        let sorted = to_sorted(serde_json::json!([]));
        assert_eq!(sorted, serde_json::json!([]));
    }

    #[test]
    fn to_sorted_pretty_has_sorted_keys() {
        let s = to_sorted_pretty(serde_json::json!({"z": 1, "a": 2})).unwrap();
        let a_pos = s.find("\"a\"").unwrap();
        let z_pos = s.find("\"z\"").unwrap();
        assert!(a_pos < z_pos);
    }

    #[test]
    fn to_sorted_pretty_is_multiline() {
        let s = to_sorted_pretty(serde_json::json!({"a": 1, "b": 2})).unwrap();
        assert!(s.contains('\n'));
    }

    #[test]
    fn to_sorted_line_is_single_line() {
        let s = to_sorted_line(serde_json::json!({"a": 1, "b": [1, 2]})).unwrap();
        assert!(!s.contains('\n'));
    }

    #[test]
    fn to_sorted_line_sorted_keys() {
        let s = to_sorted_line(serde_json::json!({"z": 1, "a": 2})).unwrap();
        assert!(s.starts_with("{\"a\":2,\"z\":1}"));
    }

    #[test]
    fn to_sorted_is_deterministic() {
        let v1 = serde_json::json!({"b": 1, "a": {"d": 4, "c": 3}});
        let v2 = serde_json::json!({"a": {"c": 3, "d": 4}, "b": 1});
        assert_eq!(to_sorted_line(v1).unwrap(), to_sorted_line(v2).unwrap());
    }
}
