//! Micro-benchmark: **iterator chain vs index loop vs intermediate `collect`**.
//!
//! Run in release mode for representative numbers:
//!
//! ```text
//! cargo run --release --bin iterbench
//! ```
//!
//! All three compute the same thing — the sum of squares of the even numbers.
//! The point: a lazy iterator chain compiles to essentially the same code as a
//! hand-written loop (zero-cost), while materialising an intermediate `Vec`
//! between steps pays for an allocation and a second pass.

use std::time::Instant;

const ITERS: u32 = 5_000;

fn time_it(label: &str, mut f: impl FnMut() -> i64) {
    let mut acc = 0i64;
    for _ in 0..100 {
        acc = acc.wrapping_add(f()); // warm up
    }
    let start = Instant::now();
    for _ in 0..ITERS {
        acc = acc.wrapping_add(f());
    }
    let per_iter = start.elapsed().as_nanos() as f64 / ITERS as f64;
    // Print acc so the optimizer cannot delete the loop as dead code.
    println!("{label:<28} {per_iter:>10.1} ns/iter   (checksum {acc})");
}

fn main() {
    let data: Vec<i64> = (0..10_000).collect();

    println!(
        "Sum of squares of evens over {} elements, {ITERS} iterations:\n",
        data.len()
    );

    // 1. Idiomatic lazy iterator chain: one pass, no allocation.
    time_it("iterator chain", || {
        data.iter().filter(|&&n| n % 2 == 0).map(|&n| n * n).sum()
    });

    // 2. Hand-written index loop: the "manual" baseline.
    time_it("index loop", || {
        let mut total = 0i64;
        for &n in &data {
            if n % 2 == 0 {
                total += n * n;
            }
        }
        total
    });

    // 3. Materialise an intermediate Vec between steps: needless allocation.
    time_it("intermediate collect", || {
        let evens: Vec<i64> = data.iter().copied().filter(|&n| n % 2 == 0).collect();
        evens.iter().map(|&n| n * n).sum()
    });

    println!("\nThe chain and the loop are neck-and-neck (zero-cost abstraction);\nthe intermediate Vec pays for an allocation and a second pass.");
}
