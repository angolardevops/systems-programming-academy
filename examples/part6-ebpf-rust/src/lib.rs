//! latencytop — the userspace half of an eBPF latency tool.
//!
//! An eBPF program attached to a tracepoint measures how long something takes
//! (a syscall, a block I/O, a function) and, *in the kernel*, drops each
//! duration into a power-of-two histogram: `hist[log2(duration)]++`. Userspace
//! reads that histogram out of a BPF map and renders it — the iconic output of
//! `biolatency`, `funclatency`, and friends.
//!
//! Loading and attaching the eBPF program needs root and a BPF library (see the
//! lesson). But the *logic* — which bucket a duration falls in, and how the
//! histogram is drawn — is pure arithmetic, identical whether it runs in the
//! kernel or here, and that is what this library tests to the byte.

/// The log2 bucket a value falls into — the same computation the eBPF program
/// runs in-kernel (`bpf_log2l`). Bucket 0 holds 0 and 1; bucket `k` (k ≥ 1)
/// holds `2^k ..= 2^(k+1) - 1`.
pub fn bucket(v: u64) -> usize {
    if v <= 1 {
        0
    } else {
        63 - v.leading_zeros() as usize // floor(log2(v))
    }
}

/// The inclusive `[low, high]` range a bucket covers, for labelling.
pub fn bucket_range(k: usize) -> (u64, u64) {
    if k == 0 {
        (0, 1)
    } else {
        (1 << k, (1 << (k + 1)) - 1)
    }
}

/// Build a histogram from raw duration samples — what the eBPF program does in
/// the kernel, done here so the bucketing can be tested directly. The result is
/// indexed by bucket; `counts[k]` is how many samples landed in bucket `k`.
pub fn tally(samples: &[u64]) -> Vec<u64> {
    let mut counts = Vec::new();
    for &s in samples {
        let b = bucket(s);
        if b >= counts.len() {
            counts.resize(b + 1, 0);
        }
        counts[b] += 1;
    }
    counts
}

const BAR_WIDTH: u64 = 40;

/// Render the classic power-of-two histogram, byte-for-byte reproducible:
///
/// ```text
///             usecs : count    distribution
///            0 -> 1 : 0        |                                        |
///            2 -> 3 : 2        |********************                    |
///            4 -> 7 : 3        |******************************          |
///           8 -> 15 : 4        |****************************************|
///          16 -> 31 : 1        |**********                              |
/// ```
///
/// Rows run from bucket 0 up to the highest non-empty bucket; bar length is the
/// count scaled to the busiest bucket. With no samples, only the header prints.
pub fn render_histogram(counts: &[u64], unit: &str) -> String {
    let mut out = format!("{unit:>18} : count    distribution");
    let last = match counts.iter().rposition(|&c| c > 0) {
        Some(i) => i,
        None => return out,
    };
    let max = *counts[..=last].iter().max().unwrap_or(&0);
    for (k, &count) in counts[..=last].iter().enumerate() {
        let (low, high) = bucket_range(k);
        let range = format!("{low} -> {high}");
        let filled = if max == 0 {
            0
        } else {
            ((count as f64 / max as f64) * BAR_WIDTH as f64 + 0.5).floor() as u64
        };
        let bar: String = "*".repeat(filled as usize) + &" ".repeat((BAR_WIDTH - filled) as usize);
        out.push_str(&format!("\n{range:>18} : {count:<8} |{bar}|"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buckets_follow_powers_of_two() {
        assert_eq!(bucket(0), 0);
        assert_eq!(bucket(1), 0);
        assert_eq!(bucket(2), 1);
        assert_eq!(bucket(3), 1);
        assert_eq!(bucket(4), 2);
        assert_eq!(bucket(7), 2);
        assert_eq!(bucket(8), 3);
        assert_eq!(bucket(1023), 9);
        assert_eq!(bucket(1024), 10);
    }

    #[test]
    fn bucket_ranges_are_inclusive_powers_of_two() {
        assert_eq!(bucket_range(0), (0, 1));
        assert_eq!(bucket_range(1), (2, 3));
        assert_eq!(bucket_range(2), (4, 7));
        assert_eq!(bucket_range(4), (16, 31));
    }

    #[test]
    fn tally_bins_samples_by_bucket() {
        // 1->b0, 2,3->b1, 5,5,6->b2, 9->b3
        let counts = tally(&[1, 2, 3, 5, 5, 6, 9]);
        assert_eq!(counts, vec![1, 2, 3, 1]);
    }

    #[test]
    fn renders_the_histogram() {
        let counts = [0, 2, 3, 4, 1];
        let expected = concat!(
            "             usecs : count    distribution\n",
            "            0 -> 1 : 0        |                                        |\n",
            "            2 -> 3 : 2        |********************                    |\n",
            "            4 -> 7 : 3        |******************************          |\n",
            "           8 -> 15 : 4        |****************************************|\n",
            "          16 -> 31 : 1        |**********                              |",
        );
        assert_eq!(render_histogram(&counts, "usecs"), expected);
    }

    #[test]
    fn renders_header_only_when_empty() {
        assert_eq!(
            render_histogram(&[0, 0, 0], "usecs"),
            "             usecs : count    distribution"
        );
    }

    #[test]
    fn trailing_empty_buckets_are_trimmed() {
        // Highest non-empty bucket is 1, so only rows 0 and 1 render.
        let out = render_histogram(&[3, 1, 0, 0], "nsecs");
        assert_eq!(out.lines().count(), 3); // header + 2 rows
        assert!(out.contains("2 -> 3 : 1"));
        assert!(!out.contains("4 -> 7"));
    }
}
