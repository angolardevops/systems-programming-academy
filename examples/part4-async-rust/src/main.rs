//! Benchmark harness: run with `cargo run --release`.
//!
//! What do 10,000 concurrent 50ms waits cost when each one is an OS thread?
//! (Compare with the Go and Python harnesses: goroutines and asyncio tasks.)

use part4_async_rust::thread_sleepers;
use std::time::Duration;

fn main() {
    const PAUSE: Duration = Duration::from_millis(50);
    println!("-- n concurrent 50ms waits, one OS thread each --");
    for n in [1_000, 10_000] {
        let elapsed = thread_sleepers(n, PAUSE);
        println!("{n:>6} threads {elapsed:>10.1?}");
    }
}
