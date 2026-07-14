//! latencytop — render a power-of-two latency histogram.
//!
//! Usage: `latencytop [unit] < samples`   (reads whitespace-separated durations
//! on stdin, one histogram to stdout)
//!
//! In production the durations come from an **eBPF** program attached to a
//! kernel tracepoint (see the lesson) — that half needs root and a BPF library.
//! This binary is the *userspace* half: it reads the same numbers from stdin, so
//! you can drive it from any source and see the exact output an eBPF tool
//! prints. Try:  `awk 'BEGIN{for(i=0;i<200;i++)print 2**int(rand()*12)}' | latencytop`
//!
//! The bucketing and rendering are the tested library; only the kernel-side
//! collection is privileged.

use part6_ebpf_rust::{render_histogram, tally};
use std::io::Read;

fn main() {
    let unit = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "usecs".to_string());

    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("latencytop: could not read stdin");
        std::process::exit(1);
    }
    let samples: Vec<u64> = input
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    if samples.is_empty() {
        eprintln!(
            "latencytop: no samples on stdin.\n\
             In production an eBPF program feeds durations; here, pipe numbers in, e.g.\n\
             printf '1 2 3 5 5 6 9 10 12 100\\n' | latencytop usecs"
        );
        std::process::exit(1);
    }

    let counts = tally(&samples);
    println!("{}", render_histogram(&counts, &unit));
}
