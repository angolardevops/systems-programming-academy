//! Threads & shared state: the three ways to share a counter safely, plus a
//! parallel sum that actually speeds up.
//!
//! The compiler is the co-star here: every function compiles only because the
//! sharing is safe. Delete the `Mutex` and keep the `+= 1` and the program does
//! not race — it fails to build.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::thread;

/// Sums `data` by splitting it into `n_threads` chunks summed in parallel.
///
/// `thread::scope` lets the spawned threads borrow `data` directly — no
/// `Arc`, no cloning — because the scope guarantees they all finish before
/// the function returns.
pub fn sum_parallel(data: &[u64], n_threads: usize) -> u64 {
    assert!(n_threads > 0, "need at least one thread");
    let chunk_size = data.len().div_ceil(n_threads).max(1);
    thread::scope(|s| {
        let handles: Vec<_> = data
            .chunks(chunk_size)
            .map(|chunk| s.spawn(move || chunk.iter().sum::<u64>()))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    })
}

/// Increments a `Mutex`-protected counter from `n_threads` threads,
/// `iters` times each. Always returns exactly `n_threads * iters`.
///
/// Scoped threads can borrow a plain `&Mutex<u64>`; `Arc` is only needed
/// when threads outlive the creating scope.
pub fn counter_mutex(n_threads: usize, iters: u64) -> u64 {
    let counter = Mutex::new(0u64);
    thread::scope(|s| {
        for _ in 0..n_threads {
            s.spawn(|| {
                for _ in 0..iters {
                    *counter.lock().unwrap() += 1;
                }
            });
        }
    });
    counter.into_inner().unwrap()
}

/// Same contract as [`counter_mutex`], but lock-free: `fetch_add` on an
/// atomic integer. One hardware instruction instead of lock/unlock.
pub fn counter_atomic(n_threads: usize, iters: u64) -> u64 {
    let counter = AtomicU64::new(0);
    thread::scope(|s| {
        for _ in 0..n_threads {
            s.spawn(|| {
                for _ in 0..iters {
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    counter.into_inner()
}

/// Batched variant: each thread accumulates locally and takes the lock
/// **once** at the end. Contention drops from `n * iters` lock operations
/// to `n`.
pub fn counter_batched(n_threads: usize, iters: u64) -> u64 {
    let counter = Mutex::new(0u64);
    thread::scope(|s| {
        for _ in 0..n_threads {
            s.spawn(|| {
                let mut local = 0u64;
                for _ in 0..iters {
                    local += 1;
                }
                *counter.lock().unwrap() += local;
            });
        }
    });
    counter.into_inner().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_parallel_matches_sequential() {
        let data: Vec<u64> = (1..=10_000).collect();
        let sequential: u64 = data.iter().sum();
        assert_eq!(sum_parallel(&data, 4), sequential);
    }

    #[test]
    fn sum_parallel_handles_more_threads_than_elements() {
        let data = [1u64, 2, 3];
        assert_eq!(sum_parallel(&data, 16), 6);
    }

    #[test]
    fn sum_parallel_empty_slice_is_zero() {
        assert_eq!(sum_parallel(&[], 4), 0);
    }

    #[test]
    fn counter_mutex_is_exact() {
        assert_eq!(counter_mutex(8, 10_000), 80_000);
    }

    #[test]
    fn counter_atomic_is_exact() {
        assert_eq!(counter_atomic(8, 10_000), 80_000);
    }

    #[test]
    fn counter_batched_is_exact() {
        assert_eq!(counter_batched(8, 10_000), 80_000);
    }
}
