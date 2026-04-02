use crate::greet;

#[test]
fn test_greet() {
    assert_eq!(greet(), "Hello, World!");
}
