//! {{project_name}} - {{description}}

/// Returns a greeting message.
pub fn greet() -> &'static str {
    "Hello, World!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet(), "Hello, World!");
    }
}
