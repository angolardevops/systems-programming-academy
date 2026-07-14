"""latencytop — the userspace half of an eBPF latency tool.

An eBPF program attached to a tracepoint measures how long something takes and,
in the kernel, drops each duration into a power-of-two histogram:
``hist[log2(duration)]++``. Userspace reads that histogram out of a BPF map and
renders it — the iconic output of biolatency, funclatency, and friends.

Loading and attaching the eBPF program needs root and a BPF library (see the
lesson). But the logic — which bucket a duration falls in, and how the histogram
is drawn — is pure arithmetic, tested here to the byte.
"""

from __future__ import annotations

import math

BAR_WIDTH = 40


def bucket(v: int) -> int:
    """The log2 bucket a value falls into — the same computation the eBPF program
    runs in-kernel (``bpf_log2l``). Bucket 0 holds 0 and 1; bucket ``k`` (k >= 1)
    holds ``2^k ..= 2^(k+1) - 1``."""
    return 0 if v <= 1 else v.bit_length() - 1  # floor(log2(v))


def bucket_range(k: int) -> tuple[int, int]:
    """The inclusive ``[low, high]`` range a bucket covers, for labelling."""
    if k == 0:
        return (0, 1)
    return (1 << k, (1 << (k + 1)) - 1)


def tally(samples: list[int]) -> list[int]:
    """Build a histogram from raw duration samples — what the eBPF program does
    in the kernel, done here so the bucketing can be tested directly."""
    counts: list[int] = []
    for s in samples:
        b = bucket(s)
        if b >= len(counts):
            counts.extend([0] * (b + 1 - len(counts)))
        counts[b] += 1
    return counts


def render_histogram(counts: list[int], unit: str) -> str:
    """Render the classic power-of-two histogram, byte-for-byte reproducible.

    Rows run from bucket 0 up to the highest non-empty bucket; bar length is the
    count scaled to the busiest bucket. With no samples, only the header prints.

    ::

                    usecs : count    distribution
                   0 -> 1 : 0        |                                        |
                   2 -> 3 : 2        |********************                    |
                   4 -> 7 : 3        |******************************          |
                  8 -> 15 : 4        |****************************************|
                 16 -> 31 : 1        |**********                              |
    """
    out = f"{unit:>18} : count    distribution"
    last = -1
    for i, c in enumerate(counts):
        if c > 0:
            last = i
    if last < 0:
        return out
    top = max(counts[: last + 1])
    for k in range(last + 1):
        low, high = bucket_range(k)
        rng = f"{low} -> {high}"
        filled = 0 if top == 0 else math.floor(counts[k] / top * BAR_WIDTH + 0.5)
        bar = "*" * filled + " " * (BAR_WIDTH - filled)
        out += f"\n{rng:>18} : {counts[k]:<8} |{bar}|"
    return out


if __name__ == "__main__":
    import sys

    # latencytop [unit] < samples — reads whitespace-separated durations on
    # stdin. In production an eBPF program feeds these (needs root); here you can
    # pipe numbers from any source and see the exact histogram an eBPF tool
    # prints:  awk 'BEGIN{for(i=0;i<200;i++)print 2**int(rand()*12)}' | python3 ebpf.py
    unit = sys.argv[1] if len(sys.argv) > 1 else "usecs"
    samples = [int(t) for t in sys.stdin.read().split() if t.lstrip("-").isdigit()]
    if not samples:
        print(
            "latencytop: no samples on stdin.\n"
            "In production an eBPF program feeds durations; here, pipe numbers in, e.g.\n"
            "printf '1 2 3 5 5 6 9 10 12 100\\n' | python3 ebpf.py usecs",
            file=sys.stderr,
        )
        raise SystemExit(1)
    print(render_histogram(tally(samples), unit))
