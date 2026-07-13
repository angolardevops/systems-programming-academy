// Package channels shows message passing as Go designed it: goroutines that
// communicate over typed channels instead of sharing state behind locks —
// "Don't communicate by sharing memory; share memory by communicating."
package channels

import "sync"

// SumSquaresPool squares every number using a pool of nWorkers goroutines
// fed by a jobs channel, and sums the results from a results channel.
//
// Closing drives shutdown: close(jobs) ends every worker's range loop;
// when the last worker finishes, a supervisor goroutine closes results,
// which ends the collector's range. No sentinels, no flags.
func SumSquaresPool(nums []uint64, nWorkers int) uint64 {
	if nWorkers < 1 {
		panic("need at least one worker")
	}
	jobs := make(chan uint64)
	results := make(chan uint64)

	var wg sync.WaitGroup
	for range nWorkers {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for n := range jobs { // ends when jobs is closed and drained
				results <- n * n
			}
		}()
	}

	go func() {
		for _, n := range nums {
			jobs <- n
		}
		close(jobs)
	}()

	go func() {
		wg.Wait()
		close(results) // safe: all senders are done
	}()

	var sum uint64
	for r := range results {
		sum += r
	}
	return sum
}

// FirstResponse returns whichever channel delivers first — the select
// statement is Go's race between communications. With one ready channel
// and one that never sends, the ready one always wins.
func FirstResponse(a, b <-chan string) string {
	select {
	case v := <-a:
		return v
	case v := <-b:
		return v
	}
}

// Throughput pushes n integers through a channel with the given buffer size
// (0 = unbuffered rendezvous) from one producer goroutine to the calling
// consumer, and returns their sum.
func Throughput(n uint64, buf int) uint64 {
	ch := make(chan uint64, buf)
	go func() {
		for i := range n {
			ch <- i
		}
		close(ch)
	}()
	var sum uint64
	for v := range ch {
		sum += v
	}
	return sum
}
