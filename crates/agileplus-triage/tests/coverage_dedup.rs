//! Deep coverage for agileplus-triage dedup primitives:
//! tokenization, Jaccard, Levenshtein/fuzzy, n-grams, simhash,
//! the hybrid scorer, MinHash signatures, LSH banding, and the Bloom filter.

use std::collections::HashSet;

use agileplus_triage::dedup::{
    add_fuzzy_ratio, find_duplicates, fuzzy_ratio, hybrid_score, levenshtein, ngram_jaccard, ngrams,
    simhash64, simhash_distance, token_jaccard, tokenize, DuplicateCandidate,
};
use agileplus_triage::lsh::LshIndex;
use agileplus_triage::minhash::MinHash;

// ============================================================
// tokenize
// ============================================================

#[test]
fn tokenize_lowercases() {
    assert_eq!(tokenize("HELLO WORLD"), vec!["hello", "world"]);
}

#[test]
fn tokenize_splits_on_punctuation() {
    assert_eq!(
        tokenize("foo-bar_baz.qux,quux"),
        vec!["foo", "bar", "baz", "qux", "quux"]
    );
}

#[test]
fn tokenize_drops_single_chars() {
    assert_eq!(tokenize("a i o bb"), vec!["bb"]);
}

#[test]
fn tokenize_keeps_exactly_two_chars() {
    assert_eq!(tokenize("ab"), vec!["ab"]);
}

#[test]
fn tokenize_empty_and_whitespace() {
    assert!(tokenize("").is_empty());
    assert!(tokenize("   ").is_empty());
}

#[test]
fn tokenize_digits_are_tokens() {
    assert_eq!(tokenize("version 42 release 7"), vec!["version", "42", "release"]);
}

#[test]
fn tokenize_unicode_alphanumeric_preserved() {
    assert_eq!(tokenize("café"), vec!["café"]);
}

// ============================================================
// token_jaccard
// ============================================================

#[test]
fn token_jaccard_identical_is_one() {
    assert!((token_jaccard("alpha beta", "alpha beta") - 1.0).abs() < 1e-9);
}

#[test]
fn token_jaccard_disjoint_is_zero() {
    assert!((token_jaccard("alpha beta", "gamma delta") - 0.0).abs() < 1e-9);
}

#[test]
fn token_jaccard_partial_in_range() {
    let j = token_jaccard("alpha beta gamma", "beta gamma delta");
    assert!((0.0..=1.0).contains(&j));
    assert!(j > 0.0 && j < 1.0);
}

#[test]
fn token_jaccard_duplicate_tokens_ignored() {
    // Set semantics: repeated tokens do not change the score.
    assert!((token_jaccard("alpha alpha alpha", "alpha") - 1.0).abs() < 1e-9);
}

#[test]
fn token_jaccard_both_empty_is_one() {
    assert!((token_jaccard("", "") - 1.0).abs() < 1e-9);
}

#[test]
fn token_jaccard_one_empty_is_zero() {
    assert!((token_jaccard("alpha beta", "") - 0.0).abs() < 1e-9);
}

#[test]
fn token_jaccard_subset_score() {
    // {alpha} / {alpha,beta} = 0.5
    assert!((token_jaccard("alpha", "alpha beta") - 0.5).abs() < 1e-9);
}

// ============================================================
// levenshtein
// ============================================================

#[test]
fn levenshtein_identical_zero() {
    assert_eq!(levenshtein("abcdef", "abcdef"), 0);
}

#[test]
fn levenshtein_empty_both_zero() {
    assert_eq!(levenshtein("", ""), 0);
}

#[test]
fn levenshtein_insertions() {
    assert_eq!(levenshtein("", "abcd"), 4);
}

#[test]
fn levenshtein_deletions() {
    assert_eq!(levenshtein("abcd", ""), 4);
}

#[test]
fn levenshtein_classic_kitten_sitting() {
    assert_eq!(levenshtein("kitten", "sitting"), 3);
}

#[test]
fn levenshtein_single_substitution() {
    assert_eq!(levenshtein("cat", "bat"), 1);
}

#[test]
fn levenshtein_transposition_costs_two() {
    // Classic Levenshtein (no transposition primitive) costs 2.
    assert_eq!(levenshtein("ab", "ba"), 2);
}

#[test]
fn levenshtein_is_symmetric() {
    for (a, b) in [("abc", "xyz"), ("hello", "hallo"), ("", "x"), ("long", "short")] {
        assert_eq!(levenshtein(a, b), levenshtein(b, a));
    }
}

#[test]
fn levenshtein_byte_semantics_for_unicode() {
    // Documented as byte-based: "é" is 2 bytes.
    assert_eq!(levenshtein("é", "e"), 2);
}

// ============================================================
// fuzzy_ratio
// ============================================================

#[test]
fn fuzzy_ratio_identical_is_one() {
    assert!((fuzzy_ratio("hello", "hello") - 1.0).abs() < 1e-9);
}

#[test]
fn fuzzy_ratio_both_empty_is_one() {
    assert!((fuzzy_ratio("", "") - 1.0).abs() < 1e-9);
}

#[test]
fn fuzzy_ratio_single_edit_high() {
    assert!(fuzzy_ratio("hello", "hallo") > 0.7);
}

#[test]
fn fuzzy_ratio_fully_different_low() {
    assert!(fuzzy_ratio("abc", "xyz") < 0.5);
}

#[test]
fn fuzzy_ratio_is_symmetric() {
    assert!((fuzzy_ratio("abcdef", "abcxyz") - fuzzy_ratio("abcxyz", "abcdef")).abs() < 1e-9);
}

#[test]
fn fuzzy_ratio_empty_vs_nonempty_is_zero() {
    assert!((fuzzy_ratio("", "abcd") - 0.0).abs() < 1e-9);
}

// ============================================================
// ngrams
// ============================================================

#[test]
fn ngrams_produces_all_windows() {
    let g = ngrams("abcd", 2);
    assert_eq!(g.len(), 3);
    assert!(g.contains("ab"));
    assert!(g.contains("bc"));
    assert!(g.contains("cd"));
}

#[test]
fn ngrams_exact_length_yields_one() {
    assert_eq!(ngrams("abc", 3).len(), 1);
    assert!(ngrams("abc", 3).contains("abc"));
}

#[test]
fn ngrams_shorter_than_n_yields_whole_string() {
    let g = ngrams("ab", 3);
    assert_eq!(g.len(), 1);
    assert!(g.contains("ab"));
}

#[test]
fn ngrams_empty_yields_single_empty() {
    let g = ngrams("", 3);
    assert_eq!(g.len(), 1);
    assert!(g.contains(""));
}

#[test]
fn ngrams_strips_non_alphanumeric() {
    assert_eq!(ngrams("a-b", 2), ngrams("ab", 2));
}

#[test]
fn ngrams_case_sensitive() {
    assert_ne!(ngrams("AB", 2), ngrams("ab", 2));
}

// ============================================================
// ngram_jaccard
// ============================================================

#[test]
fn ngram_jaccard_identical_is_one() {
    assert!((ngram_jaccard("hello", "hello", 3) - 1.0).abs() < 1e-9);
}

#[test]
fn ngram_jaccard_disjoint_is_zero() {
    assert!((ngram_jaccard("aaa", "bbb", 2) - 0.0).abs() < 1e-9);
}

#[test]
fn ngram_jaccard_partial() {
    let j = ngram_jaccard("abcd", "abce", 2);
    // {"ab","bc","cd"} vs {"ab","bc","ce"} = 2/4 = 0.5
    assert!((j - 0.5).abs() < 1e-9, "got {j}");
}

#[test]
fn ngram_jaccard_both_empty_is_one() {
    assert!((ngram_jaccard("", "", 3) - 1.0).abs() < 1e-9);
}

#[test]
fn ngram_jaccard_is_symmetric() {
    assert!((ngram_jaccard("abcdef", "abcxyz", 3) - ngram_jaccard("abcxyz", "abcdef", 3)).abs() < 1e-9);
}

// ============================================================
// simhash
// ============================================================

#[test]
fn simhash_deterministic() {
    assert_eq!(simhash64("the quick brown fox"), simhash64("the quick brown fox"));
}

#[test]
fn simhash_near_identical_small_distance() {
    let a = simhash64("implement login flow with oauth2 support");
    let b = simhash64("implement login flow with oauth2 support.");
    assert!(simhash_distance(a, b) < 16, "distance {}", simhash_distance(a, b));
}

#[test]
fn simhash_unrelated_larger_distance() {
    let a = simhash64("database migration for postgres");
    let b = simhash64("css styling for the dashboard header");
    assert!(simhash_distance(a, b) > 8);
}

#[test]
fn simhash_distance_identical_zero() {
    let h = simhash64("anything at all");
    assert_eq!(simhash_distance(h, h), 0);
}

#[test]
fn simhash_distance_is_symmetric() {
    let a = simhash64("alpha beta");
    let b = simhash64("gamma delta");
    assert_eq!(simhash_distance(a, b), simhash_distance(b, a));
}

#[test]
fn simhash_distance_bounded_by_64() {
    let a = simhash64("one text");
    let b = simhash64("another entirely different text");
    assert!(simhash_distance(a, b) <= 64);
}

#[test]
fn simhash_empty_input_is_deterministic() {
    assert_eq!(simhash64(""), simhash64(""));
}

// ============================================================
// hybrid_score
// ============================================================

#[test]
fn hybrid_score_identical_is_near_one() {
    let (score, tj, fr, nj, sh) = hybrid_score("hello world", "hello world");
    assert!(score > 0.95);
    assert!((tj - 1.0).abs() < 1e-9);
    assert!((fr - 1.0).abs() < 1e-9);
    assert!((nj - 1.0).abs() < 1e-9);
    assert_eq!(sh, 0);
}

#[test]
fn hybrid_score_unrelated_is_low() {
    let (score, ..) = hybrid_score("alpha beta gamma", "zulu yankee xray");
    assert!(score < 0.5, "score {score}");
}

#[test]
fn hybrid_score_components_bounded() {
    let (score, tj, fr, nj, sh) = hybrid_score("foo bar baz", "foo bar qux");
    assert!((0.0..=1.0).contains(&score));
    assert!((0.0..=1.0).contains(&tj));
    assert!((0.0..=1.0).contains(&fr));
    assert!((0.0..=1.0).contains(&nj));
    assert!(sh <= 64);
}

#[test]
fn hybrid_score_weights_match_formula() {
    let (score, tj, fr, nj, sh) = hybrid_score("alpha beta", "alpha gamma");
    let expected = 0.5 * tj + 0.2 * fr + 0.2 * nj + 0.1 * (1.0 - sh as f64 / 64.0);
    assert!((score - expected).abs() < 1e-9, "{score} vs {expected}");
}

#[test]
fn hybrid_score_is_symmetric() {
    let a = hybrid_score("one two three", "one two four");
    let b = hybrid_score("one two four", "one two three");
    assert!((a.0 - b.0).abs() < 1e-9);
}

#[test]
fn add_fuzzy_ratio_is_a_noop_safe_call() {
    add_fuzzy_ratio("a", "b", 0.42);
    add_fuzzy_ratio("", "", 0.0);
}

// ============================================================
// find_duplicates
// ============================================================

fn items(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
}

#[test]
fn find_duplicates_empty_input() {
    assert!(find_duplicates(&[], 0.5).is_empty());
}

#[test]
fn find_duplicates_single_item() {
    assert!(find_duplicates(&items(&[("a", "text")]), 0.0).is_empty());
}

#[test]
fn find_duplicates_identical_pair_found() {
    let results = find_duplicates(&items(&[("a", "hello world"), ("b", "hello world")]), 0.5);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].a_id, "a");
    assert_eq!(results[0].b_id, "b");
}

#[test]
fn find_duplicates_high_threshold_filters() {
    let results = find_duplicates(&items(&[("a", "alpha beta"), ("b", "gamma delta")]), 0.9);
    assert!(results.is_empty());
}

#[test]
fn find_duplicates_threshold_zero_returns_all_pairs() {
    let results = find_duplicates(&items(&[("a", "one"), ("b", "two"), ("c", "three")]), 0.0);
    // n=3 -> 3 pairs, all satisfy score >= 0.0.
    assert_eq!(results.len(), 3);
}

#[test]
fn find_duplicates_sorted_descending() {
    let input = items(&[
        ("a", "hello world foo bar"),
        ("b", "hello world foo baz"),
        ("c", "totally different content"),
        ("d", "hello world foo bar"),
    ]);
    let results = find_duplicates(&input, 0.3);
    for w in results.windows(2) {
        assert!(w[0].hybrid_score >= w[1].hybrid_score);
    }
}

#[test]
fn find_duplicates_score_breakdown_populated() {
    let results = find_duplicates(
        &items(&[("a", "fix race in cache layer"), ("b", "fix race in cache layer")]),
        0.5,
    );
    assert_eq!(results.len(), 1);
    let c: &DuplicateCandidate = &results[0];
    assert!(c.token_jaccard > 0.8);
    assert!(c.ngram_jaccard > 0.8);
    assert_eq!(c.simhash_distance, 0);
}

#[test]
fn find_duplicates_candidate_serde_roundtrip() {
    let c = DuplicateCandidate {
        a_id: "a".into(),
        b_id: "b".into(),
        hybrid_score: 0.9,
        token_jaccard: 0.8,
        fuzzy_ratio: 0.95,
        ngram_jaccard: 0.85,
        simhash_distance: 3,
    };
    let json = serde_json::to_string(&c).unwrap();
    let back: DuplicateCandidate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.a_id, "a");
    assert_eq!(back.simhash_distance, 3);
}

#[test]
fn duplicates_are_symmetric_scoring() {
    let ab = find_duplicates(&items(&[("a", "one two three"), ("b", "one two four")]), 0.0);
    let ba = find_duplicates(&items(&[("b", "one two four"), ("a", "one two three")]), 0.0);
    assert!((ab[0].hybrid_score - ba[0].hybrid_score).abs() < 1e-9);
}

// ============================================================
// MinHash
// ============================================================

#[test]
fn minhash_length_matches_k() {
    for k in [1usize, 4, 16, 128] {
        let s = MinHash::sign("hello world", k);
        assert_eq!(s.len(), k);
        assert_eq!(s.as_slice().len(), k);
    }
}

#[test]
fn minhash_identical_texts_jaccard_one() {
    let a = MinHash::sign("the quick brown fox", 128);
    let b = MinHash::sign("the quick brown fox", 128);
    assert_eq!(a.jaccard(&b), 1.0);
    assert_eq!(a, b);
}

#[test]
fn minhash_disjoint_texts_low_jaccard() {
    let a = MinHash::sign("alpha beta gamma delta", 256);
    let b = MinHash::sign("zulu yankee xray whiskey", 256);
    assert!(a.jaccard(&b) < 0.1);
}

#[test]
fn minhash_small_edit_high_jaccard() {
    let a = MinHash::sign("implement authentication flow for login using oauth2", 256);
    let b = MinHash::sign("implement authentication flow for login using oauth2.", 256);
    assert!(a.jaccard(&b) > 0.8);
}

#[test]
fn minhash_empty_inputs_collide() {
    let a = MinHash::sign("", 64);
    let b = MinHash::sign("", 64);
    assert_eq!(a.jaccard(&b), 1.0);
}

#[test]
fn minhash_empty_vs_nonempty_low() {
    let a = MinHash::sign("", 64);
    let c = MinHash::sign("hello world", 64);
    assert!(a.jaccard(&c) < 0.1);
}

#[test]
fn minhash_is_deterministic() {
    let a = MinHash::sign("rain in spain stays mainly plain", 128);
    let b = MinHash::sign("rain in spain stays mainly plain", 128);
    assert_eq!(a, b);
}

#[test]
fn minhash_mismatched_k_compares_common_prefix() {
    let a = MinHash::sign("alpha beta gamma delta", 64);
    let b = MinHash::sign("alpha beta gamma delta", 256);
    let j = a.jaccard(&b);
    assert!((0.0..=1.0).contains(&j));
    assert!(j > 0.95);
}

#[test]
fn minhash_estimation_error_is_small() {
    fn exact(a: &str, b: &str) -> f64 {
        let ta: HashSet<String> = tokenize(a).into_iter().collect();
        let tb: HashSet<String> = tokenize(b).into_iter().collect();
        if ta.is_empty() && tb.is_empty() {
            return 1.0;
        }
        let inter = ta.intersection(&tb).count() as f64;
        let union = ta.union(&tb).count() as f64;
        if union == 0.0 { 0.0 } else { inter / union }
    }
    for (a, b) in [
        ("add login button to header", "add login form to header"),
        ("refactor user service", "rewrite user service module"),
        ("fix race in cache", "fix race condition in cache layer"),
        ("unrelated food content", "completely unrelated content"),
    ] {
        let est = MinHash::sign(a, 512).jaccard(&MinHash::sign(b, 512));
        let truth = exact(a, b);
        assert!(
            (est - truth).abs() < 0.12 || (truth < 0.1 && est < 0.15),
            "est={est:.3} truth={truth:.3} for {a:?} vs {b:?}"
        );
    }
}

#[test]
fn minhash_is_empty_false_for_positive_k() {
    let s = MinHash::sign("x", 3);
    assert!(!s.is_empty());
}

#[test]
#[should_panic(expected = "k > 0")]
fn minhash_sign_panics_on_zero_k() {
    let _ = MinHash::sign("anything", 0);
}

#[test]
fn minhash_clone_and_eq() {
    let a = MinHash::sign("hello world", 8);
    let b = a.clone();
    assert_eq!(a, b);
    assert_eq!(a.as_slice(), b.as_slice());
}

// ============================================================
// LSH
// ============================================================

fn sig(seed: u64, len: usize) -> Vec<u64> {
    (0..len).map(|i| seed.wrapping_add(i as u64)).collect()
}

#[test]
fn lsh_new_reports_parameters() {
    let idx = LshIndex::new(4, 8);
    assert_eq!(idx.num_bands(), 4);
    assert_eq!(idx.rows_per_band(), 8);
    assert_eq!(idx.expected_signature_len(), 32);
}

#[test]
fn lsh_insert_then_query_finds_document() {
    let mut idx = LshIndex::new(5, 4);
    let s = sig(7, 20);
    idx.insert("doc", &s);
    assert!(idx.query(&s).contains(&"doc".to_string()));
}

#[test]
fn lsh_query_empty_index_returns_nothing() {
    let idx = LshIndex::new(5, 4);
    assert!(idx.query(&sig(1, 20)).is_empty());
}

#[test]
fn lsh_similar_signatures_collide() {
    let mut idx = LshIndex::new(10, 5);
    let a = sig(0, 50);
    let mut b = a.clone();
    b[0] = 9999;
    idx.insert("a", &a);
    assert!(idx.query(&b).contains(&"a".to_string()));
}

#[test]
fn lsh_dissimilar_signatures_do_not_collide() {
    let mut idx = LshIndex::new(10, 5);
    idx.insert("a", &sig(0, 50));
    assert!(idx.query(&sig(10_000, 50)).is_empty());
}

#[test]
fn lsh_insert_deduplicates_ids_within_bucket() {
    let mut idx = LshIndex::new(3, 4);
    let s = sig(2, 12);
    idx.insert("dup", &s);
    idx.insert("dup", &s);
    assert_eq!(idx.query(&s).iter().filter(|x| x.as_str() == "dup").count(), 1);
}

#[test]
fn lsh_query_returns_all_matching_documents() {
    let mut idx = LshIndex::new(2, 4);
    let a = sig(0, 8);
    let mut b = a.clone();
    b[4] = 999;
    idx.insert("a", &a);
    idx.insert("b", &b);
    let found = idx.query(&a);
    assert!(found.contains(&"a".to_string()));
    assert!(found.contains(&"b".to_string()));
}

#[test]
fn lsh_band_hash_deterministic() {
    let s = sig(0, 6);
    assert_eq!(LshIndex::band_hash(&s, 0, 3), LshIndex::band_hash(&s, 0, 3));
}

#[test]
fn lsh_band_hash_varies_by_slice() {
    let s = sig(0, 6);
    assert_ne!(LshIndex::band_hash(&s, 0, 3), LshIndex::band_hash(&s, 1, 4));
}

#[test]
fn lsh_band_hash_empty_slice_is_fnv_offset() {
    let s = sig(0, 3);
    assert_eq!(LshIndex::band_hash(&s, 0, 0), 0xcbf2_9ce4_8422_2325);
}

#[test]
fn lsh_default_is_empty_index() {
    let idx = LshIndex::default();
    assert_eq!(idx.num_bands(), 0);
    assert_eq!(idx.rows_per_band(), 0);
}

#[test]
#[should_panic(expected = "num_bands must be > 0")]
fn lsh_new_zero_bands_panics() {
    let _ = LshIndex::new(0, 5);
}

#[test]
#[should_panic(expected = "rows_per_band must be > 0")]
fn lsh_new_zero_rows_panics() {
    let _ = LshIndex::new(5, 0);
}

#[test]
#[should_panic(expected = "signature length")]
fn lsh_insert_wrong_length_panics() {
    let mut idx = LshIndex::new(5, 4);
    idx.insert("x", &sig(0, 19));
}

#[test]
#[should_panic(expected = "signature length")]
fn lsh_query_wrong_length_panics() {
    let idx = LshIndex::new(5, 4);
    let _ = idx.query(&sig(0, 19));
}

#[test]
#[should_panic(expected = "band_start must be <= band_end")]
fn lsh_band_hash_inverted_range_panics() {
    let s = sig(0, 4);
    let _ = LshIndex::band_hash(&s, 3, 1);
}

#[test]
#[should_panic(expected = "band_end must be <= signature.len()")]
fn lsh_band_hash_out_of_range_panics() {
    let s = sig(0, 4);
    let _ = LshIndex::band_hash(&s, 0, 9);
}

// ============================================================
// Bloom filter
// ============================================================

#[cfg(feature = "bloom")]
mod bloom {
    use agileplus_triage::bloom::{optimal_k, optimal_m, BloomFilter};

    #[test]
    fn optimal_m_grows_with_n() {
        assert!(optimal_m(10_000, 0.01) > optimal_m(100, 0.01));
    }

    #[test]
    fn optimal_m_grows_as_fp_tightens() {
        assert!(optimal_m(10_000, 0.001) > optimal_m(10_000, 0.1));
    }

    #[test]
    fn optimal_m_is_power_of_two() {
        for (n, p) in [(100, 0.01), (10_000, 0.001), (50_000, 0.05), (1, 0.5)] {
            let m = optimal_m(n, p);
            assert!(m > 0);
            assert_eq!(m & (m - 1), 0);
        }
    }

    #[test]
    fn optimal_m_degenerate_inputs_zero() {
        assert_eq!(optimal_m(0, 0.01), 0);
        assert_eq!(optimal_m(100, 0.0), 0);
        assert_eq!(optimal_m(100, 1.0), 0);
        assert_eq!(optimal_m(100, f64::NAN), 0);
        assert_eq!(optimal_m(100, f64::INFINITY), 0);
    }

    #[test]
    fn optimal_m_capped_at_2_pow_28() {
        assert_eq!(optimal_m(usize::MAX / 2, 1e-6), 1usize << 28);
    }

    #[test]
    fn optimal_k_matches_expected_values() {
        assert_eq!(optimal_k(0.5), 1);
        assert_eq!(optimal_k(0.25), 2);
        assert_eq!(optimal_k(0.01), 7);
        assert!(optimal_k(0.001) >= 10);
    }

    #[test]
    fn optimal_k_clamps_to_16() {
        assert_eq!(optimal_k(1e-12), 16);
    }

    #[test]
    fn optimal_k_degenerate_is_one() {
        assert_eq!(optimal_k(0.0), 1);
        assert_eq!(optimal_k(1.0), 1);
        assert_eq!(optimal_k(f64::NAN), 1);
    }

    #[test]
    fn filter_new_reports_dimensions() {
        let f = BloomFilter::new(1000, 0.01);
        assert!(f.m() > 0);
        assert_eq!(f.m() & (f.m() - 1), 0);
        assert!(f.k() >= 1);
        assert_eq!(f.len(), 0);
        assert!(f.is_empty());
    }

    #[test]
    fn filter_with_defaults_is_10000_at_1pct() {
        let f = BloomFilter::with_defaults();
        assert!((f.target_fp() - 0.01).abs() < 1e-12);
        assert!(f.m() > 0);
    }

    #[test]
    fn filter_target_fp_is_clamped_low() {
        let f = BloomFilter::new(100, 1e-12);
        assert!((f.target_fp() - 1e-6).abs() < 1e-12);
    }

    #[test]
    fn filter_target_fp_is_clamped_high() {
        let f = BloomFilter::new(100, 0.99);
        assert!((f.target_fp() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn no_false_negatives_for_inserted_items() {
        let mut f = BloomFilter::new(500, 0.01);
        for i in 0..500u32 {
            let key = format!("key-{i}");
            f.insert(key.as_bytes());
            assert!(f.contains(key.as_bytes()), "false negative for {key}");
        }
        assert_eq!(f.len(), 500);
    }

    #[test]
    fn empty_filter_contains_nothing() {
        let f = BloomFilter::new(100, 0.01);
        assert!(!f.contains(b"anything"));
    }

    #[test]
    fn insert_increments_len() {
        let mut f = BloomFilter::new(10, 0.01);
        assert_eq!(f.len(), 0);
        f.insert(b"a");
        f.insert(b"b");
        assert_eq!(f.len(), 2);
        assert!(!f.is_empty());
    }

    #[test]
    fn duplicate_insert_still_counts() {
        let mut f = BloomFilter::new(10, 0.01);
        f.insert(b"same");
        f.insert(b"same");
        assert_eq!(f.len(), 2);
        assert!(f.contains(b"same"));
    }

    #[test]
    fn popcount_grows_with_inserts() {
        let mut f = BloomFilter::new(1000, 0.01);
        let before = f.popcount();
        for i in 0..100u32 {
            f.insert(format!("item-{i}").as_bytes());
        }
        assert!(f.popcount() > before);
    }

    #[test]
    fn popcount_never_exceeds_m() {
        let mut f = BloomFilter::new(100, 0.01);
        for i in 0..1000u32 {
            f.insert(format!("x{i}").as_bytes());
        }
        assert!(f.popcount() <= f.m());
    }

    #[test]
    fn empirical_fp_stays_below_target_when_sized_correctly() {
        let mut f = BloomFilter::new(1000, 0.01);
        for i in 0..1000u32 {
            f.insert(format!("key-{i}").as_bytes());
        }
        assert!(f.empirical_fp() <= 0.02, "fp {}", f.empirical_fp());
    }

    #[test]
    fn clear_resets_filter() {
        let mut f = BloomFilter::new(10, 0.01);
        f.insert(b"a");
        f.clear();
        assert_eq!(f.len(), 0);
        assert!(f.is_empty());
        assert_eq!(f.popcount(), 0);
        assert!(!f.contains(b"a"));
    }

    #[test]
    fn clear_then_reinsert_works() {
        let mut f = BloomFilter::new(10, 0.01);
        f.insert(b"a");
        f.clear();
        f.insert(b"b");
        assert!(f.contains(b"b"));
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn unicode_and_binary_keys_supported() {
        let mut f = BloomFilter::new(50, 0.01);
        f.insert("café".as_bytes());
        f.insert(&[0u8, 255, 128]);
        assert!(f.contains("café".as_bytes()));
        assert!(f.contains(&[0u8, 255, 128]));
    }

    #[test]
    fn empty_key_is_supported() {
        let mut f = BloomFilter::new(10, 0.01);
        f.insert(b"");
        assert!(f.contains(b""));
    }

    #[test]
    fn empirical_fp_zero_for_fresh_filter() {
        let f = BloomFilter::new(100, 0.01);
        assert_eq!(f.empirical_fp(), 0.0);
    }

    #[test]
    fn clone_preserves_membership() {
        let mut f = BloomFilter::new(10, 0.01);
        f.insert(b"x");
        let g = f.clone();
        assert!(g.contains(b"x"));
        assert_eq!(g.len(), 1);
        assert_eq!(g.m(), f.m());
        assert_eq!(g.k(), f.k());
    }

    #[test]
    #[should_panic(expected = "expected_items must be > 0")]
    fn new_panics_on_zero_items() {
        let _ = BloomFilter::new(0, 0.01);
    }

    #[test]
    #[should_panic(expected = "target_fp must be finite")]
    fn new_panics_on_non_finite_fp() {
        let _ = BloomFilter::new(10, f64::NAN);
    }
}
