//! Companion library for the lesson **Generics & Lifetimes**.
//!
//! Generics let one piece of code work for many types; lifetimes let the
//! compiler prove that references never outlive the data they point to. Both are
//! compile-time only — they vanish before the program runs.
//!
//! Every public item is covered by the tests at the bottom of this file:
//!
//! ```text
//! cargo test
//! ```

use std::fmt::Display;

/// Returns the largest element of a slice, or `None` if it is empty.
///
/// Generic over any `T` that can be **ordered** (`PartialOrd`). One definition
/// works for `i32`, `f64`, `&str`, and anything else comparable — the compiler
/// monomorphizes a specialised copy per type it is actually called with.
pub fn largest<T: PartialOrd>(list: &[T]) -> Option<&T> {
    let mut it = list.iter();
    let mut biggest = it.next()?;
    for item in it {
        if item > biggest {
            biggest = item;
        }
    }
    Some(biggest)
}

/// A pair of values of the *same* type `T`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair<T> {
    pub first: T,
    pub second: T,
}

impl<T> Pair<T> {
    /// Builds a pair. Available for every `T`.
    pub fn new(first: T, second: T) -> Self {
        Pair { first, second }
    }

    /// Swaps the two elements, consuming and returning a new pair.
    pub fn swapped(self) -> Pair<T> {
        Pair {
            first: self.second,
            second: self.first,
        }
    }
}

// A *conditional* impl: `larger` exists only when T can be compared AND
// displayed. Different bounds unlock different methods on the same generic type.
impl<T: PartialOrd + Display> Pair<T> {
    /// Returns a reference to the larger of the two, breaking ties toward `first`.
    pub fn larger(&self) -> &T {
        if self.first >= self.second {
            &self.first
        } else {
            &self.second
        }
    }
}

/// Returns the longer of two string slices.
///
/// The lifetime `'a` ties the output to **both** inputs: the returned reference
/// is valid only as long as *both* `x` and `y` are — which is exactly what makes
/// this safe. Without the annotation the compiler cannot know which input the
/// result borrows from.
pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}

/// A struct that *holds a reference* must name the lifetime, promising the struct
/// cannot outlive the borrowed data.
#[derive(Debug, PartialEq, Eq)]
pub struct Excerpt<'a> {
    pub part: &'a str,
}

impl<'a> Excerpt<'a> {
    /// The first sentence (up to the first '.') of `text`, borrowed not copied.
    pub fn first_sentence(text: &'a str) -> Excerpt<'a> {
        let end = text.find('.').map(|i| i + 1).unwrap_or(text.len());
        Excerpt { part: &text[..end] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn largest_works_for_integers() {
        assert_eq!(largest(&[3, 7, 2, 9, 4]), Some(&9));
        assert_eq!(largest::<i32>(&[]), None);
    }

    #[test]
    fn largest_works_for_floats_and_strs() {
        assert_eq!(largest(&[1.5, 0.2, 3.9, 2.1]), Some(&3.9));
        assert_eq!(largest(&["apple", "pear", "fig"]), Some(&"pear")); // by Ord on &str
    }

    #[test]
    fn pair_new_and_swap() {
        let p = Pair::new(1, 2);
        assert_eq!(p.clone().swapped(), Pair::new(2, 1));
        assert_eq!(p.first, 1);
    }

    #[test]
    fn pair_larger_uses_conditional_impl() {
        assert_eq!(*Pair::new(3, 8).larger(), 8);
        assert_eq!(*Pair::new(9, 9).larger(), 9); // tie -> first
    }

    #[test]
    fn longest_returns_the_longer() {
        assert_eq!(longest("hello", "hi"), "hello");
        assert_eq!(longest("a", "bb"), "bb");
        assert_eq!(longest("eq", "eq"), "eq"); // tie -> first
    }

    #[test]
    fn excerpt_borrows_first_sentence() {
        let text = String::from("First. Second. Third.");
        let ex = Excerpt::first_sentence(&text);
        assert_eq!(ex.part, "First.");
        // `text` is still fully owned and usable after borrowing part of it.
        assert_eq!(text.len(), 21);
    }

    #[test]
    fn excerpt_handles_no_period() {
        let ex = Excerpt::first_sentence("no period here");
        assert_eq!(ex.part, "no period here");
    }
}
