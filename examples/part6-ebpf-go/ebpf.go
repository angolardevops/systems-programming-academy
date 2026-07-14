// Package main — latencytop: the userspace half of an eBPF latency tool.
//
// An eBPF program attached to a tracepoint measures how long something takes
// and, in the kernel, drops each duration into a power-of-two histogram:
// hist[log2(duration)]++. Userspace reads that histogram out of a BPF map and
// renders it — the iconic output of biolatency, funclatency, and friends.
//
// Loading and attaching the eBPF program needs root and a BPF library (see the
// lesson). But the logic — which bucket a duration falls in, and how the
// histogram is drawn — is pure arithmetic, tested here to the byte. Library and
// command share one package so `go run .` and `go test` both work.
package main

import (
	"fmt"
	"math"
	"math/bits"
	"strings"
)

// Bucket returns the log2 bucket a value falls into — the same computation the
// eBPF program runs in-kernel (bpf_log2l). Bucket 0 holds 0 and 1; bucket k
// (k >= 1) holds 2^k ..= 2^(k+1)-1.
func Bucket(v uint64) int {
	if v <= 1 {
		return 0
	}
	return bits.Len64(v) - 1 // floor(log2(v))
}

// BucketRange returns the inclusive [low, high] range a bucket covers.
func BucketRange(k int) (uint64, uint64) {
	if k == 0 {
		return 0, 1
	}
	return 1 << k, (1 << (k + 1)) - 1
}

// Tally builds a histogram from raw duration samples — what the eBPF program
// does in the kernel, done here so the bucketing can be tested directly.
func Tally(samples []uint64) []uint64 {
	var counts []uint64
	for _, s := range samples {
		b := Bucket(s)
		if b >= len(counts) {
			grown := make([]uint64, b+1)
			copy(grown, counts)
			counts = grown
		}
		counts[b]++
	}
	return counts
}

const barWidth = 40

// RenderHistogram renders the classic power-of-two histogram, byte-for-byte
// reproducible. Rows run from bucket 0 up to the highest non-empty bucket; bar
// length is the count scaled to the busiest bucket. With no samples, only the
// header prints.
func RenderHistogram(counts []uint64, unit string) string {
	var b strings.Builder
	fmt.Fprintf(&b, "%18s : count    distribution", unit)

	last := -1
	var max uint64
	for i, c := range counts {
		if c > 0 {
			last = i
			if c > max {
				max = c
			}
		}
	}
	if last < 0 {
		return b.String()
	}
	for k := 0; k <= last; k++ {
		low, high := BucketRange(k)
		rng := fmt.Sprintf("%d -> %d", low, high)
		filled := 0
		if max > 0 {
			filled = int(math.Floor(float64(counts[k])/float64(max)*barWidth + 0.5))
		}
		bar := strings.Repeat("*", filled) + strings.Repeat(" ", barWidth-filled)
		fmt.Fprintf(&b, "\n%18s : %-8d |%s|", rng, counts[k], bar)
	}
	return b.String()
}
