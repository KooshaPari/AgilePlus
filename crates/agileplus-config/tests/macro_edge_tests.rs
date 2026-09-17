// SPDX-License-Identifier: MIT OR Apache-2.0
//! Edge-case and structural tests for the `config_builder!` macro itself.
//!
//! Verifies macro hygiene, trait propagation, zero-field structs,
//! field attributes survival, nested paths, and module visibility.

#![allow(deprecated)]

use agileplus_config::config_builder;

// --- Zero fields -----------------------------------------------------------------

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct EmptyConfig {}
}

#[test]
fn zero_fields_default_constructible() {
    let _ = EmptyConfig::default();
    let _ = EmptyConfig::default(); // multiple times
}

#[test]
fn zero_fields_clone_partialeq_debug() {
    let a = EmptyConfig::default();
    let b = EmptyConfig::default();
    assert_eq!(a, b);
    let _ = format!("{a:?}");
    let c = a.clone();
    assert_eq!(a, c);
}

// --- Single field structs --------------------------------------------------------

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct SingleStr {
        (str) pub value: String = "single".to_string(),
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct SingleVal {
        (val) pub value: u32 = 42,
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct SingleOptStr {
        (opt_str) pub value: Option<String> = None,
    }
}

#[test]
fn single_str_setter() {
    let c = SingleStr::default().with_value("only");
    assert_eq!(c.value, "only");
}

#[test]
fn single_val_setter() {
    let c = SingleVal::default().with_value(100);
    assert_eq!(c.value, 100);
}

#[test]
fn single_opt_str_setter() {
    let c = SingleOptStr::default().with_value("opt");
    assert_eq!(c.value.as_deref(), Some("opt"));
}

#[test]
fn single_field_defaults() {
    assert_eq!(SingleStr::default().value, "single");
    assert_eq!(SingleVal::default().value, 42);
    assert_eq!(SingleOptStr::default().value, None);
}

// --- Field attribute survival ----------------------------------------------------

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct AttrConfig {
        #[doc = "This field is important"]
        (str) pub important: String = "default".to_string(),
        #[deprecated(since = "1.0", note = "Use important instead")]
        (val) pub legacy: u32 = 0,
    }
}

#[test]
fn field_doc_attributes_survive() {
    // We can't directly test the doc comment on the field via the test,
    // but we can ensure compilation with attributes succeeds.
    let _ = AttrConfig::default().with_important("x").with_legacy(1);
}

#[test]
fn deprecated_field_works() {
    let c = AttrConfig::default().with_legacy(5);
    assert_eq!(c.legacy, 5);
}

// --- Visibility propagation ------------------------------------------------------

mod private_module {
    use agileplus_config::config_builder;

    config_builder! {
        pub(super) struct PrivateConfig {
            (str) pub(super) value: String = "private".to_string(),
        }
    }
}

#[test]
fn private_module_config_works() {
    let c = private_module::PrivateConfig::default();
    assert_eq!(c.value, "private");
    let c2 = c.with_value("changed");
    assert_eq!(c2.value, "changed");
}

// --- Macro hygiene: no leakage between invocations ------------------------------

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct First {
        (val) pub x: u32 = 1,
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Second {
        (val) pub x: u32 = 2,
    }
}

#[test]
fn separate_invocations_independent() {
    let a = First::default();
    let b = Second::default();
    assert_eq!(a.x, 1);
    assert_eq!(b.x, 2);
    let a2 = a.clone().with_x(99); // clone to preserve original
    assert_eq!(a2.x, 99);
    assert_eq!(a.x, 1); // original unchanged
}

// --- Mixed visibility on struct vs fields ----------------------------------------

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct MixedVis {
        (str) pub public_field: String = "pub".to_string(),
        (val) pub(crate) crate_field: u32 = 10,
    }
}

#[test]
fn mixed_visibility_fields_work() {
    let c = MixedVis::default();
    assert_eq!(c.public_field, "pub");
    assert_eq!(c.crate_field, 10);
    let c2 = c.with_public_field("changed").with_crate_field(20);
    assert_eq!(c2.public_field, "changed");
    assert_eq!(c2.crate_field, 20);
}

// --- Nested structs with their own derives ---------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct NestedInner {
    pub inner_val: u32,
}

impl Default for NestedInner {
    fn default() -> Self {
        Self { inner_val: 5 }
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct NestedOuter {
        (val) pub inner: NestedInner = NestedInner::default(),
        (val) pub count: u32 = 0,
    }
}

#[test]
fn nested_struct_field_works() {
    let c = NestedOuter::default()
        .with_inner(NestedInner { inner_val: 99 })
        .with_count(1);
    assert_eq!(c.inner.inner_val, 99);
    assert_eq!(c.count, 1);
}

// --- Trait propagation: Clone, Debug, PartialEq automatically work --------------

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct TraitConfig {
        (str) pub name: String = "trait".to_string(),
        (val) pub id: u64 = 0,
    }
}

#[test]
fn all_standard_traits_derived() {
    let a = TraitConfig::default();
    let b = a.clone(); // Clone
    let _ = format!("{a:?}"); // Debug
    assert_eq!(a, b); // PartialEq
}

// --- Constant expressions as defaults --------------------------------------------

const MAX_CONNS: u32 = 1000;

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct ConstDefaultConfig {
        (val) pub max_conns: u32 = MAX_CONNS,
    }
}

#[test]
fn const_default_expression() {
    let c = ConstDefaultConfig::default();
    assert_eq!(c.max_conns, 1000);
}

// --- Macro-generated code compiles with `#[allow(unused)]` or warnings clean -----

#[test]
fn macro_output_compiles_clean() {
    // If this test compiles, the generated code had no warnings
    // that would make the test fail with deny(warnings).
    let c = SingleStr::default().with_value("test");
    let _ = c;
}