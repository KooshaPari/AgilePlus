# traceability-macros

Proc macros for Phenotype Traceability system.

## Usage

```rust
use traceability_macros::trace_to;

#[trace_to("FR-AGILE-001")]
#[test]
fn test_feature() {
    // Test implementation
}
```

## License

Apache-2.0
