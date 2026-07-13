// Package main is the Go implementation of the Part 3 log-analyzer project.
// The same tool is built in Rust, Go, and Python and benchmarked head-to-head.
//
//	go test ./...
//	go build && ./loganalyzer access.log
package main

import (
	"fmt"
	"io"
	"os"
	"sort"
	"strconv"
	"strings"
)

// Entry is one successfully parsed request line.
type Entry struct {
	Path   string
	Status int
}

// ParseLine parses one log line; ok=false means malformed (counted, not fatal).
func ParseLine(line string) (Entry, bool) {
	parts := strings.SplitN(line, `"`, 3)
	if len(parts) != 3 {
		return Entry{}, false
	}
	reqFields := strings.Fields(parts[1])
	if len(reqFields) < 2 {
		return Entry{}, false
	}
	sufFields := strings.Fields(parts[2])
	if len(sufFields) < 1 {
		return Entry{}, false
	}
	status, err := strconv.Atoi(sufFields[0])
	if err != nil || status < 100 || status > 599 {
		return Entry{}, false
	}
	return Entry{Path: reqFields[1], Status: status}, true
}

// Stats aggregates a whole log.
type Stats struct {
	Total     uint64
	Malformed uint64
	ByClass   [5]uint64 // index 0 => 1xx ... 4 => 5xx
	Paths     map[string]uint64
}

// NewStats builds an empty aggregate.
func NewStats() *Stats { return &Stats{Paths: make(map[string]uint64)} }

// AddLine folds one line into the stats.
func (s *Stats) AddLine(line string) {
	entry, ok := ParseLine(line)
	if !ok {
		s.Malformed++
		return
	}
	s.Total++
	s.ByClass[entry.Status/100-1]++
	s.Paths[entry.Path]++
}

// ErrorRate is the percentage of valid requests that were 4xx/5xx.
func (s *Stats) ErrorRate() float64 {
	if s.Total == 0 {
		return 0
	}
	return float64(s.ByClass[3]+s.ByClass[4]) / float64(s.Total) * 100
}

type pathCount struct {
	Path  string
	Count uint64
}

// TopPaths returns the n most-requested paths, count desc then path asc.
func (s *Stats) TopPaths(n int) []pathCount {
	pairs := make([]pathCount, 0, len(s.Paths))
	for p, c := range s.Paths {
		pairs = append(pairs, pathCount{p, c})
	}
	sort.Slice(pairs, func(i, j int) bool {
		if pairs[i].Count != pairs[j].Count {
			return pairs[i].Count > pairs[j].Count
		}
		return pairs[i].Path < pairs[j].Path
	})
	if n > len(pairs) {
		n = len(pairs)
	}
	return pairs[:n]
}

// Report analyzes a whole log text and renders the shared output format.
func Report(input string) string {
	stats := NewStats()
	for _, line := range strings.Split(input, "\n") {
		if strings.TrimSpace(line) != "" {
			stats.AddLine(line)
		}
	}
	var b strings.Builder
	fmt.Fprintf(&b, "total: %d\n", stats.Total)
	for i, label := range []string{"1xx", "2xx", "3xx", "4xx", "5xx"} {
		fmt.Fprintf(&b, "%s: %d\n", label, stats.ByClass[i])
	}
	fmt.Fprintf(&b, "malformed: %d\n", stats.Malformed)
	fmt.Fprintf(&b, "error_rate: %.1f%%\n", stats.ErrorRate())
	b.WriteString("top paths:\n")
	for _, pc := range stats.TopPaths(3) {
		fmt.Fprintf(&b, "  %s: %d\n", pc.Path, pc.Count)
	}
	return b.String()
}

func main() {
	var data []byte
	var err error
	if len(os.Args) > 1 {
		data, err = os.ReadFile(os.Args[1])
	} else {
		data, err = io.ReadAll(os.Stdin)
	}
	if err != nil {
		fmt.Fprintln(os.Stderr, "loganalyzer:", err)
		os.Exit(1)
	}
	fmt.Print(Report(string(data)))
}
