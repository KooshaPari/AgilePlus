//! Mock builder for constructing mock instances

use super::{MockError, Result, Verifiable};
use std::any::Any;
use std::collections::HashMap;

/// Builder for creating mock instances with multiple methods
#[derive(Debug, Default)]
pub struct MockBuilder {
    methods: HashMap<String, Box<dyn Any>>,
}

impl MockBuilder {
    /// Create a new mock builder
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
        }
    }

    /// Add a method to the mock
    pub fn with_method<T: Any + 'static>(mut self, name: &str, implementation: T) -> Self {
        self.methods.insert(name.to_string(), Box::new(implementation));
        self
    }

    /// Build the mock instance
    pub fn build(self) -> MockInstance {
        MockInstance {
            methods: self.methods,
        }
    }
}

/// A constructed mock instance
#[derive(Debug)]
pub struct MockInstance {
    methods: HashMap<String, Box<dyn Any>>,
}

impl MockInstance {
    /// Create a new empty mock instance
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
        }
    }

    /// Get a method implementation
    pub fn get_method<T: Any + Clone>(&self, name: &str) -> Option<T> {
        self.methods
            .get(name)
            .and_then(|m| m.downcast_ref::<T>())
            .cloned()
    }

    /// Set a method implementation
    pub fn set_method<T: Any + 'static>(&mut self, name: &str, implementation: T) {
        self.methods
            .insert(name.to_string(), Box::new(implementation));
    }

    /// Call a method by name
    pub fn call<T: Any, R: Any + Clone, F>(&self, name: &str, args: T) -> Option<R>
    where
        F: Fn(T) -> R + Clone + 'static,
    {
        self.methods
            .get(name)
            .and_then(|m| m.downcast_ref::<F>())
            .map(|f| f(args))
    }
}

impl Default for MockInstance {
    fn default() -> Self {
        Self::new()
    }
}

impl Verifiable for MockInstance {
    fn verify(&self) -> Result<()> {
        // MockInstance doesn't track calls by default
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_builder() {
        let mock = MockBuilder::new()
            .with_method("add", Box::new(|x: i32| x + 1) as Box<dyn Fn(i32) -> i32>)
            .with_method("multiply", Box::new(|x: i32| x * 2) as Box<dyn Fn(i32) -> i32>)
            .build();

        let add_fn: Option<Box<dyn Fn(i32) -> i32>> = mock.get_method("add");
        assert!(add_fn.is_some());
        if let Some(add) = add_fn {
            assert_eq!(add(5), 6);
        }
    }

    #[test]
    fn test_mock_instance_set_method() {
        let mut mock = MockInstance::new();
        mock.set_method("get", Box::new(|_: ()| -> i32 { 42 }) as Box<dyn Fn(()) -> i32>);

        let result: Option<Box<dyn Fn(()) -> i32>> = mock.get_method("get");
        assert!(result.is_some());
        if let Some(get_fn) = result {
            assert_eq!(get_fn(()), 42);
        }
    }
}
