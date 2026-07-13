//! Message passing: transfer ownership through a channel instead of sharing
//! state behind a lock.
//!
//! Rust's `std::sync::mpsc` is **m**ulti-**p**roducer, **s**ingle-**c**onsumer:
//! `Sender` clones freely, but only one `Receiver` exists. A worker pool
//! therefore shares the receiving end behind a `Mutex` — the pattern from the
//! Rust Book's final project. `send` *moves* its value: using data after
//! sending it is a compile error, which is the whole point.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

/// Squares every number using a pool of `n_workers` threads fed by a jobs
/// channel, and sums the results arriving on a results channel.
///
/// Closing behaviour drives shutdown: dropping the jobs `Sender` makes every
/// worker's `recv()` return `Err`, so they exit; dropping the last results
/// `Sender` ends the collector's iterator. No sentinel values, no flags.
pub fn sum_squares_pool(nums: Vec<u64>, n_workers: usize) -> u64 {
    assert!(n_workers > 0, "need at least one worker");
    let (job_tx, job_rx) = mpsc::channel::<u64>();
    let (res_tx, res_rx) = mpsc::channel::<u64>();
    // mpsc = single consumer: workers share the one Receiver behind a Mutex.
    let job_rx = Arc::new(Mutex::new(job_rx));

    thread::scope(|s| {
        for _ in 0..n_workers {
            let job_rx = Arc::clone(&job_rx);
            let res_tx = res_tx.clone();
            s.spawn(move || {
                loop {
                    // Lock only to receive; release before computing.
                    let job = job_rx.lock().unwrap().recv();
                    match job {
                        Ok(n) => res_tx.send(n * n).unwrap(),
                        Err(_) => break, // jobs channel closed: we are done
                    }
                }
            });
        }
        drop(res_tx); // keep only the workers' clones alive

        for n in nums {
            job_tx.send(n).unwrap();
        }
        drop(job_tx); // close jobs: workers drain and exit

        res_rx.iter().sum() // ends when the last worker drops its res_tx
    })
}

/// Sends a value through a channel to a worker thread and returns what the
/// worker produced. The `String` is *moved* into the channel: after
/// `send(msg)`, the caller no longer owns it — enforced at compile time.
pub fn shout_via_channel(msg: String) -> String {
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let received: String = rx.recv().unwrap();
        received.to_uppercase()
    });
    tx.send(msg).unwrap();
    handle.join().unwrap()
}

/// Pushes `n` integers through a channel from one producer thread to the
/// calling consumer and returns their sum. `bound` selects the flavour:
/// `None` = unbounded `channel()`, `Some(k)` = `sync_channel(k)` with
/// backpressure (a `Some(0)` channel is a rendezvous: every send waits for
/// its recv).
pub fn throughput(n: u64, bound: Option<usize>) -> u64 {
    enum Tx {
        Unbounded(mpsc::Sender<u64>),
        Bounded(mpsc::SyncSender<u64>),
    }
    let (tx, rx) = match bound {
        None => {
            let (tx, rx) = mpsc::channel();
            (Tx::Unbounded(tx), rx)
        }
        Some(k) => {
            let (tx, rx) = mpsc::sync_channel(k);
            (Tx::Bounded(tx), rx)
        }
    };
    thread::spawn(move || match tx {
        Tx::Unbounded(tx) => {
            for i in 0..n {
                tx.send(i).unwrap();
            }
        }
        Tx::Bounded(tx) => {
            for i in 0..n {
                tx.send(i).unwrap();
            }
        }
    });
    rx.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected_sum_squares(nums: &[u64]) -> u64 {
        nums.iter().map(|n| n * n).sum()
    }

    #[test]
    fn pool_matches_sequential() {
        let nums: Vec<u64> = (1..=1_000).collect();
        let want = expected_sum_squares(&nums);
        assert_eq!(sum_squares_pool(nums, 4), want);
    }

    #[test]
    fn pool_works_with_one_worker() {
        assert_eq!(sum_squares_pool(vec![1, 2, 3], 1), 14);
    }

    #[test]
    fn pool_with_more_workers_than_jobs() {
        assert_eq!(sum_squares_pool(vec![3], 16), 9);
    }

    #[test]
    fn pool_empty_input_is_zero() {
        assert_eq!(sum_squares_pool(vec![], 4), 0);
    }

    #[test]
    fn ownership_transfers_through_channel() {
        assert_eq!(shout_via_channel(String::from("hello")), "HELLO");
    }

    #[test]
    fn throughput_sums_all_flavours() {
        let want = (0..10_000u64).sum::<u64>();
        assert_eq!(throughput(10_000, None), want);
        assert_eq!(throughput(10_000, Some(1024)), want);
        assert_eq!(throughput(10_000, Some(0)), want);
    }
}
