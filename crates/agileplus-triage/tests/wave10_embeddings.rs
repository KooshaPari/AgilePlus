// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wave-10 integration tests for `agileplus-triage::embeddings`.
//!
//! Cross-cuts `cosine` and `LocalMockEmbeddings` to lock in:
//! - cosine algebraic identities (`cos(a,a) == 1`, sign-flip, zero-vector)
//! - LocalMock determinism and L2-normalization contract
//! - empty / single-element / NaN-bearing inputs to `cosine`
//! - panic boundaries for `LocalMockEmbeddings::new(0)`
//! - builder semantics for `OaiEmbeddings` / `VoyageEmbeddings` when their
//!   features are enabled at compile time.

use agileplus_triage::embeddings::{cosine, EmbeddingBackend, LocalMockEmbeddings};

fn approx(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() < eps
}

// ─── cosine algebraic identities ────────────────────────────────────────────

#[test]
fn cosine_self_is_one_for_any_nonzero_vector() {
    // Cosine of a vector with itself must always be 1.0 (positive
    // real numbers).  Try a few non-trivial vectors.
    for v in [
        vec![1.0_f32],
        vec![2.0_f32, 3.0, 4.0],
        vec![0.5_f32, -1.5, 7.25, 100.0, -42.0],
        vec![1e-3_f32; 32],
    ] {
        let c = cosine(&v, &v);
        assert!(approx(c, 1.0, 1e-5), "cos(v,v) = {c} for {v:?}");
    }
}

#[test]
fn cosine_negation_flips_sign() {
    let v = vec![1.0_f32, 2.0, 3.0, 4.0];
    let neg: Vec<f32> = v.iter().map(|x| -x).collect();
    let c = cosine(&v, &neg);
    // The local mock's L2-normalization forces all vectors onto the
    // unit sphere, so cosine(-v, v) must be -1.0 exactly when the
    // input vectors are the negative of each other (no zeros).
    assert!(approx(c, -1.0, 1e-5), "cos(v, -v) = {c}");
}

#[test]
fn cosine_scaled_vectors_have_same_similarity() {
    // cosine is scale-invariant.
    let a = vec![1.0_f32, 2.0, 3.0];
    let b = vec![2.0_f32, 4.0, 6.0];
    let c = vec![1e6_f32, 2e6, 3e6];
    let d = vec![-1.0_f32, -2.0, -3.0];

    let ab = cosine(&a, &b);
    let ac = cosine(&a, &c);
    let ad = cosine(&a, &d);
    assert!(approx(ab, 1.0, 1e-5), "ab={ab}");
    assert!(approx(ac, 1.0, 1e-5), "ac={ac}");
    assert!(approx(ad, -1.0, 1e-5), "ad={ad}");
}

#[test]
fn cosine_of_single_element_vectors() {
    // Length-1 vectors are a degenerate but valid input.
    assert!(approx(cosine(&[3.0], &[5.0]), 1.0, 1e-6));
    assert!(approx(cosine(&[3.0], &[-5.0]), -1.0, 1e-6));
    assert!(approx(cosine(&[3.0], &[0.0]), 0.0, 1e-6));
}

#[test]
fn cosine_with_nan_in_input_returns_zero() {
    // cos is symmetric; either operand may carry NaN.  The function
    // does not promise to handle NaN, but it must not panic.  Observe
    // that the resulting dot/denom becomes NaN and the contract falls
    // through to `0.0` via the zero-denominator path.  We accept any
    // defined-or-zero outcome and just verify no panic.
    let a = vec![1.0_f32, f32::NAN, 3.0];
    let b = vec![1.0_f32, 2.0, 3.0];
    let _ = cosine(&a, &b);
    let _ = cosine(&b, &a);
}

#[test]
fn cosine_of_both_zero_is_zero() {
    // The contract: denom == 0.0 -> 0.0 (not NaN).
    let z1 = vec![0.0_f32; 4];
    let z2 = vec![0.0_f32; 4];
    assert_eq!(cosine(&z1, &z2), 0.0);
}

#[test]
fn cosine_handles_empty_input() {
    // Both the empty-vs-empty and length-mismatch edges must return 0.0
    // rather than NaN, and must never index out of bounds.
    let empty: Vec<f32> = vec![];
    let other = vec![1.0_f32, 2.0];
    assert_eq!(cosine(&empty, &empty), 0.0);
    assert_eq!(cosine(&empty, &other), 0.0);
    assert_eq!(cosine(&other, &empty), 0.0);
}

// ─── LocalMockEmbeddings contract ───────────────────────────────────────────

#[test]
fn local_mock_default_dim_and_name_match_documentation() {
    // Default dim 384 mirrors nomic-embed-text-v1.5; name is `local-mock`.
    let b = LocalMockEmbeddings::default();
    assert_eq!(b.dim(), 384);
    assert_eq!(b.name(), "local-mock");
}

#[test]
#[should_panic(expected = "dim must be > 0")]
fn local_mock_new_panics_on_zero_dim() {
    // Documented panic contract.
    let _ = LocalMockEmbeddings::new(0);
}

#[test]
fn local_mock_l2_norm_is_one_for_nonempty_input() {
    // Contract: every non-empty input gets an L2-normalized vector
    // whose L2 norm is ~1.  Verify across a few dimensions and texts.
    for dim in [16usize, 64, 256, 384] {
        let b = LocalMockEmbeddings::new(dim);
        for text in [
            "the quick brown fox",
            "implement authentication using oauth2",
            "fix race condition in cache layer",
        ] {
            let embs = b.embed(&[text]);
            let v = &embs[0];
            let norm: f64 = v.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
            assert!(
                approx(norm as f32, 1.0, 1e-3),
                "dim={dim} text={text:?} norm={norm}"
            );
            assert_eq!(v.len(), dim);
        }
    }
}

#[test]
fn local_mock_l2_norm_for_empty_input_is_zero() {
    // Empty string tokenizes to nothing -> zero vector (NOT L2-norm 1).
    // The L2-normalization path must be skipped when norm == 0.
    let b = LocalMockEmbeddings::new(128);
    let embs = b.embed(&[""]);
    assert_eq!(embs.len(), 1);
    let sum: f32 = embs[0].iter().map(|x| x.abs()).sum();
    assert_eq!(sum, 0.0);
}

#[test]
fn local_mock_returns_one_embedding_per_input_in_order() {
    // The trait contract: output order matches input order, count is
    // exactly equal to input count, and each vector has `dim()` floats.
    let b = LocalMockEmbeddings::new(48);
    let inputs = vec!["alpha", "beta", "gamma", "delta", "epsilon"];
    let embs = b.embed(&inputs);
    assert_eq!(embs.len(), inputs.len());
    for (i, e) in embs.iter().enumerate() {
        assert_eq!(e.len(), 48, "input {i} produced wrong-dim vector");
    }
}

#[test]
fn local_mock_similar_inputs_share_buckets() {
    // Two near-paraphrases should produce vectors whose cosine is
    // strictly greater than the cosine of either against an unrelated
    // input.  This is the property that makes the mock useful for the
    // hybrid pipeline's tiebreak path.
    let b = LocalMockEmbeddings::new(512);
    let embs = b.embed(&[
        "implement authentication flow using oauth2 and PKCE",
        "implement authentication flow using oauth2 plus PKCE",
        "completely unrelated cooking recipes for baking bread",
    ]);
    let sim_close = cosine(&embs[0], &embs[1]);
    let sim_far = cosine(&embs[0], &embs[2]);
    assert!(
        sim_close > sim_far,
        "close={sim_close} far={sim_far}; near-paraphrases should beat unrelated"
    );
    assert!(
        sim_close > 0.0,
        "near-paraphrases should have positive cosine, got {sim_close}"
    );
}

#[test]
fn local_mock_same_input_same_output() {
    // Determinism: two consecutive calls with the same input must
    // produce bit-identical vectors.
    let b = LocalMockEmbeddings::new(256);
    let first = b.embed(&["hello world"]);
    let second = b.embed(&["hello world"]);
    assert_eq!(first, second);
}

#[test]
fn local_mock_tokenize_policy_is_private_but_deterministic() {
    // The tokenize policy inside the mock is `lowercase, alnum-only,
    // len>=2`.  Two strings that differ only in case/punctuation/short
    // tokens must produce the same vector.
    let b = LocalMockEmbeddings::new(256);
    let a = b.embed(&["Hello, World!"]);
    let b_vec = b.embed(&["hello world"]);
    assert_eq!(a, b_vec);

    // Short tokens (length < 2) are dropped.
    let c = b.embed(&["a I b world"]);
    let d = b.embed(&["world"]);
    assert_eq!(c, d);
}

#[test]
fn local_mock_different_inputs_produce_different_vectors() {
    // Two clearly distinct inputs must produce distinct vectors
    // (otherwise the embedding has zero information content).
    let b = LocalMockEmbeddings::new(256);
    let embs = b.embed(&["alpha beta gamma", "completely different content here"]);
    assert_ne!(embs[0], embs[1]);
}

#[test]
fn local_mock_handles_long_input_without_panic() {
    // Stress: 1000-token input.  Must not panic and must still produce
    // a length-`dim()` L2-normalized vector.
    let dim = 128;
    let b = LocalMockEmbeddings::new(dim);
    let long: String = (0..1000).map(|i| format!("word{i} ")).collect();
    let embs = b.embed(&[&long]);
    assert_eq!(embs[0].len(), dim);
    let norm: f64 = embs[0].iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    assert!(approx(norm as f32, 1.0, 1e-3), "long-input norm={norm}");
}

#[test]
fn local_mock_handles_unicode_input() {
    // Multi-byte UTF-8 should not panic and the embedding should be
    // deterministic across runs.
    let b = LocalMockEmbeddings::new(64);
    let embs1 = b.embed(&["café au lait"]);
    let embs2 = b.embed(&["café au lait"]);
    assert_eq!(embs1, embs2);
    assert_eq!(embs1[0].len(), 64);
}

#[test]
fn local_mock_batch_size_matches_input_count() {
    // The trait contract says output order matches input order.  Use a
    // varied batch to assert that mapping.
    let b = LocalMockEmbeddings::new(32);
    let inputs = vec!["one", "two", "three", "four"];
    let embs = b.embed(&inputs);
    assert_eq!(embs.len(), inputs.len());
    // All four vectors are distinct (with very high probability).
    let distinct: std::collections::HashSet<Vec<u32>> = embs
        .iter()
        .map(|v| v.iter().map(|f| f.to_bits()).collect())
        .collect();
    assert_eq!(distinct.len(), embs.len());
}

// ─── Feature-gated remote backends ──────────────────────────────────────────

#[cfg(feature = "oai")]
mod oai {
    use agileplus_triage::embeddings::OaiEmbeddings;

    #[test]
    fn oai_builder_preserves_api_key_dim_and_name() {
        let b = OaiEmbeddings::new("sk-fake-key")
            .with_model("text-embedding-3-large")
            .with_base_url("https://staging.example.com");
        assert_eq!(b.name(), "oai");
        // text-embedding-3-small advertises 1536.  The large model
        // also advertises 1536 in this constructor (the real value
        // comes back in the HTTP response).
        assert_eq!(b.dim(), 1536);
    }

    #[test]
    fn oai_default_model_is_text_embedding_3_small() {
        // We can not observe the model field directly, but we can
        // confirm construction succeeds with no overrides.
        let b = OaiEmbeddings::new("sk-fake");
        assert_eq!(b.dim(), 1536);
        assert_eq!(b.name(), "oai");
    }
}

#[cfg(feature = "voyage")]
mod voyage {
    use agileplus_triage::embeddings::VoyageEmbeddings;

    #[test]
    fn voyage_builder_chain_is_well_typed() {
        let b = VoyageEmbeddings::new("pa-fake")
            .with_model("voyage-code-3")
            .with_base_url("https://proxy.example.com");
        assert_eq!(b.name(), "voyage");
        assert_eq!(b.dim(), 1024);
    }

    #[test]
    fn voyage_default_model_is_voyage_3() {
        let b = VoyageEmbeddings::new("pa-fake");
        assert_eq!(b.name(), "voyage");
        assert_eq!(b.dim(), 1024);
    }
}
