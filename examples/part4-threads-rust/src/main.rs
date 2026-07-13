//! Benchmark harness: run with `cargo run --release`.
//!
//! Prints timings for (a) parallel sum speedup and (b) the cost of sharing a
//! counter three ways: mutex-per-increment, atomic, batched.

use part4_threads_rust::{counter_atomic, counter_batched, counter_mutex, sum_parallel};
use std::time::Instant;

fn time<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let out = f();
    println!("{label:<28} {:>8.1?} ", start.elapsed());
    out
}

fn main() {
    let data: Vec<u64> = (0..50_000_000).collect();
    let expected: u64 = data.iter().sum();

    println!("-- parallel sum, 50M elements --");
    assert_eq!(time("sum 1 thread", || sum_parallel(&data, 1)), expected);
    assert_eq!(time("sum 4 threads", || sum_parallel(&data, 4)), expected);

    const THREADS: usize = 4;
    const ITERS: u64 = 1_000_000;
    println!("-- shared counter, {THREADS} threads x {ITERS} increments --");
    assert_eq!(
        time("mutex per increment", || counter_mutex(THREADS, ITERS)),
        THREADS as u64 * ITERS
    );
    assert_eq!(
        time("atomic fetch_add", || counter_atomic(THREADS, ITERS)),
        THREADS as u64 * ITERS
    );
    assert_eq!(
        time("batched (lock once)", || counter_batched(THREADS, ITERS)),
        THREADS as u64 * ITERS
    );
}
