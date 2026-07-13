package threads

import "testing"

func seq(n uint64) []uint64 {
	data := make([]uint64, n)
	for i := range data {
		data[i] = uint64(i) + 1
	}
	return data
}

func TestSumParallelMatchesSequential(t *testing.T) {
	data := seq(10_000)
	var want uint64
	for _, v := range data {
		want += v
	}
	if got := SumParallel(data, 4); got != want {
		t.Fatalf("SumParallel = %d, want %d", got, want)
	}
}

func TestSumParallelMoreWorkersThanElements(t *testing.T) {
	if got := SumParallel([]uint64{1, 2, 3}, 16); got != 6 {
		t.Fatalf("SumParallel = %d, want 6", got)
	}
}

func TestSumParallelEmpty(t *testing.T) {
	if got := SumParallel(nil, 4); got != 0 {
		t.Fatalf("SumParallel(nil) = %d, want 0", got)
	}
}

func TestCounterMutexIsExact(t *testing.T) {
	if got := CounterMutex(8, 10_000); got != 80_000 {
		t.Fatalf("CounterMutex = %d, want 80000", got)
	}
}

func TestCounterAtomicIsExact(t *testing.T) {
	if got := CounterAtomic(8, 10_000); got != 80_000 {
		t.Fatalf("CounterAtomic = %d, want 80000", got)
	}
}

func TestCounterBatchedIsExact(t *testing.T) {
	if got := CounterBatched(8, 10_000); got != 80_000 {
		t.Fatalf("CounterBatched = %d, want 80000", got)
	}
}

// Benchmarks: run with `go test -bench=. -benchtime=1x` for one-shot timings
// comparable to the Rust and Python harnesses.

const (
	benchGoroutines = 4
	benchIters      = 1_000_000
)

func BenchmarkCounterMutex(b *testing.B) {
	for i := 0; i < b.N; i++ {
		CounterMutex(benchGoroutines, benchIters)
	}
}

func BenchmarkCounterAtomic(b *testing.B) {
	for i := 0; i < b.N; i++ {
		CounterAtomic(benchGoroutines, benchIters)
	}
}

func BenchmarkCounterBatched(b *testing.B) {
	for i := 0; i < b.N; i++ {
		CounterBatched(benchGoroutines, benchIters)
	}
}

func BenchmarkSumParallel1(b *testing.B) {
	data := seq(50_000_000)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		SumParallel(data, 1)
	}
}

func BenchmarkSumParallel4(b *testing.B) {
	data := seq(50_000_000)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		SumParallel(data, 4)
	}
}
