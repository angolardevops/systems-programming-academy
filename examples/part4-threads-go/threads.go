// Package threads shows the three ways to share a counter safely across
// goroutines, plus a parallel sum that actually speeds up.
//
// Unlike Rust, nothing here fails to compile if you remove the mutex — the
// race detector (`go test -race`) is the safety net, and CI runs it.
package threads

import (
	"sync"
	"sync/atomic"
)

// SumParallel sums data by splitting it into nWorkers chunks summed in
// parallel goroutines. A WaitGroup joins them; each goroutine writes to its
// own slot in results, so no locking is needed.
func SumParallel(data []uint64, nWorkers int) uint64 {
	if nWorkers < 1 {
		panic("need at least one worker")
	}
	chunkSize := (len(data) + nWorkers - 1) / nWorkers
	if chunkSize == 0 {
		chunkSize = 1
	}

	var wg sync.WaitGroup
	results := make([]uint64, 0, nWorkers)
	for start := 0; start < len(data); start += chunkSize {
		end := min(start+chunkSize, len(data))
		results = append(results, 0)
		slot := &results[len(results)-1]
		wg.Add(1)
		go func(chunk []uint64) {
			defer wg.Done()
			var sum uint64
			for _, v := range chunk {
				sum += v
			}
			*slot = sum
		}(data[start:end])
	}
	wg.Wait()

	var total uint64
	for _, r := range results {
		total += r
	}
	return total
}

// CounterMutex increments a mutex-protected counter from nGoroutines
// goroutines, iters times each. Always returns exactly nGoroutines*iters.
func CounterMutex(nGoroutines int, iters uint64) uint64 {
	var (
		mu      sync.Mutex
		counter uint64
		wg      sync.WaitGroup
	)
	for range nGoroutines {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for range iters {
				mu.Lock()
				counter++
				mu.Unlock()
			}
		}()
	}
	wg.Wait()
	return counter
}

// CounterAtomic is the lock-free variant: atomic.Uint64.Add compiles to one
// hardware instruction instead of a lock/unlock pair.
func CounterAtomic(nGoroutines int, iters uint64) uint64 {
	var (
		counter atomic.Uint64
		wg      sync.WaitGroup
	)
	for range nGoroutines {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for range iters {
				counter.Add(1)
			}
		}()
	}
	wg.Wait()
	return counter.Load()
}

// CounterBatched accumulates locally and takes the lock once per goroutine:
// contention drops from n*iters lock operations to n.
func CounterBatched(nGoroutines int, iters uint64) uint64 {
	var (
		mu      sync.Mutex
		counter uint64
		wg      sync.WaitGroup
	)
	for range nGoroutines {
		wg.Add(1)
		go func() {
			defer wg.Done()
			var local uint64
			for range iters {
				local++
			}
			mu.Lock()
			counter += local
			mu.Unlock()
		}()
	}
	wg.Wait()
	return counter
}
