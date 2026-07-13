// Package main is the Go implementation of the Part 3 health-check agent. All
// targets are probed in parallel (goroutines + WaitGroup) with a connect
// timeout; the report is deterministic and the exit code scripting-friendly.
//
//	go test ./...
//	go build && ./healthcheck targets.conf
package main

import (
	"fmt"
	"net"
	"os"
	"sort"
	"strings"
	"sync"
	"time"
)

// Target is one named probe target.
type Target struct {
	Name string
	Addr string // host:port
}

// Probe is the outcome of probing one target.
type Probe struct {
	Name string
	Up   bool
}

// ParseTargets parses `name = host:port` lines (# comments, blanks tolerated).
func ParseTargets(text string) []Target {
	var targets []Target
	for _, raw := range strings.Split(text, "\n") {
		line := strings.TrimSpace(raw)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		name, addr, found := strings.Cut(line, "=")
		if !found {
			continue
		}
		targets = append(targets, Target{
			Name: strings.TrimSpace(name),
			Addr: strings.TrimSpace(addr),
		})
	}
	return targets
}

// ProbeAddr TCP-probes one address within the timeout.
func ProbeAddr(addr string, timeout time.Duration) bool {
	conn, err := net.DialTimeout("tcp", addr, timeout)
	if err != nil {
		return false
	}
	conn.Close()
	return true
}

// CheckAll probes every target in parallel — Part 1's goroutines doing real
// ops work: N targets cost one timeout, not N.
func CheckAll(targets []Target, timeout time.Duration) []Probe {
	probes := make([]Probe, len(targets))
	var wg sync.WaitGroup
	for i, t := range targets {
		wg.Add(1)
		go func(i int, t Target) {
			defer wg.Done()
			probes[i] = Probe{Name: t.Name, Up: ProbeAddr(t.Addr, timeout)}
		}(i, t)
	}
	wg.Wait()
	sort.Slice(probes, func(i, j int) bool { return probes[i].Name < probes[j].Name })
	return probes
}

// Report renders the shared format and derives the exit code.
func Report(probes []Probe) (string, int) {
	var b strings.Builder
	up := 0
	for _, p := range probes {
		status := "DOWN"
		if p.Up {
			status = "UP"
			up++
		}
		fmt.Fprintf(&b, "%s %s\n", status, p.Name)
	}
	down := len(probes) - up
	fmt.Fprintf(&b, "---\n%d up, %d down\n", up, down)
	code := 0
	if down > 0 {
		code = 1
	}
	return b.String(), code
}

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintln(os.Stderr, "usage: healthcheck <targets-file>")
		os.Exit(2)
	}
	data, err := os.ReadFile(os.Args[1])
	if err != nil {
		fmt.Fprintln(os.Stderr, "healthcheck:", err)
		os.Exit(2)
	}
	probes := CheckAll(ParseTargets(string(data)), 500*time.Millisecond)
	text, code := Report(probes)
	fmt.Print(text)
	os.Exit(code)
}
