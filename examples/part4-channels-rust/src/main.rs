//! Benchmark harness: run with `cargo run --release`.
//!
//! Measures pushing 1M integers through the three channel flavours: how much
//! does backpressure (and rendezvous hand-off) cost?

use part4_channels_rust::throughput;
use std::time::Instant;

fn time(label: &str, f: impl FnOnce() -> u64) -> u64 {
    let start = Instant::now();
    let out = f();
    println!("{label:<32} {:>9.1?}", start.elapsed());
    out
}

fn main() {
    const N: u64 = 1_000_000;
    let want: u64 = (0..N).sum();

    println!("-- 1M integers, one producer -> one consumer --");
    assert_eq!(time("channel() unbounded", || throughput(N, None)), want);
    assert_eq!(
        time("sync_channel(1024) bounded", || throughput(N, Some(1024))),
        want
    );
    assert_eq!(
        time("sync_channel(0) rendezvous", || throughput(N, Some(0))),
        want
    );
}
