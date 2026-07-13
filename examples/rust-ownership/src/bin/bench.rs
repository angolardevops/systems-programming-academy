//! Dependency-free micro-benchmark that quantifies the cost the lesson talks
//! about: **borrowing vs cloning** when passing data to a function.
//!
//! Run with an optimized build for representative numbers:
//!
//! ```text
//! cargo run --release --bin bench
//! ```
//!
//! It reports nanoseconds-per-iteration for three strategies of summing a
//! large vector repeatedly. The numbers in the lesson's benchmark table come
//! from running exactly this binary.

use std::time::Instant;

use ownership::sum;

const N: usize = 100_000; // elements per vector
const ITERS: u32 = 2_000; // measured iterations

fn time_it(label: &str, mut f: impl FnMut() -> u64) {
    // Warm up so we measure steady-state, not first-touch page faults.
    let mut acc = 0u64;
    for _ in 0..50 {
        acc = acc.wrapping_add(f());
    }
    let start = Instant::now();
    for _ in 0..ITERS {
        acc = acc.wrapping_add(f());
    }
    let elapsed = start.elapsed();
    let per_iter = elapsed.as_nanos() as f64 / ITERS as f64;
    // Print acc so the optimizer cannot delete the whole loop as dead code.
    println!("{label:<28} {per_iter:>12.1} ns/iter   (checksum {acc})");
}

fn main() {
    let data: Vec<u64> = (0..N as u64).collect();

    println!("Summing a {N}-element Vec<u64>, {ITERS} iterations each:\n");

    // 1. Borrow the vector — zero copies, the ownership-idiomatic way.
    time_it("borrow &[u64]", || sum(&data));

    // 2. Clone the whole vector on every call — the anti-pattern.
    time_it("clone Vec each call", || {
        let copy = data.clone();
        sum(&copy)
    });

    // 3. Borrow but collect into a fresh Vec first (needless allocation).
    // `iter().copied().collect()` is written out on purpose to make the
    // needless copy visible; clippy would prefer `.to_vec()` (also a copy).
    #[allow(clippy::iter_cloned_collect)]
    time_it("needless collect() copy", || {
        let copy: Vec<u64> = data.iter().copied().collect();
        sum(&copy)
    });

    println!("\nBorrowing avoids the allocation + memcpy the other two pay for.");
}
