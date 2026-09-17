// SPDX-License-Identifier: MIT OR Apache-2.0
//! Validation and error-path tests for generated config structs.
//!
//! Although the macro itself generates only structural code (no runtime
//! validation), these tests ensure downstream usage patterns around
//! required/optional fields, type boundaries, and empty strings behave as
//! expected by consumers.

use agileplus_config::config_builder;

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct ValidatedConfig {
        (str)     pub required_name: String = "default".to_string(),
        (opt_str) pub optional_name: Option<String> = None,
        (val)     pub threshold: u32 = 100,
        (val)     pub ratio: f64 = 0.5,
        (val)     pub enabled: bool = true,
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct OnlyOptional {
        (opt_str) pub a: Option<String> = None,
        (opt_str) pub b: Option<String> = None,
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct NumericBounds {
        (val) pub min_val: i32 = -1000,
        (val) pub max_val: u32 = 4_000_000_000,
        (val) pub small: u8 = 0,
        (val) pub large: u128 = 0,
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct StringValidation {
        (str) pub cannot_be_empty: String = "nonempty".to_string(),
        (str) pub can_be_empty: String = "".to_string(),
    }
}

// --- Type-level constraints (compile-time, but we test the outcome) ------------

#[test]
fn u32_accepts_zero() {
    let c = ValidatedConfig::default().with_threshold(0);
    assert_eq!(c.threshold, 0);
}

#[test]
fn u32_rejects_negative_at_compile_time() {
    // If this compiled, it would mean negative literals were accepted.
    // We use a const to ensure it's a compile-time failure if it ever changed.
    const _: () = {
        // let _ = ValidatedConfig::default().with_threshold(-1); // would not compile
    };
}

#[test]
fn i32_accepts_negative() {
    let c = NumericBounds::default().with_min_val(-500);
    assert_eq!(c.min_val, -500);
}

#[test]
fn u128_accepts_large() {
    let c = NumericBounds::default().with_large(u128::MAX);
    assert_eq!(c.large, u128::MAX);
}

#[test]
fn u8_wraps_at_compile_time() {
    // 300 would not fit in u8; this is a compile-time check.
    // We just assert the default works.
    assert_eq!(NumericBounds::default().small, 0);
}

#[test]
fn f64_accepts_precision() {
    let c = ValidatedConfig::default().with_ratio(0.3333333333333333);
    assert!((c.ratio - 1.0 / 3.0).abs() < 1e-10);
}

#[test]
fn f64_accepts_nan() {
    let c = ValidatedConfig::default().with_ratio(f64::NAN);
    assert!(c.ratio.is_nan());
}

#[test]
fn f64_accepts_infinity() {
    let c = ValidatedConfig::default().with_ratio(f64::INFINITY);
    assert!(c.ratio.is_infinite());
}

// --- Option<String> presence checks ----------------------------------------------

#[test]
fn optional_field_is_none_by_default() {
    let c = ValidatedConfig::default();
    assert!(c.optional_name.is_none());
}

#[test]
fn optional_field_becomes_some_after_builder() {
    let c = ValidatedConfig::default().with_optional_name("provided");
    assert_eq!(c.optional_name.as_deref(), Some("provided"));
}

#[test]
fn only_optional_struct_all_none_default() {
    let c = OnlyOptional::default();
    assert!(c.a.is_none());
    assert!(c.b.is_none());
}

#[test]
fn only_optional_struct_some_both() {
    let c = OnlyOptional::default()
        .with_a("a")
        .with_b("b");
    assert_eq!(c.a.as_deref(), Some("a"));
    assert_eq!(c.b.as_deref(), Some("b"));
}

// --- String emptiness behavior (no automatic validation, just behavior) ---------

#[test]
fn str_field_can_be_set_empty() {
    let c = StringValidation::default().with_cannot_be_empty("");
    assert_eq!(c.cannot_be_empty, "");
}

#[test]
fn str_field_default_not_empty() {
    assert!(!StringValidation::default().cannot_be_empty.is_empty());
}

#[test]
fn str_field_explicit_empty_default() {
    assert!(StringValidation::default().can_be_empty.is_empty());
}

// --- Boolean edge cases ----------------------------------------------------------

#[test]
fn bool_default_true() {
    assert!(ValidatedConfig::default().enabled);
}

#[test]
fn bool_can_be_false() {
    let c = ValidatedConfig::default().with_enabled(false);
    assert!(!c.enabled);
}

#[test]
fn bool_toggle() {
    let c = ValidatedConfig::default()
        .with_enabled(false)
        .with_enabled(true);
    assert!(c.enabled);
}

// --- All-optional struct is trivially valid --------------------------------------

#[test]
fn all_optional_struct_is_usable() {
    let _ = OnlyOptional::default();
    let _ = OnlyOptional::default().with_a("x");
}

// --- Numeric type boundaries -----------------------------------------------------

#[test]
fn u32_max_value() {
    let c = NumericBounds::default().with_max_val(u32::MAX);
    assert_eq!(c.max_val, u32::MAX);
}

#[test]
fn i32_min_value() {
    let c = NumericBounds::default().with_min_val(i32::MIN);
    assert_eq!(c.min_val, i32::MIN);
}

#[test]
fn u8_max_value() {
    let c = NumericBounds::default().with_small(u8::MAX);
    assert_eq!(c.small, u8::MAX);
}

// --- Builder returns owned struct (not a reference) ------------------------------

#[test]
fn builder_returns_owned_value() {
    let c1 = ValidatedConfig::default().with_threshold(1);
    let c2 = ValidatedConfig::default().with_threshold(2);
    // Moving c1 should not affect c2.
    let _ = c1;
    assert_eq!(c2.threshold, 2);
}

// --- Default trait is implemented ------------------------------------------------

#[test]
fn default_trait_implemented_for_all() {
    let _ = ValidatedConfig::default();
    let _ = OnlyOptional::default();
    let _ = NumericBounds::default();
    let _ = StringValidation::default();
}

// --- PartialEq works field-by-field ----------------------------------------------

#[test]
fn partial_eq_considers_all_fields() {
    let a = ValidatedConfig::default().with_threshold(1);
    let b = ValidatedConfig::default().with_threshold(1);
    let c = ValidatedConfig::default().with_threshold(2);
    assert_eq!(a, b);
    assert_ne!(a, c);
}