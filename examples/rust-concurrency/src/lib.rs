//! Companion library for the lesson **Concurrency Basics**.
//!
//! Rust's promise is "fearless concurrency": the same ownership and borrowing
//! rules that prevent memory bugs also prevent **data races** at compile time.
//! This crate shows the three building blocks — threads, channels, and shared
//! state via `Arc<Mutex<T>>` — with deterministic results so they can be tested.
//!
//! ```text
//! cargo test
//! ```

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

/// Sums a slice in parallel by splitting it into `threads` chunks, summing each
/// chunk on its own thread, and adding the partial sums.
///
/// Uses `thread::scope`: scoped threads are guaranteed to finish before the
/// scope returns, which lets them safely **borrow** `data` without `'static` or
/// `Arc`. The result is identical to a sequential sum — parallelism must not
/// change the answer.
pub fn parallel_sum(data: &[i64], threads: usize) -> i64 {
    let threads = threads.max(1);
    if data.is_empty() {
        return 0;
    }
    let chunk_size = data.len().div_ceil(threads);

    thread::scope(|scope| {
        let handles: Vec<_> = data
            .chunks(chunk_size)
            .map(|chunk| scope.spawn(move || chunk.iter().sum::<i64>()))
            .collect();

        handles.into_iter().map(|h| h.join().unwrap()).sum()
    })
}

/// Squares each input on a worker thread and streams the results back through a
/// channel. The order is preserved because we send in input order and the
/// receiver reads in the same order.
pub fn square_via_channel(inputs: Vec<i64>) -> Vec<i64> {
    let (tx, rx) = mpsc::channel();

    let worker = thread::spawn(move || {
        for n in inputs {
            // `send` fails only if the receiver was dropped; here it never is.
            tx.send(n * n).unwrap();
        }
        // tx is dropped here, which closes the channel and ends the `for` below.
    });

    let results: Vec<i64> = rx.iter().collect();
    worker.join().unwrap();
    results
}

/// Spawns `threads` threads that each increment a shared counter `per_thread`
/// times. `Arc` shares ownership across threads; `Mutex` guarantees only one
/// thread mutates at a time. The total is always `threads * per_thread` — no
/// lost updates, because the lock serialises the read-modify-write.
pub fn shared_counter(threads: usize, per_thread: usize) -> i64 {
    let counter = Arc::new(Mutex::new(0i64));

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let counter = Arc::clone(&counter); // each thread gets its own handle
            thread::spawn(move || {
                for _ in 0..per_thread {
                    let mut guard = counter.lock().unwrap();
                    *guard += 1;
                    // guard drops here, releasing the lock for the next thread.
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let total = *counter.lock().unwrap();
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_sum_matches_sequential() {
        let data: Vec<i64> = (1..=1000).collect();
        let expected: i64 = data.iter().sum(); // 500500
        assert_eq!(parallel_sum(&data, 4), expected);
        assert_eq!(parallel_sum(&data, 1), expected);
        assert_eq!(parallel_sum(&data, 7), expected); // uneven chunks
    }

    #[test]
    fn parallel_sum_handles_empty_and_single() {
        assert_eq!(parallel_sum(&[], 4), 0);
        assert_eq!(parallel_sum(&[42], 4), 42);
    }

    #[test]
    fn channel_preserves_order_and_squares() {
        assert_eq!(square_via_channel(vec![1, 2, 3, 4]), vec![1, 4, 9, 16]);
        assert_eq!(square_via_channel(vec![]), Vec::<i64>::new());
    }

    #[test]
    fn shared_counter_has_no_lost_updates() {
        // 8 threads * 10_000 increments = 80_000, every single time.
        assert_eq!(shared_counter(8, 10_000), 80_000);
    }

    #[test]
    fn shared_counter_edge_cases() {
        assert_eq!(shared_counter(0, 100), 0); // no threads
        assert_eq!(shared_counter(5, 0), 0); // no work
        assert_eq!(shared_counter(1, 1), 1);
    }
}
