//! Companion library for the lesson **Unsafe Rust & FFI**.
//!
//! `unsafe` does not turn off the borrow checker or invite chaos. It unlocks
//! exactly five extra abilities (raw-pointer deref, calling `unsafe` fns,
//! implementing `unsafe` traits, mutable statics, `union` fields) and asks *you*
//! to uphold the invariants the compiler normally checks. The craft is wrapping
//! a small `unsafe` core in a **safe API** — as the standard library does.
//!
//! Every public item is covered by the tests at the bottom of this file:
//!
//! ```text
//! cargo test
//! ```

use std::os::raw::c_int;
use std::slice;

/// A safe reimplementation of the standard library's `slice::split_at_mut`.
///
/// You cannot write this in safe Rust: it hands out two `&mut` references into
/// one slice, and the borrow checker can't prove they don't overlap. Internally
/// we use raw pointers and `unsafe` — but the signature is 100% safe, and the
/// split point is validated, so callers can never trigger undefined behaviour.
///
/// # Panics
///
/// Panics if `mid > slice.len()`.
pub fn split_at_mut<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    let len = slice.len();
    assert!(mid <= len, "mid {mid} out of bounds for slice of len {len}");
    let ptr = slice.as_mut_ptr();

    // SAFETY: `mid <= len` was asserted above, so both halves stay within the
    // original allocation, and `[0, mid)` and `[mid, len)` are disjoint — so the
    // two &mut slices never alias. This upholds Rust's aliasing invariant.
    unsafe {
        (
            slice::from_raw_parts_mut(ptr, mid),
            slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

/// Demonstrates creating and dereferencing **raw pointers**. Creating them is
/// safe; only dereferencing requires `unsafe`.
pub fn raw_pointer_roundtrip(value: i32) -> i32 {
    let r = &value;
    let raw: *const i32 = r; // coercion from reference to raw pointer

    // SAFETY: `raw` was derived from a live reference `r` to `value`, which is
    // still in scope here, so the pointer is valid and properly aligned.
    unsafe { *raw }
}

// --- FFI: calling C ------------------------------------------------------

// Declares a function implemented in C's standard library. Calling it is
// `unsafe` because Rust cannot verify C's contract.
extern "C" {
    fn abs(input: c_int) -> c_int;
}

/// Safe wrapper around C's `abs`. The wrapper's signature is safe; the `unsafe`
/// is confined to the FFI call, whose contract (takes an int, returns its
/// absolute value, no pointers) we know holds.
pub fn c_abs(x: i32) -> i32 {
    // SAFETY: `abs` is a pure function over an int with no memory safety
    // requirements; passing any i32 is valid.
    unsafe { abs(x as c_int) as i32 }
}

// --- FFI: exposing Rust to C --------------------------------------------

/// A Rust function exported with the C ABI so C code could call it. `no_mangle`
/// keeps the symbol name as `rust_triple`. It is a normal safe function on the
/// Rust side; we can call it directly in tests.
#[no_mangle]
pub extern "C" fn rust_triple(x: c_int) -> c_int {
    x * 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_at_mut_splits_correctly() {
        let mut v = [1, 2, 3, 4, 5];
        let (left, right) = split_at_mut(&mut v, 2);
        assert_eq!(left, &mut [1, 2]);
        assert_eq!(right, &mut [3, 4, 5]);
    }

    #[test]
    fn split_at_mut_allows_independent_mutation() {
        let mut v = [10, 20, 30, 40];
        let (a, b) = split_at_mut(&mut v, 1);
        a[0] += 1;
        b[2] += 5;
        assert_eq!(v, [11, 20, 30, 45]);
    }

    #[test]
    fn split_at_mut_edges() {
        let mut v = [1, 2, 3];
        let (l, r) = split_at_mut(&mut v, 0);
        assert!(l.is_empty());
        assert_eq!(r, &mut [1, 2, 3]);

        let (l, r) = split_at_mut(&mut v, 3);
        assert_eq!(l, &mut [1, 2, 3]);
        assert!(r.is_empty());
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn split_at_mut_panics_past_end() {
        let mut v = [1, 2, 3];
        let _ = split_at_mut(&mut v, 4);
    }

    #[test]
    fn raw_pointer_reads_the_value() {
        assert_eq!(raw_pointer_roundtrip(42), 42);
    }

    #[test]
    fn c_abs_matches_rust_abs() {
        assert_eq!(c_abs(-5), 5);
        assert_eq!(c_abs(5), 5);
        assert_eq!(c_abs(0), 0);
    }

    #[test]
    fn exported_rust_triple_works() {
        assert_eq!(rust_triple(7), 21);
        assert_eq!(rust_triple(-2), -6);
    }
}
