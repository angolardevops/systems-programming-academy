//! Companion library for the lesson **Smart Pointers & Interior Mutability**.
//!
//! Three tools, three jobs:
//! - [`List`] uses `Box<T>` to build a type of *known size* that contains itself.
//! - [`Rc`]-based sharing lets several owners share read-only data.
//! - [`Counter`] uses `Rc<RefCell<T>>` to share *mutable* state safely.
//!
//! Every public item is covered by the tests at the bottom of this file:
//!
//! ```text
//! cargo test
//! ```

use std::cell::RefCell;
use std::rc::Rc;

/// A singly linked "cons list". A recursive type needs a `Box` so its size is
/// finite: `Cons` holds an `i32` and a *pointer* to the rest, not the rest
/// inline (which would be infinitely large).
#[derive(Debug, PartialEq, Eq)]
pub enum List {
    Cons(i32, Box<List>),
    Nil,
}

impl List {
    /// Builds a list from a slice, e.g. `List::from_slice(&[1, 2, 3])`.
    pub fn from_slice(items: &[i32]) -> List {
        let mut list = List::Nil;
        for &x in items.iter().rev() {
            list = List::Cons(x, Box::new(list));
        }
        list
    }

    /// Sums the list by walking the boxed tail. `Box` derefs transparently, so
    /// `*tail` gives us the inner `List` to recurse on.
    pub fn sum(&self) -> i32 {
        match self {
            List::Cons(value, tail) => value + tail.sum(),
            List::Nil => 0,
        }
    }
}

/// Demonstrates **shared ownership**: `Rc<T>` (reference counted) lets multiple
/// owners point at the same allocation, freed only when the last one is dropped.
/// Returns the strong count observed while three handles are alive.
pub fn shared_owners() -> usize {
    let a = Rc::new(String::from("shared config"));
    let _b = Rc::clone(&a); // +1 owner (cheap: bumps a counter, no deep copy)
    let _c = Rc::clone(&a); // +1 owner
    Rc::strong_count(&a) // 3 while a, _b, _c are all alive
}

/// A counter shared by multiple handles. `Rc` gives shared ownership; `RefCell`
/// adds **interior mutability** — the ability to mutate through a shared (`&`)
/// reference, with the borrow rules checked at *run time* instead of compile
/// time.
#[derive(Clone)]
pub struct Counter {
    inner: Rc<RefCell<i64>>,
}

impl Counter {
    /// Creates a counter starting at zero.
    pub fn new() -> Self {
        Counter {
            inner: Rc::new(RefCell::new(0)),
        }
    }

    /// Increments the shared value by `n`. Note it takes `&self`, yet mutates —
    /// that is interior mutability at work.
    pub fn add(&self, n: i64) {
        *self.inner.borrow_mut() += n;
    }

    /// Reads the current value.
    pub fn get(&self) -> i64 {
        *self.inner.borrow()
    }

    /// A second handle to the *same* counter (shares the Rc, not a copy).
    pub fn handle(&self) -> Counter {
        self.clone()
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boxed_list_builds_and_sums() {
        let list = List::from_slice(&[1, 2, 3, 4]);
        assert_eq!(list.sum(), 10);
    }

    #[test]
    fn empty_list_sums_to_zero() {
        assert_eq!(List::Nil.sum(), 0);
        assert_eq!(List::from_slice(&[]), List::Nil);
    }

    #[test]
    fn list_structure_is_nested_boxes() {
        // [5] == Cons(5, Box(Nil))
        assert_eq!(List::from_slice(&[5]), List::Cons(5, Box::new(List::Nil)));
    }

    #[test]
    fn rc_tracks_shared_owner_count() {
        assert_eq!(shared_owners(), 3);
    }

    #[test]
    fn rc_count_drops_when_owners_go_out_of_scope() {
        let a = Rc::new(0u8);
        assert_eq!(Rc::strong_count(&a), 1);
        {
            let _b = Rc::clone(&a);
            assert_eq!(Rc::strong_count(&a), 2);
        } // _b dropped here
        assert_eq!(Rc::strong_count(&a), 1);
    }

    #[test]
    fn interior_mutability_through_shared_handles() {
        let c = Counter::new();
        let h = c.handle(); // second handle to the SAME counter
        c.add(10);
        h.add(5); // mutates via a shared reference
        assert_eq!(c.get(), 15);
        assert_eq!(h.get(), 15); // both see the same value
    }

    #[test]
    fn default_counter_starts_at_zero() {
        assert_eq!(Counter::default().get(), 0);
    }
}
