package channels

import "testing"

func TestSumSquaresPoolMatchesSequential(t *testing.T) {
	nums := make([]uint64, 1000)
	var want uint64
	for i := range nums {
		nums[i] = uint64(i) + 1
		want += nums[i] * nums[i]
	}
	if got := SumSquaresPool(nums, 4); got != want {
		t.Fatalf("SumSquaresPool = %d, want %d", got, want)
	}
}

func TestSumSquaresPoolOneWorker(t *testing.T) {
	if got := SumSquaresPool([]uint64{1, 2, 3}, 1); got != 14 {
		t.Fatalf("SumSquaresPool = %d, want 14", got)
	}
}

func TestSumSquaresPoolMoreWorkersThanJobs(t *testing.T) {
	if got := SumSquaresPool([]uint64{3}, 16); got != 9 {
		t.Fatalf("SumSquaresPool = %d, want 9", got)
	}
}

func TestSumSquaresPoolEmpty(t *testing.T) {
	if got := SumSquaresPool(nil, 4); got != 0 {
		t.Fatalf("SumSquaresPool(nil) = %d, want 0", got)
	}
}

func TestFirstResponsePicksTheReadyChannel(t *testing.T) {
	ready := make(chan string, 1)
	ready <- "fast"
	never := make(chan string) // nothing ever sends on this one
	if got := FirstResponse(ready, never); got != "fast" {
		t.Fatalf("FirstResponse = %q, want %q", got, "fast")
	}
	// Order of arguments must not matter.
	ready <- "fast"
	if got := FirstResponse(never, ready); got != "fast" {
		t.Fatalf("FirstResponse = %q, want %q", got, "fast")
	}
}

func TestThroughputBufferedAndUnbuffered(t *testing.T) {
	const n = 10_000
	var want uint64
	for i := uint64(0); i < n; i++ {
		want += i
	}
	if got := Throughput(n, 0); got != want {
		t.Fatalf("Throughput(unbuffered) = %d, want %d", got, want)
	}
	if got := Throughput(n, 1024); got != want {
		t.Fatalf("Throughput(buffered) = %d, want %d", got, want)
	}
}

const benchN = 1_000_000

func BenchmarkThroughputUnbuffered(b *testing.B) {
	for i := 0; i < b.N; i++ {
		Throughput(benchN, 0)
	}
}

func BenchmarkThroughputBuffered1024(b *testing.B) {
	for i := 0; i < b.N; i++ {
		Throughput(benchN, 1024)
	}
}
