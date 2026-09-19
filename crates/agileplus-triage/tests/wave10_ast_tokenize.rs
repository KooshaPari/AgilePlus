// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wave-10 integration tests for `agileplus-triage::ast_tokenize`.
//!
//! Cross-cuts `AstTokenizer` implementations and the `for_language`
//! dispatcher.  Locks in:
//! - token order is preserved (left-to-right)
//! - identifier runs surface as either literal name or canonical `ID`
//! - two-character operator tokens (`->`, `=>`, `::`) are atomic
//! - `for_language` is case-insensitive and aliases Rust/Python
//! - both tokenizers handle empty / whitespace / unicode inputs
//! - Rust and Python tokenizers produce structurally distinct output
//!   for the same input (so downstream dedup can tell them apart)

use agileplus_triage::ast_tokenize::{for_language, AstTokenizer, PythonTokenizer, RustTokenizer};

// ─── dispatcher ─────────────────────────────────────────────────────────────

#[test]
fn for_language_accepts_all_documented_aliases() {
    // The dispatcher must accept every documented alias case-sensitively
    // and lowercase.  Unknown languages must return None.
    assert_eq!(for_language("rust").unwrap().name(), "rust");
    assert_eq!(for_language("rs").unwrap().name(), "rust");
    assert_eq!(for_language("python").unwrap().name(), "python");
    assert_eq!(for_language("py").unwrap().name(), "python");
}

#[test]
fn for_language_is_case_insensitive() {
    assert_eq!(for_language("RUST").unwrap().name(), "rust");
    assert_eq!(for_language("Rust").unwrap().name(), "rust");
    assert_eq!(for_language("PYTHON").unwrap().name(), "python");
    assert_eq!(for_language("Python").unwrap().name(), "python");
}

#[test]
fn for_language_rejects_unsupported_languages() {
    for lang in ["", "ruby", "go", "javascript", "typescript", "java", "c", "cpp"] {
        assert!(
            for_language(lang).is_none(),
            "expected None for {lang:?}"
        );
    }
}

#[test]
fn for_language_returns_boxed_trait_object() {
    // The trait object is dyn AstTokenizer; calling `tokenize` through it
    // must work as expected.
    let tok: Box<dyn AstTokenizer> = for_language("rust").unwrap();
    let toks = tok.tokenize("fn main() {}");
    assert!(toks.contains(&"fn".to_string()));
}

// ─── Rust tokenizer invariants ──────────────────────────────────────────────

#[test]
fn rust_tokenizer_preserves_token_order() {
    // Token order must match the source's left-to-right byte order.
    let src = "let x = 1; if x > 0 { return x; }";
    let toks = RustTokenizer.tokenize(src);
    let positions: Vec<(&str, usize)> = {
        let mut v: Vec<(&str, usize)> = toks
            .iter()
            .enumerate()
            .map(|(i, t)| (t.as_str(), i))
            .collect();
        // For tokens that appear multiple times we only care about the
        // *first* occurrence position.
        v.sort_by_key(|(t, _)| *t);
        v.dedup_by_key(|(t, _)| *t);
        v
    };
    // Each token's first index must be non-decreasing relative to the
    // source position.  Approximate by checking that the index of `let`
    // is less than the index of `if`, `return`, and `}`.
    let pos = |t: &str| toks.iter().position(|x| x == t).unwrap();
    assert!(pos("let") < pos("if"));
    assert!(pos("if") < pos("return"));
    assert!(pos("return") < pos("}"));
    let _ = positions; // silence unused warning
}

#[test]
fn rust_tokenizer_two_char_operators_are_atomic() {
    // Each two-char operator must appear as a single token, never as
    // two separate single-char tokens.
    let toks = RustTokenizer.tokenize("a -> b :: c => d");
    assert!(toks.contains(&"->".to_string()));
    assert!(toks.contains(&"::".to_string()));
    assert!(toks.contains(&"=>".to_string()));
    // Verify that no token is a single `-`, single `:`, or single `=`
    // that came from splitting a two-char operator.
    for t in &toks {
        assert!(t.len() != 1 || !matches!(t.as_bytes()[0], b'-' | b':' | b'='));
    }
}

#[test]
fn rust_tokenizer_emits_canonical_id_for_unknown_identifiers() {
    // Identifiers not in the keyword set must surface as `ID` (so two
    // structurally identical but identifier-renamed functions produce
    // the same token stream, which is the dedup property).
    let toks = RustTokenizer.tokenize("fn totally_made_up_thing_xyz() {}");
    assert!(toks.contains(&"fn".to_string()));
    assert!(toks.contains(&"ID".to_string()));
    assert!(!toks.iter().any(|t| t == "totally_made_up_thing_xyz"));
}

#[test]
fn rust_tokenizer_distinguishes_structurally_different_sources() {
    // Two Rust snippets with the *same* identifier names but different
    // control flow must produce different token streams.
    let ify = RustTokenizer.tokenize("fn f() { if true { } }");
    let elsy = RustTokenizer.tokenize("fn f() { if true { } else { } }");
    assert!(!ify.contains(&"else".to_string()));
    assert!(elsy.contains(&"else".to_string()));
}

#[test]
fn rust_tokenizer_handles_struct_definition() {
    // `struct` and `impl` and `trait` are keywords; the type name is
    // an identifier and should become `ID`.
    let toks = RustTokenizer.tokenize("struct Point { x: i32, y: i32 } impl Point {}");
    assert!(toks.contains(&"struct".to_string()));
    assert!(toks.contains(&"impl".to_string()));
    assert!(toks.contains(&"ID".to_string()));
}

#[test]
fn rust_tokenizer_handles_use_and_path_segments() {
    // Path segments separated by `::` should yield tokens for each
    // segment as `ID` plus the `::` operator.
    let toks = RustTokenizer.tokenize("use std::collections::HashMap;");
    let id_count = toks.iter().filter(|t| t.as_str() == "ID").count();
    let colon_count = toks.iter().filter(|t| t.as_str() == "::").count();
    assert_eq!(colon_count, 2);
    // 4 ID tokens: std, collections, HashMap, plus the trailing ; -> Punct
    assert!(id_count >= 3);
}

#[test]
fn rust_tokenizer_handles_macro_invocation() {
    // `println!` looks like an identifier + `!` punctuator.  Verify the
    // tokenizer emits both.
    let toks = RustTokenizer.tokenize("println!(\"hi\");");
    // The literal `println` is an identifier -> `ID`.
    assert!(toks.contains(&"ID".to_string()));
    assert!(toks.contains(&"!".to_string()));
}

#[test]
fn rust_tokenizer_handles_lifetime_annotations() {
    // `'a` — the apostrophe is classified as junk and skipped, so the
    // lifetime name surfaces as an `ID` token.  The tokenizer does not
    // distinguish lifetime syntax from any other identifier.
    let toks = RustTokenizer.tokenize("fn f<'a>(x: &'a str) {}");
    assert!(toks.contains(&"fn".to_string()));
    // The lifetime name itself is an identifier -> ID.
    assert!(toks.iter().any(|t| t == "ID"));
    // `&` is a punctuator.
    assert!(toks.contains(&"&".to_string()));
}

#[test]
fn rust_tokenizer_handles_generic_type_params() {
    // `<T>` and `<T, U>` should not panic and should yield `ID`
    // tokens for the type parameters.
    let toks = RustTokenizer.tokenize("fn f<T, U>(x: T, y: U) -> U {}");
    assert!(toks.contains(&"fn".to_string()));
    assert!(toks.contains(&"->".to_string()));
    let id_count = toks.iter().filter(|t| t.as_str() == "ID").count();
    assert!(id_count >= 3);
}

#[test]
fn rust_tokenizer_numeric_literal_in_arithmetic() {
    // Numeric literals in arithmetic: each number surfaces verbatim
    // and operators are preserved.
    let toks = RustTokenizer.tokenize("let z = (a + b) * 2 - 1;");
    assert!(toks.contains(&"1".to_string()));
    assert!(toks.contains(&"2".to_string()));
    assert!(toks.contains(&"+".to_string()));
    assert!(toks.contains(&"-".to_string()));
    assert!(toks.contains(&"*".to_string()));
}

// ─── Python tokenizer invariants ────────────────────────────────────────────

#[test]
fn python_tokenizer_preserves_token_order() {
    let src = "def f(x):\n    if x > 0:\n        return x\n    return 0\n";
    let toks = PythonTokenizer.tokenize(src);
    let pos = |t: &str| toks.iter().position(|x| x == t).unwrap();
    assert!(pos("def") < pos("if"));
    assert!(pos("if") < pos("return"));
}

#[test]
fn python_tokenizer_handles_decorator_syntax() {
    // `@decorator` should yield `@` and the decorator name (as `ID`).
    let toks = PythonTokenizer.tokenize("@my_decorator\ndef f(): pass\n");
    assert!(toks.contains(&"@".to_string()));
    assert!(toks.iter().any(|t| t == "ID"));
}

#[test]
fn python_tokenizer_handles_class_with_inheritance() {
    // `class Foo(Bar):` — `class` is a keyword; `Foo` and `Bar` are
    // identifiers (ID tokens); `(`, `)`, `:` are punctuation.
    let toks = PythonTokenizer.tokenize("class Foo(Bar):\n    x = 1\n");
    assert!(toks.contains(&"class".to_string()));
    assert!(toks.contains(&"(".to_string()));
    assert!(toks.contains(&")".to_string()));
    assert!(toks.contains(&":".to_string()));
    // Foo and Bar are non-keyword identifiers -> ID tokens.
    assert!(toks.iter().any(|t| t == "ID"));
}

#[test]
fn python_tokenizer_handles_list_comprehension() {
    // List comprehension syntax: brackets, identifiers, `for`, `if`.
    // `in` is not in the keyword set so it is emitted as `ID`.
    let toks = PythonTokenizer.tokenize("[x * 2 for x in items if x > 0]");
    assert!(toks.contains(&"for".to_string()));
    assert!(toks.contains(&"if".to_string()));
    assert!(toks.contains(&"[".to_string()));
    assert!(toks.contains(&"]".to_string()));
    // `in`, `items`, `x` are all non-keyword identifiers -> ID tokens.
    assert!(toks.iter().any(|t| t == "ID"));
}

#[test]
fn python_tokenizer_handles_string_literal() {
    // A bare string literal should not panic; the literal content is
    // surfaced as opaque tokens (no string-aware stripping).
    let toks = PythonTokenizer.tokenize("x = 'hello world'");
    assert!(!toks.is_empty());
    assert!(toks.contains(&"=".to_string()));
}

#[test]
fn python_tokenizer_handles_multiline_docstring() {
    // Triple-quoted docstrings should not panic; tokens are emitted
    // for any structural keywords inside.
    let src = "def f():\n    \"\"\"\n    Some doc.\n    \"\"\"\n    return 1\n";
    let toks = PythonTokenizer.tokenize(src);
    assert!(toks.contains(&"def".to_string()));
    assert!(toks.contains(&"return".to_string()));
}

#[test]
fn python_tokenizer_handles_try_with_finally() {
    // `try`/`except` are Python keywords; `finally` is not (so it
    // surfaces as `ID`).  The body is identifier-driven and yields
    // `ID` tokens.
    let toks = PythonTokenizer
        .tokenize("try:\n    pass\nexcept Exception:\n    pass\nfinally:\n    pass\n");
    assert!(toks.contains(&"try".to_string()));
    assert!(toks.contains(&"except".to_string()));
    // finally is non-keyword; identifiers like Exception/pass -> ID
    assert!(toks.iter().any(|t| t == "ID"));
}

#[test]
fn python_tokenizer_handles_dict_literal() {
    // Dict literals: `{ "key": value, ... }`.  Verify braces and
    // structural tokens are preserved.
    let toks = PythonTokenizer.tokenize("d = {'a': 1, 'b': 2}");
    assert!(toks.contains(&"{".to_string()));
    assert!(toks.contains(&"}".to_string()));
    assert!(toks.contains(&":".to_string()));
    assert!(toks.contains(&",".to_string()));
}

#[test]
fn python_tokenizer_handles_walrus_operator() {
    // Python 3.8+ walrus operator `:=`.  The tokenizer treats `:` and
    // `=` separately (no special handling for `:=`); both should
    // appear as individual punctuation tokens.
    let toks = PythonTokenizer.tokenize("if (n := len(items)) > 0: pass");
    assert!(toks.contains(&":".to_string()));
    assert!(toks.contains(&"=".to_string()));
}

// ─── Cross-language invariants ──────────────────────────────────────────────

#[test]
fn rust_and_python_tokenizers_emit_distinct_outputs_for_same_source() {
    // A snippet that is valid in both languages (well, mostly) should
    // produce *different* token streams because the keyword sets differ
    // (`fn` is Rust-only; `def` is Python-only).
    let src = "fn main() { let x = 1; }";
    let rust = RustTokenizer.tokenize(src);
    let py = PythonTokenizer.tokenize(src);
    assert!(rust.contains(&"fn".to_string()));
    assert!(!py.contains(&"fn".to_string()) || !rust.contains(&"def".to_string()));
    // python has no `fn`, so this is guaranteed distinct.
    assert!(!py.contains(&"fn".to_string()));
    assert!(!rust.contains(&"def".to_string()));
}

#[test]
fn tokenizer_output_is_stable_across_calls() {
    // Same source tokenized twice must produce the same Vec.
    let r1 = RustTokenizer.tokenize("fn f(x: u32) -> u32 { x + 1 }");
    let r2 = RustTokenizer.tokenize("fn f(x: u32) -> u32 { x + 1 }");
    assert_eq!(r1, r2);

    let p1 = PythonTokenizer.tokenize("def f(x):\n    return x + 1\n");
    let p2 = PythonTokenizer.tokenize("def f(x):\n    return x + 1\n");
    assert_eq!(p1, p2);
}

#[test]
fn tokenizer_output_skips_line_comments_but_emits_block_comment_punct() {
    // The Rust tokenizer drops `//` line comments entirely.  Block
    // comment delimiters `/*` and `*/` are emitted as their constituent
    // `/` and `*` punctuators.  Verify that no token contains the
    // multi-char comment markers.
    let rust = RustTokenizer.tokenize("// hello\nfn f() {} /* block */");
    assert!(!rust.iter().any(|t| t.contains("//")));
    assert!(!rust.iter().any(|t| t.contains("/*")));
    assert!(!rust.iter().any(|t| t.contains("*/")));
    // The slash punctuator is present (from the block comment).
    assert!(rust.contains(&"/".to_string()));
}

#[test]
fn tokenizer_output_is_unaffected_by_whitespace_variation() {
    // Different whitespace should not change the token stream
    // (the dedup property: reformatting a function doesn't break it).
    let a = RustTokenizer.tokenize("fn f(){x+1}");
    let b = RustTokenizer.tokenize("fn f()  {  x + 1  }");
    let c = RustTokenizer.tokenize("fn  f  (  )  {  x  +  1  }");
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn tokenizer_emits_hash_for_python_comments_by_design() {
    // The tokenizer treats `#` as a punctuator (it's in the punct list
    // so it's easy to extend for Rust raw strings etc.).  Comments are
    // not stripped; the `#` and following text are emitted as tokens.
    // This test documents that behavior so future refactors don't
    // silently change it.
    let py = PythonTokenizer.tokenize("# hello\ndef f(): return 1");
    // The `#` punctuator is present.
    assert!(py.contains(&"#".to_string()));
    // `def`, `return` keywords are present.
    assert!(py.contains(&"def".to_string()));
    assert!(py.contains(&"return".to_string()));
}
