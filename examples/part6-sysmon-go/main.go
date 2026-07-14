package main

// sysmon — a live terminal performance dashboard.
//
// Reads /proc every second and redraws the CPU/MEM/NET/DISK heat bars in place.
// /proc is world-readable — no privileges needed. Ctrl-C to quit.
//
// Run: `go run .`

import (
	"fmt"
	"os"
	"time"
)

func read(path string) string {
	b, _ := os.ReadFile(path)
	return string(b)
}

func main() {
	const iface, disk = "eth0", "sda"
	const interval = 1.0

	prevCPU := ParseCPU(read("/proc/stat"))
	prevRx, prevTx := ParseNet(read("/proc/net/dev"), iface)
	prevRd, prevWr := ParseDisk(read("/proc/diskstats"), disk)

	fmt.Print("\x1b[2J")
	for {
		time.Sleep(time.Duration(interval * float64(time.Second)))

		curCPU := ParseCPU(read("/proc/stat"))
		used, total := ParseMem(read("/proc/meminfo"))
		rx, tx := ParseNet(read("/proc/net/dev"), iface)
		rd, wr := ParseDisk(read("/proc/diskstats"), disk)

		cpu := CPUPercent(prevCPU, curCPU)
		mem := MemPercent(used, total)
		net := RateBps(prevRx+prevTx, rx+tx, interval)
		diskBps := RateBps(prevRd+prevWr, rd+wr, interval) * 512.0

		fmt.Print("\x1b[H")
		fmt.Println("  system monitor  (Ctrl-C to quit)")
		fmt.Println()
		fmt.Println(RenderDashboard(cpu, mem, net, diskBps))

		prevCPU = curCPU
		prevRx, prevTx = rx, tx
		prevRd, prevWr = rd, wr
	}
}
