//! Companion library for the lesson **Collections & Iterators**.
//!
//! It shows the three collections you reach for daily — `Vec<T>`, `HashMap<K,
//! V>`, and `String` — and the iterator adapters that make them a pleasure to
//! work with. Every public item is covered by the tests at the bottom of this
//! file:
//!
//! ```text
//! cargo test
//! ```

use std::collections::HashMap;

/// Counts how often each whitespace-separated word appears, case-insensitively
/// and ignoring surrounding ASCII punctuation.
///
/// Demonstrates the `HashMap` **entry API**: `entry(k).or_insert(0)` gets a
/// mutable reference to the value for `k`, inserting a default first if absent —
/// the idiomatic "get-or-create then update" in one lookup.
pub fn word_count(text: &str) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for raw in text.split_whitespace() {
        let word = raw
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if word.is_empty() {
            continue;
        }
        *counts.entry(word).or_insert(0) += 1;
    }
    counts
}

/// Returns the `n` most frequent words, most-frequent first. Ties are broken
/// alphabetically so the output is deterministic (important for tests).
pub fn top_words(text: &str, n: usize) -> Vec<(String, usize)> {
    let mut pairs: Vec<(String, usize)> = word_count(text).into_iter().collect();
    // Sort by count descending, then by word ascending for stable ordering.
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    pairs.truncate(n);
    pairs
}

/// Sum of the squares of the even numbers, expressed as a lazy iterator chain.
///
/// `filter` and `map` build a pipeline that runs in a single pass with no
/// intermediate allocation; `sum` drives it to completion.
pub fn sum_of_even_squares(nums: &[i64]) -> i64 {
    nums.iter().filter(|&&n| n % 2 == 0).map(|&n| n * n).sum()
}

/// Returns the sorted, de-duplicated values. Shows the `sort` + `dedup` idiom
/// (dedup only removes *consecutive* duplicates, so sorting must come first).
pub fn unique_sorted(nums: &[i64]) -> Vec<i64> {
    let mut v = nums.to_vec();
    v.sort_unstable();
    v.dedup();
    v
}

/// Splits numbers into (evens, odds) in one pass using `partition`.
pub fn split_parity(nums: &[i64]) -> (Vec<i64>, Vec<i64>) {
    nums.iter().partition(|&&n| n % 2 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_count_is_case_insensitive_and_strips_punctuation() {
        let counts = word_count("The cat, the CAT! the dog.");
        assert_eq!(counts.get("the"), Some(&3));
        assert_eq!(counts.get("cat"), Some(&2));
        assert_eq!(counts.get("dog"), Some(&1));
        assert_eq!(counts.get("missing"), None);
    }

    #[test]
    fn word_count_ignores_empty_and_symbol_only_tokens() {
        let counts = word_count("hello --- world");
        assert_eq!(counts.len(), 2);
        assert_eq!(counts.get("hello"), Some(&1));
    }

    #[test]
    fn top_words_orders_by_frequency_then_alphabetically() {
        let text = "b a a b c b";
        // b:3, a:2, c:1
        assert_eq!(
            top_words(text, 2),
            vec![("b".to_string(), 3), ("a".to_string(), 2)]
        );
    }

    #[test]
    fn top_words_breaks_ties_alphabetically() {
        // apple, banana and cherry all appear once -> alphabetical order.
        let text = "cherry banana apple";
        assert_eq!(
            top_words(text, 3),
            vec![
                ("apple".to_string(), 1),
                ("banana".to_string(), 1),
                ("cherry".to_string(), 1),
            ]
        );
    }

    #[test]
    fn sum_of_even_squares_runs_the_pipeline() {
        // evens: 2,4,6 -> 4+16+36 = 56
        assert_eq!(sum_of_even_squares(&[1, 2, 3, 4, 5, 6]), 56);
        assert_eq!(sum_of_even_squares(&[]), 0);
        assert_eq!(sum_of_even_squares(&[1, 3, 5]), 0);
    }

    #[test]
    fn unique_sorted_sorts_and_dedups() {
        assert_eq!(unique_sorted(&[3, 1, 2, 3, 1, 2]), vec![1, 2, 3]);
        assert_eq!(unique_sorted(&[]), Vec::<i64>::new());
    }

    #[test]
    fn split_parity_partitions_in_one_pass() {
        let (evens, odds) = split_parity(&[1, 2, 3, 4, 5]);
        assert_eq!(evens, vec![2, 4]);
        assert_eq!(odds, vec![1, 3, 5]);
    }
}
