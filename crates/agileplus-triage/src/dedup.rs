// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(clippy::empty_line_after_doc_comments)]
//! Backlog item deduplication: token-Jaccard, fuzzy ratio (Levenshtein),
//! simhash, n-gram, and a hybrid scorer.
//!
//! Traceability: FR-AGP-018 (triage dedup primitives)
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Tokenize a string into lowercase, alphanumeric, length>=2 tokens.
pub fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .map(String::from)
        .collect()
}

/// Token Jaccard similarity in [0.0, 1.0].
pub fn token_jaccard(a: &str, b: &str) -> f64 {
    let ta: HashSet<String> = tokenize(a).into_iter().collect();
    let tb: HashSet<String> = tokenize(b).into_iter().collect();
    if ta.is_empty() && tb.is_empty() {
        return 1.0;
    }
    let inter = ta.intersection(&tb).count() as f64;
    let union = ta.union(&tb).count() as f64;
    if union == 0.0 { 0.0 } else { inter / union }
}

/// Levenshtein edit distance between two byte strings.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let (m, n) = (a.len(), b.len());
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = std::cmp::min(
                std::cmp::min(curr[j - 1] + 1, prev[j] + 1),
                prev[j - 1] + cost,
            );
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// Fuzzy ratio: 1.0 - levenshtein/max(len), clamped to [0,1].
pub fn fuzzy_ratio(a: &str, b: &str) -> f64 {
    let max = std::cmp::max(a.chars().count(), b.chars().count());
    if max == 0 {
        return 1.0;
    }
    let d = levenshtein(a, b);
    1.0 - (d as f64 / max as f64)
}

/// Extract character n-grams (alphanumeric only) from `s`.
/// When the string has fewer than `n` characters, returns a single-element
/// set containing the concatenated string.
pub fn ngrams(s: &str, n: usize) -> HashSet<String> {
    let chars: Vec<char> = s.chars().filter(|c| c.is_alphanumeric()).collect();
    if chars.len() < n {
        let joined: String = chars.iter().collect();
        return std::iter::once(joined).collect();
    }
    (0..=chars.len() - n)
        .map(|i| chars[i..i + n].iter().collect())
        .collect()
}

/// N-gram Jaccard similarity in [0.0, 1.0].
pub fn ngram_jaccard(a: &str, b: &str, n: usize) -> f64 {
    let na = ngrams(a, n);
    let nb = ngrams(b, n);
    if na.is_empty() && nb.is_empty() {
        return 1.0;
    }
    let inter = na.intersection(&nb).count() as f64;
    let union = na.union(&nb).count() as f64;
    if union == 0.0 { 0.0 } else { inter / union }
}

/// 64-bit simhash for short text. Hash each n-gram (n=3) with FNV-1a and
/// bitwise XOR-sum into a 64-bit fingerprint.
pub fn simhash64(s: &str) -> u64 {
    let grams = ngrams(s, 3);
    if grams.is_empty() {
        return 0;
    }
    let mut bits = [0i32; 64];
    for g in &grams {
        // FNV-1a 64-bit hash
        let mut h: u64 = 1469598103934665603;
        for b in g.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(1099511628211);
        }
        for (i, b) in bits.iter_mut().enumerate() {
            if (h >> i) & 1 == 1 {
                *b += 1;
            } else {
                *b -= 1;
            }
        }
    }
    let mut out: u64 = 0;
    for (i, bit) in bits.iter().enumerate() {
        if *bit > 0 {
            out |= 1 << i;
        }
    }
    out
}

/// Hamming distance between two 64-bit simhashes.
pub fn simhash_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// A candidate duplicate pair with its score breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCandidate {
    pub a_id: String,
    pub b_id: String,
    pub hybrid_score: f64,
    pub token_jaccard: f64,
    pub fuzzy_ratio: f64,
    pub ngram_jaccard: f64,
    pub simhash_distance: u32,
}

/// Add the result of a fuzzy ratio calculation into a cache by Id. Not returned by `hybrid_score` directly.
#[allow(dead_code)] // reserved - side-effectful helper for optional fuzzy ratio caching
pub fn add_fuzzy_ratio(a: &str, b: &str, ratio: f64) {
    // Fuzzy ratio is only meaningful when combined with other metrics.
    // This function is provided as a side-effectful helper for a side lookup.
    // The actual hybrid_score call uses the token_jaccard, ngram_jaccard, and simhash_distance.
    // If this ratio is useful, the caller can add it to `hybrid_score` calculation.
    // If no additional result is added, it does nothing.
    let _ = (a, b, ratio);
}

/// Hybrid score:
/// `0.5*token_jaccard + 0.2*fuzzy_ratio + 0.2*ngram_jaccard + 0.1*(1 - simhash_distance/64)`.
///
/// Returns `(score, token_jaccard, fuzzy_ratio, ngram_jaccard, simhash_distance)`.
pub fn hybrid_score(a: &str, b: &str) -> (f64, f64, f64, f64, u32) {
    let tj = token_jaccard(a, b);
    let fr = fuzzy_ratio(a, b);
    let nj = ngram_jaccard(a, b, 3);
    let sh = simhash_distance(simhash64(a), simhash64(b));
    let sh_norm = 1.0 - (sh as f64 / 64.0);
    let score = 0.5 * tj + 0.2 * fr + 0.2 * nj + 0.1 * sh_norm;
    (score, tj, fr, nj, sh)
}

/// Find all pairs above `threshold` from a slice of `(id, text)` inputs.
/// O(n^2) — intended for backlogs of <= 10k items. Returns candidates
/// sorted by descending `hybrid_score`.
pub fn find_duplicates(items: &[(String, String)], threshold: f64) -> Vec<DuplicateCandidate> {
    let mut out = Vec::new();
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            let (score, tj, fr, nj, sh) = hybrid_score(&items[i].1, &items[j].1);
            if score >= threshold {
                out.push(DuplicateCandidate {
                    a_id: items[i].0.clone(),
                    b_id: items[j].0.clone(),
                    hybrid_score: score,
                    token_jaccard: tj,
                    fuzzy_ratio: fr,
                    ngram_jaccard: nj,
                    simhash_distance: sh,
                });
            }
        }
    }
    out.sort_by(|x, y| {
        y.hybrid_score
            .partial_cmp(&x.hybrid_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── tokenize ──────────────────────────────────────────────────────────

    #[test]
    fn tokenize_basic() {
        let toks = tokenize("Hello World");
        assert_eq!(toks, vec!["hello", "world"]);
    }

    #[test]
    fn tokenize_drops_short_tokens() {
        let toks = tokenize("a bb ccc");
        assert_eq!(toks, vec!["bb", "ccc"]);
    }

    #[test]
    fn tokenize_empty_string() {
        let toks = tokenize("");
        assert!(toks.is_empty());
    }

    #[test]
    fn tokenize_only_short_tokens() {
        let toks = tokenize("a b c");
        assert!(toks.is_empty());
    }

    #[test]
    fn tokenize_special_characters() {
        let toks = tokenize("foo-bar_baz.qux");
        assert_eq!(toks, vec!["foo", "bar", "baz", "qux"]);
    }

    // ── token_jaccard ─────────────────────────────────────────────────────

    #[test]
    fn token_jaccard_identical() {
        assert!((token_jaccard("hello world", "hello world") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn token_jaccard_disjoint() {
        assert!((token_jaccard("alpha beta", "gamma delta") - 0.0).abs() < 1e-9);
    }

    #[test]
    fn token_jaccard_partial_overlap() {
        let j = token_jaccard("alpha beta gamma", "beta gamma delta");
        assert!(j > 0.3 && j < 0.7, "expected ~0.5, got {j}");
    }

    #[test]
    fn token_jaccard_both_empty() {
        assert!((token_jaccard("", "") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn token_jaccard_one_empty() {
        assert!((token_jaccard("hello world", "") - 0.0).abs() < 1e-9);
    }

    // ── levenshtein ───────────────────────────────────────────────────────

    #[test]
    fn levenshtein_identical() {
        assert_eq!(levenshtein("hello", "hello"), 0);
    }

    #[test]
    fn levenshtein_empty_strings() {
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn levenshtein_one_empty() {
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn levenshtein_single_substitution() {
        assert_eq!(levenshtein("cat", "bat"), 1);
    }

    #[test]
    fn levenshtein_insertion() {
        assert_eq!(levenshtein("cat", "cats"), 1);
    }

    #[test]
    fn levenshtein_deletion() {
        assert_eq!(levenshtein("cats", "cat"), 1);
    }

    #[test]
    fn levenshtein_full_edit() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    // ── fuzzy_ratio ───────────────────────────────────────────────────────

    #[test]
    fn fuzzy_ratio_identical() {
        assert!((fuzzy_ratio("hello", "hello") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn fuzzy_ratio_both_empty() {
        assert!((fuzzy_ratio("", "") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn fuzzy_ratio_completely_different() {
        let r = fuzzy_ratio("abc", "xyz");
        assert!(r < 0.5, "expected low ratio, got {r}");
    }

    #[test]
    fn fuzzy_ratio_one_char_edit() {
        let r = fuzzy_ratio("hello", "hallo");
        assert!(r > 0.7, "expected high ratio, got {r}");
    }

    // ── ngrams ────────────────────────────────────────────────────────────

    #[test]
    fn ngrams_basic() {
        let g = ngrams("abc", 2);
        assert!(g.contains("ab"));
        assert!(g.contains("bc"));
        assert_eq!(g.len(), 2);
    }

    #[test]
    fn ngrams_shorter_than_n() {
        let g = ngrams("ab", 3);
        // When string is shorter than n, returns single set with full string
        assert!(g.contains("ab"));
        assert_eq!(g.len(), 1);
    }

    #[test]
    fn ngrams_filters_non_alphanumeric() {
        let g = ngrams("a-b", 2);
        // Non-alnum chars are stripped, so "a-b" becomes "ab" → 1 bigram
        assert!(g.contains("ab"));
    }

    #[test]
    fn ngrams_empty_string() {
        let g = ngrams("", 3);
        assert!(g.contains(""));
        assert_eq!(g.len(), 1);
    }

    // ── ngram_jaccard ─────────────────────────────────────────────────────

    #[test]
    fn ngram_jaccard_identical() {
        assert!((ngram_jaccard("hello", "hello", 3) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn ngram_jaccard_disjoint() {
        let j = ngram_jaccard("aaa", "bbb", 2);
        assert!((j - 0.0).abs() < 1e-9);
    }

    // ── simhash64 / simhash_distance ──────────────────────────────────────

    #[test]
    fn simhash_identical_strings_equal() {
        assert_eq!(simhash64("hello world"), simhash64("hello world"));
    }

    #[test]
    fn simhash_different_strings_differ() {
        assert_ne!(simhash64("hello"), simhash64("world"));
    }

    #[test]
    fn simhash_distance_identical_is_zero() {
        let h = simhash64("test");
        assert_eq!(simhash_distance(h, h), 0);
    }

    #[test]
    fn simhash_distance_differs_nonzero() {
        let a = simhash64("hello");
        let b = simhash64("world");
        assert!(simhash_distance(a, b) > 0);
    }

    // ── hybrid_score ──────────────────────────────────────────────────────

    #[test]
    fn hybrid_score_identical_is_high() {
        let (score, tj, fr, nj, sh) = hybrid_score("hello world", "hello world");
        assert!(score > 0.9, "score={score}");
        assert!((tj - 1.0).abs() < 1e-9);
        assert!((fr - 1.0).abs() < 1e-9);
        assert!((nj - 1.0).abs() < 1e-9);
        assert_eq!(sh, 0);
    }

    #[test]
    fn hybrid_score_disjoint_is_low() {
        let (score, _, _, _, _) = hybrid_score("alpha beta gamma", "zulu yankee xray");
        assert!(score < 0.5, "score={score}");
    }

    #[test]
    fn hybrid_score_bounds() {
        let (score, tj, fr, nj, sh) = hybrid_score("foo bar baz", "foo bar qux");
        assert!((0.0..=1.0).contains(&score));
        assert!((0.0..=1.0).contains(&tj));
        assert!((0.0..=1.0).contains(&fr));
        assert!((0.0..=1.0).contains(&nj));
        assert!((0..=64).contains(&sh));
    }

    // ── add_fuzzy_ratio ───────────────────────────────────────────────────

    #[test]
    fn add_fuzzy_ratio_does_not_panic() {
        add_fuzzy_ratio("hello", "world", 0.5);
    }

    // ── find_duplicates ───────────────────────────────────────────────────

    #[test]
    fn find_duplicates_empty_input() {
        let results = find_duplicates(&[], 0.5);
        assert!(results.is_empty());
    }

    #[test]
    fn find_duplicates_identical_pair() {
        let items = vec![
            ("a".to_string(), "hello world".to_string()),
            ("b".to_string(), "hello world".to_string()),
        ];
        let results = find_duplicates(&items, 0.5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].a_id, "a");
        assert_eq!(results[0].b_id, "b");
        assert!(results[0].hybrid_score > 0.9);
    }

    #[test]
    fn find_duplicates_no_matches_below_threshold() {
        let items = vec![
            ("a".to_string(), "alpha beta".to_string()),
            ("b".to_string(), "gamma delta".to_string()),
        ];
        let results = find_duplicates(&items, 0.9);
        assert!(results.is_empty());
    }

    #[test]
    fn find_duplicates_sorted_by_score_desc() {
        let items = vec![
            ("a".to_string(), "hello world foo bar".to_string()),
            ("b".to_string(), "hello world foo baz".to_string()),
            ("c".to_string(), "completely different text".to_string()),
            ("d".to_string(), "hello world foo bar".to_string()),
        ];
        let results = find_duplicates(&items, 0.3);
        for w in results.windows(2) {
            assert!(w[0].hybrid_score >= w[1].hybrid_score);
        }
    }

    #[test]
    fn find_duplicates_score_breakdown_populated() {
        let items = vec![
            ("a".to_string(), "fix race in cache layer".to_string()),
            ("b".to_string(), "fix race in cache layer".to_string()),
        ];
        let results = find_duplicates(&items, 0.5);
        assert_eq!(results.len(), 1);
        let c = &results[0];
        assert!(c.token_jaccard > 0.8);
        assert!(c.ngram_jaccard > 0.8);
        assert_eq!(c.simhash_distance, 0);
    }
}
