//! Companion library for the lesson **Ownership & the Borrow Checker**.
//!
//! Every public item here is referenced from the lesson prose and is covered
//! by the tests at the bottom of this file. Run them with:
//!
//! ```text
//! cargo test
//! ```
//!
//! The goal is not cleverness — it is to make each ownership rule observable
//! and testable, so a reader can change the code and watch the compiler react.

/// Consumes a `String`, returning its length.
///
/// The parameter is taken **by value**, so ownership moves into the function.
/// After calling this, the caller can no longer use the original binding — the
/// lesson uses this to demonstrate a *move*.
pub fn consume_string(s: String) -> usize {
    s.len()
}

/// Borrows a string slice immutably and returns its length.
///
/// Because we take `&str` (a shared borrow), the caller keeps ownership and can
/// keep using the value afterwards. Accepting `&str` rather than `&String` also
/// lets callers pass string literals and slices — the idiomatic fix for the
/// "value moved" error.
pub fn borrow_len(s: &str) -> usize {
    s.len()
}

/// Appends `" world"` to the given string through a **mutable** borrow.
///
/// Demonstrates `&mut`: exclusive access for the duration of the borrow.
pub fn push_world(s: &mut String) {
    s.push_str(" world");
}

/// Returns the first word of `text` as a **string slice** that borrows from the
/// input. The returned `&str` is tied to the lifetime of `text`, so the borrow
/// checker guarantees `text` outlives the slice.
pub fn first_word(text: &str) -> &str {
    match text.as_bytes().iter().position(|&b| b == b' ') {
        Some(i) => &text[..i],
        None => text,
    }
}

/// A tiny owned tree node used to show that `Clone` is an explicit, visible
/// cost in Rust — not something that happens silently.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub label: String,
    pub children: Vec<Node>,
}

impl Node {
    /// Creates a leaf node with no children.
    pub fn leaf(label: &str) -> Self {
        Node {
            label: label.to_string(),
            children: Vec::new(),
        }
    }

    /// Total number of nodes in the subtree rooted at `self`, computed through
    /// shared (`&`) borrows only — no allocation, no cloning.
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(Node::count).sum::<usize>()
    }
}

/// Sums a slice of `u64` through a shared borrow. Taking `&[u64]` instead of
/// `Vec<u64>` means this works for arrays, vectors and sub-slices alike, and
/// never takes ownership of the caller's data.
pub fn sum(values: &[u64]) -> u64 {
    values.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_takes_ownership_and_reports_length() {
        let s = String::from("borrow checker");
        assert_eq!(consume_string(s), 14);
        // `s` is intentionally NOT used here: it was moved into consume_string.
    }

    #[test]
    fn borrow_leaves_caller_in_control() {
        let s = String::from("still mine");
        assert_eq!(borrow_len(&s), 10);
        // Because we only borrowed, `s` is still usable:
        assert_eq!(s, "still mine");
    }

    #[test]
    fn mutable_borrow_mutates_in_place() {
        let mut s = String::from("hello");
        push_world(&mut s);
        assert_eq!(s, "hello world");
    }

    #[test]
    fn first_word_slices_without_allocating() {
        assert_eq!(first_word("hello world"), "hello");
        assert_eq!(first_word("single"), "single");
        assert_eq!(first_word(""), "");
    }

    #[test]
    fn count_traverses_by_shared_borrow() {
        let tree = Node {
            label: "root".into(),
            children: vec![Node::leaf("a"), Node::leaf("b")],
        };
        assert_eq!(tree.count(), 3);
        // The tree is untouched by counting — still fully owned and usable.
        assert_eq!(tree.children.len(), 2);
    }

    #[test]
    fn clone_is_a_deep_independent_copy() {
        let a = Node {
            label: "n".into(),
            children: vec![Node::leaf("x")],
        };
        let mut b = a.clone();
        b.label.push('!');
        assert_eq!(a.label, "n"); // original untouched
        assert_eq!(b.label, "n!");
    }

    #[test]
    fn sum_accepts_any_slice_source() {
        let v = vec![1u64, 2, 3, 4];
        let arr = [10u64, 20, 30];
        assert_eq!(sum(&v), 10);
        assert_eq!(sum(&arr), 60);
        assert_eq!(sum(&v[1..3]), 5);
    }
}
