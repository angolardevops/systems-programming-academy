package main

// portscan — a concurrent TCP connect scanner.
//
// Usage: portscan <host> [ports]   (ports default: common set)
// No privileges required — this is a TCP connect scan.
//
// Run: `go run . 127.0.0.1 1-1024`

import (
	"fmt"
	"os"
	"time"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintln(os.Stderr, "usage: portscan <host> [ports]   e.g. portscan 127.0.0.1 1-1024")
		os.Exit(2)
	}
	host := os.Args[1]
	spec := "21,22,23,25,53,80,110,143,443,3306,5432,6379,8080"
	if len(os.Args) > 2 {
		spec = os.Args[2]
	}
	ports := ParsePorts(spec)

	fmt.Printf("scanning %s (%d ports)…\n\n", host, len(ports))
	results := ScanAll(host, ports, 500*time.Millisecond, 128)

	var open []Result
	for _, r := range results {
		if r.State == Open {
			open = append(open, r)
		}
	}
	if len(open) == 0 {
		fmt.Println("no open ports found")
	} else {
		fmt.Println(RenderTable(open))
	}
}
