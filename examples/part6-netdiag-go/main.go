package main

// netdiag — one CLI over the Part 6 tools.
//
// Usage: netdiag <scan|ping|trace> <host> [args]
//
// The command parsing and shared report format are the tested library. `scan`
// runs here directly (a TCP connect scan needs no privilege); `ping` and `trace`
// need a raw socket, so this capstone reports the plan and points at the
// dedicated tools from the ping and traceroute lessons.
//
// Run: `go run . scan 127.0.0.1 20-25,80,443`

import (
	"fmt"
	"net"
	"os"
	"sort"
	"strconv"
	"strings"
	"time"
)

func main() {
	cmd, err := ParseCommand(os.Args[1:])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(2)
	}
	switch cmd.Kind {
	case "scan":
		scan(cmd.Host, cmd.Ports)
	case "ping":
		fmt.Println(Banner("ping " + cmd.Host))
		fmt.Println(Section("plan"))
		fmt.Printf("  %d ICMP echo probes to %s\n", cmd.Count, cmd.Host)
		delegatesNote("ping")
	case "trace":
		fmt.Println(Banner("trace " + cmd.Host))
		fmt.Println(Section("plan"))
		fmt.Printf("  up to %d hops to %s\n", cmd.MaxHops, cmd.Host)
		delegatesNote("traceroute")
	}
}

func delegatesNote(tool string) {
	fmt.Println(Section("note"))
	fmt.Printf("  %s needs a raw socket (root / CAP_NET_RAW).\n", tool)
	fmt.Printf("  Run the dedicated `%s` tool from its lesson with sudo.\n", tool)
}

// scan is a compact TCP connect scan — the unprivileged probe from the
// port-scanner lesson, rendered through the shared report format.
func scan(host, spec string) {
	fmt.Println(Banner("scan " + host))
	fmt.Println(Section("open ports"))
	found := 0
	for _, port := range parsePorts(spec) {
		addr := net.JoinHostPort(host, strconv.Itoa(port))
		conn, err := net.DialTimeout("tcp", addr, 300*time.Millisecond)
		if err == nil {
			conn.Close()
			fmt.Printf("  %d/tcp open\n", port)
			found++
		}
	}
	if found == 0 {
		fmt.Println("  (none)")
	}
}

func parsePorts(spec string) []int {
	set := map[int]bool{}
	for _, item := range strings.Split(spec, ",") {
		item = strings.TrimSpace(item)
		if lo, hi, ok := strings.Cut(item, "-"); ok {
			a, errA := strconv.Atoi(strings.TrimSpace(lo))
			b, errB := strconv.Atoi(strings.TrimSpace(hi))
			if errA == nil && errB == nil {
				if a > b {
					a, b = b, a
				}
				for p := a; p <= b; p++ {
					set[p] = true
				}
			}
		} else if p, err := strconv.Atoi(item); err == nil {
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
