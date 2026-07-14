// Command portscan is a concurrent TCP port scanner — a nmap-lite — that probes
// a host's ports and prints an elegant results table.
//
// It reuses the fan-out/fan-in concurrency of the Part 3 health-check agent: a
// pool of goroutines pulls ports from a channel and tries to connect. The TCP
// connect scan here is completely unprivileged. Pure parts (port-spec parsing,
// service lookup, table rendering) are tested directly; the scan is tested
// against real loopback sockets. A real nmap also does a raw-socket SYN scan,
// which needs root — the connect scan trades stealth for portability.
package main

import (
	"errors"
	"fmt"
	"net"
	"sort"
	"strconv"
	"strings"
	"syscall"
	"time"
)

// State is the result of probing one port.
type State int

const (
	Open State = iota
	Closed
	Filtered
)

// Label returns the lowercase name of the state.
func (s State) Label() string {
	switch s {
	case Open:
		return "open"
	case Closed:
		return "closed"
	default:
		return "filtered"
	}
}

// ParsePorts parses a comma-separated spec of ports and inclusive ranges
// ("22,80,1-1024") into a sorted, de-duplicated list. Invalid items are skipped.
func ParsePorts(spec string) []int {
	set := map[int]bool{}
	for _, item := range strings.Split(spec, ",") {
		item = strings.TrimSpace(item)
		if lo, hi, ok := strings.Cut(item, "-"); ok {
			a, errA := strconv.Atoi(strings.TrimSpace(lo))
			b, errB := strconv.Atoi(strings.TrimSpace(hi))
			if errA == nil && errB == nil && valid(a) && valid(b) {
				if a > b {
					a, b = b, a
				}
				for p := a; p <= b; p++ {
					set[p] = true
				}
			}
		} else if p, err := strconv.Atoi(item); err == nil && valid(p) {
			set[p] = true
		}
	}
	out := make([]int, 0, len(set))
	for p := range set {
		out = append(out, p)
	}
	sort.Ints(out)
	return out
}

func valid(p int) bool { return p >= 0 && p <= 65535 }

// ServiceName returns the well-known service name for a port, or "unknown".
func ServiceName(port int) string {
	names := map[int]string{
		21: "ftp", 22: "ssh", 23: "telnet", 25: "smtp", 53: "dns",
		80: "http", 110: "pop3", 143: "imap", 443: "https",
		3306: "mysql", 5432: "postgres", 6379: "redis", 8080: "http-alt",
	}
	if n, ok := names[port]; ok {
		return n
	}
	return "unknown"
}

// ScanPort probes host:port with a connect timeout. Connected -> Open,
// refused -> Closed, timeout/unreachable -> Filtered.
func ScanPort(host string, port int, timeout time.Duration) State {
	addr := net.JoinHostPort(host, strconv.Itoa(port))
	conn, err := net.DialTimeout("tcp", addr, timeout)
	if err == nil {
		conn.Close()
		return Open
	}
	if errors.Is(err, syscall.ECONNREFUSED) {
		return Closed
	}
	return Filtered
}

// Result pairs a port with its scan state.
type Result struct {
	Port  int
	State State
}

// ScanAll scans every port of host concurrently with `workers` goroutines,
// returning results sorted by port.
func ScanAll(host string, ports []int, timeout time.Duration, workers int) []Result {
	if workers < 1 {
		workers = 1
	}
	jobs := make(chan int)
	results := make(chan Result)

	for range workers {
		go func() {
			for port := range jobs {
				results <- Result{Port: port, State: ScanPort(host, port, timeout)}
			}
		}()
	}
	go func() {
		for _, p := range ports {
			jobs <- p
		}
		close(jobs)
	}()

	out := make([]Result, 0, len(ports))
	for range ports {
		out = append(out, <-results)
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Port < out[j].Port })
	return out
}

// RenderTable renders a results table for the given rows. The exact bytes are
// the cross-language contract.
func RenderTable(rows []Result) string {
	lines := []string{fmt.Sprintf("%-10s%-10s%s", "PORT", "STATE", "SERVICE")}
	for _, r := range rows {
		lines = append(lines, fmt.Sprintf("%-10s%-10s%s",
			strconv.Itoa(r.Port)+"/tcp", r.State.Label(), ServiceName(r.Port)))
	}
	return strings.Join(lines, "\n")
}
