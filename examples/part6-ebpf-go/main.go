package main

// latencytop — render a power-of-two latency histogram.
//
// Usage: latencytop [unit] < samples   (reads whitespace-separated durations on
// stdin, one histogram to stdout)
//
// In production the durations come from an eBPF program attached to a kernel
// tracepoint (see the lesson) — that half needs root and a BPF library. This
// binary is the userspace half: it reads the same numbers from stdin, so you
// can drive it from any source and see the exact output an eBPF tool prints:
//
//	awk 'BEGIN{for(i=0;i<200;i++)print 2**int(rand()*12)}' | go run . usecs

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
)

func main() {
	unit := "usecs"
	if len(os.Args) > 1 {
		unit = os.Args[1]
	}

	var samples []uint64
	sc := bufio.NewScanner(os.Stdin)
	sc.Split(bufio.ScanWords)
	for sc.Scan() {
		if n, err := strconv.ParseUint(sc.Text(), 10, 64); err == nil {
			samples = append(samples, n)
		}
	}
	if len(samples) == 0 {
		fmt.Fprintln(os.Stderr,
			"latencytop: no samples on stdin.\n"+
				"In production an eBPF program feeds durations; here, pipe numbers in, e.g.\n"+
				"printf '1 2 3 5 5 6 9 10 12 100\\n' | latencytop usecs")
		os.Exit(1)
	}

	fmt.Println(RenderHistogram(Tally(samples), unit))
}
