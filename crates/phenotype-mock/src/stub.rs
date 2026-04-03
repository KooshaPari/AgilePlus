//! Mock stub implementation for function mocking.
//!
//! Provides a simple stub implementation for creating mock functions
//! that can be used in tests.

use std::sync::{Arc, Mutex};

/// A mock stub that captures calls and returns predefined values.
///
/// # Type Parameters
///
/// * `I` - Input type (the function argument)
/// * `O` - Output type (the function return value)
#[derive(Debug)]
pub struct Stub<I, O> {
    func: Arc<Mutex<Box<dyn Fn(I) -> O + Send + Sync>>>,
    call_count: Arc<Mutex<usize>>,
    recorded_calls: Arc<Mutex<Vec<I>>>,
}

impl<I: Clone, O> Stub<I, O> {
    /// Create a new stub from a function
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(I) -> O + Send + Sync + 'static,
    {
        Self {
            func: Arc::new(Mutex::new(Box::new(func))),
            call_count: Arc::new(Mutex::new(0)),
            recorded_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Invoke the stub with the given input
    pub fn call(&self, input: I) -> O {
        {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
        }
        {
            let mut calls = self.recorded_calls.lock().unwrap();
            calls.push(input.clone());
        }
        let func = self.func.lock().unwrap();
        func(input)
    }

    /// Returns how many times this stub has been called
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    /// Returns all recorded calls
    pub fn recorded_calls(&self) -> Vec<I> {
        self.recorded_calls.lock().unwrap().clone()
    }

    /// Clear all recorded calls
    pub fn clear_calls(&self) {
        self.recorded_calls.lock().unwrap().clear();
    }

    /// Create a stub that returns a constant value
    pub fn returning(value: O) -> Self
    where
        O: Clone + Send + Sync + 'static,
    {
        Self::new(move |_input: I| value.clone())
    }

    /// Create a stub that panics when called
    pub fn panic(msg: String) -> Self
    where
        O: Send + Sync + 'static,
    {
        Self::new(move |_input: I| panic!("{}", msg))
    }
}

impl<I: Clone, O> Default for Stub<I, O>
where
    O: Default + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::returning(O::default())
    }
}

impl<I: Clone, O> Clone for Stub<I, O> {
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
            call_count: self.call_count.clone(),
            recorded_calls: self.recorded_calls.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-MOCK-001
    #[test]
    fn test_stub_call_count() {
        let stub = Stub::<i32, i32>::returning(42);
        assert_eq!(stub.call(1), 42);
        assert_eq!(stub.call(2), 42);
        assert_eq!(stub.call_count(), 2);
    }

    // Traces to: FR-MOCK-002
    #[test]
    fn test_stub_recorded_calls() {
        let stub = Stub::<i32, i32>::returning(10);
        stub.call(1);
        stub.call(2);
        stub.call(3);
        assert_eq!(stub.recorded_calls(), vec![1, 2, 3]);
    }

    // Traces to: FR-MOCK-003
    #[test]
    fn test_stub_clear_calls() {
        let stub = Stub::<String, i32>::returning(5);
        stub.call("a".to_string());
        stub.call("b".to_string());
        assert_eq!(stub.call_count(), 2);
        stub.clear_calls();
        assert_eq!(stub.call_count(), 2); // Count preserved
        assert!(stub.recorded_calls().is_empty());
    }

    // Traces to: FR-MOCK-004
    #[test]
    fn test_stub_with_custom_function() {
        let stub = Stub::new(|x: i32| x * 2);
        assert_eq!(stub.call(5), 10);
        assert_eq!(stub.call(3), 6);
    }

    // Traces to: FR-MOCK-005
    #[test]
    fn test_stub_clone_independence() {
        let stub = Stub::<i32, i32>::returning(100);
        let stub2 = stub.clone();
        stub.call(1);
        stub2.call(1);
        assert_eq!(stub.call_count(), 1);
        assert_eq!(stub2.call_count(), 1);
    }

    // Traces to: FR-MOCK-006
    #[test]
    fn test_stub_with_unit_input() {
        let stub = Stub::<(), String>::returning("done".to_string());
        assert_eq!(stub.call(()), "done");
    }

    // Traces to: FR-MOCK-007
    #[test]
    fn test_stub_panics() {
        let stub = Stub::<i32, i32>::panic("test panic".to_string());
        let result = std::panic::catch_unwind(|| stub.call(42));
        assert!(result.is_err());
    }

    // Traces to: FR-MOCK-008
    #[test]
    fn test_stub_default() {
        let stub: Stub<i32, String> = Default::default();
        assert_eq!(stub.call(1), "");
        assert_eq!(stub.call(2), "");
    }
}
