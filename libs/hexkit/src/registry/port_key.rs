//! Port key types

use std::any::type_name;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortKey {
    type_name: String,
    qualifier: Option<String>,
}

impl PortKey {
    pub fn new<T: 'static>() -> Self {
        Self {
            type_name: type_name::<T>().to_string(),
            qualifier: None,
        }
    }

    pub fn with_qualifier<T: 'static>(qualifier: impl Into<String>) -> Self {
        Self {
            type_name: type_name::<T>().to_string(),
            qualifier: Some(qualifier.into()),
        }
    }

    pub fn as_str(&self) -> String {
        match &self.qualifier {
            Some(q) => format!("{}:{}", self.type_name, q),
            None => self.type_name.clone(),
        }
    }
}

impl fmt::Display for PortKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
