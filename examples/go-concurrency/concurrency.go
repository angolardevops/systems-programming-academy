// Package concurrency is the tested companion code for the Academy lesson
// "Go: Goroutines & Channels". Goroutines are lightweight threads scheduled by
// the Go runtime; channels are typed pipes that let them communicate. Go's motto:
// "Don't communicate by sharing memory; share memory by communicating."
//
// Every function here is deterministic so it can be tested.
//
//	go test -race ./...
package concurrency

import (
	"sort"
	"sync"
)

// SquareAll squares each input concurrently: one goroutine per element, results
// gathered through a channel. Output is sorted so the result is deterministic
// even though the goroutines finish in an unpredictable order.
func SquareAll(nums []int) []int {
	ch := make(chan int)
	for _, n := range nums {
		go func(n int) { // each goroutine captures its own n
			ch <- n * n
		}(n)
	}

	out := make([]int, 0, len(nums))
	for range nums { // receive exactly len(nums) results
		out = append(out, <-ch)
	}
	sort.Ints(out)
	return out
}

// Pipeline wires two stages with channels: generate -> square. Ranging over the
// returned channel drains it until the sending stage closes it.
func Pipeline(nums []int) []int {
	// Stage 1: emit the numbers, then close.
	gen := make(chan int)
	go func() {
		defer close(gen)
		for _, n := range nums {
			gen <- n
		}
	}()

	// Stage 2: square each value, then close.
	sq := make(chan int)
	go func() {
		defer close(sq)
		for n := range gen {
			sq <- n * n
		}
	}()

	out := make([]int, 0, len(nums))
	for n := range sq { // ends when stage 2 closes sq
		out = append(out, n)
	}
	return out
}

// SharedCounter runs `workers` goroutines that each increment a shared counter
// `perWorker` times, using a Mutex to serialise access and a WaitGroup to wait
// for all of them. The total is always workers*perWorker — no lost updates.
func SharedCounter(workers, perWorker int) int {
	var (
		mu    sync.Mutex
		count int
		wg    sync.WaitGroup
	)
	for i := 0; i < workers; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < perWorker; j++ {
				mu.Lock()
				count++
				mu.Unlock()
			}
		}()
	}
	wg.Wait()
	return count
}

// FanInSum starts `workers` goroutines that each sum a chunk of the slice and
// send their partial sum on a channel; the main goroutine adds the partials.
// Demonstrates the fan-out/fan-in pattern with a deterministic total.
func FanInSum(nums []int, workers int) int {
	if workers < 1 {
		workers = 1
	}
	chunkSize := (len(nums) + workers - 1) / workers // ceil division
	partials := make(chan int)

	started := 0
	for i := 0; i < len(nums); i += chunkSize {
		end := i + chunkSize
		if end > len(nums) {
			end = len(nums)
		}
		chunk := nums[i:end]
		started++
		go func(chunk []int) {
			sum := 0
			for _, n := range chunk {
				sum += n
			}
			partials <- sum
		}(chunk)
	}

	total := 0
	for i := 0; i < started; i++ {
		total += <-partials
	}
	return total
}
