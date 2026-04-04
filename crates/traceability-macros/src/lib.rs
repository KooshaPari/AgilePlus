//! Proc macros for Phenotype Traceability
//!
//! Provides the `#[trace_to]` attribute for test functions.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr};

/// Attribute macro to mark a test function as tracing to a Feature Requirement (FR).
///
/// # Example
/// ```rust
/// #[trace_to("FR-THEGENT-001")]
/// #[test]
/// fn test_feature() {
///     assert!(true);
/// }
/// ```
#[proc_macro_attribute]
pub fn trace_to(args: TokenStream, input: TokenStream) -> TokenStream {
    let fr_id = parse_macro_input!(args as LitStr);
    let input_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;

    // Generate the output function with the traceability attribute preserved
    let expanded = quote! {
        #(#fn_attrs)*
        #[doc = concat!("Traces to: ", #fr_id)]
        #fn_vis #fn_sig {
            // In a full implementation, we might register the trace here
            // For now, we just pass through to the original function body
            #fn_block
        }
    };

    TokenStream::from(expanded)
}
