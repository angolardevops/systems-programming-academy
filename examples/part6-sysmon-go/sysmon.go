// Command sysmon is a terminal performance dashboard: it reads Linux /proc, computes
// CPU, memory, network, and disk metrics, and render them as colored "heat"
// bars.
//
// The design that keeps it testable separates three concerns: parsing (/proc
// files are plain text, so parsers take a string and tests feed fixtures),
// computing (CPU% and rates are deltas between two snapshots), and rendering
// (the frame is an ANSI-colored string asserted byte-for-byte). A live monitor
// just loops these over real /proc — see main. Nothing needs root: /proc is
// world-readable.
package main

import (
	"fmt"
	"strconv"
	"strings"
)

// Reference maxima to scale the network and disk throughput bars.
const (
	netMaxBps  = 125_000_000.0 // ~1 Gbit/s
	diskMaxBps = 128_000_000.0 // ~128 MB/s
)

// CpuTimes holds busy/total jiffies from the aggregate cpu line of /proc/stat.
type CpuTimes struct {
	Busy  uint64
	Total uint64
}

// ParseCPU parses the aggregate "cpu " line of /proc/stat. Busy is everything
// except idle and iowait.
func ParseCPU(stat string) CpuTimes {
	line := "cpu 0 0 0 0 0 0 0 0"
	for _, l := range strings.Split(stat, "\n") {
		if strings.HasPrefix(l, "cpu ") {
			line = l
			break
		}
	}
	fields := strings.Fields(line)[1:]
	var vals []uint64
	for _, f := range fields {
		n, err := strconv.ParseUint(f, 10, 64)
		if err == nil {
			vals = append(vals, n)
		}
	}
	idle := at(vals, 3) + at(vals, 4)
	var sum uint64
	for _, v := range vals {
		sum += v
	}
	busy := sum - idle
	return CpuTimes{Busy: busy, Total: busy + idle}
}

func at(v []uint64, i int) uint64 {
	if i < len(v) {
		return v[i]
	}
	return 0
}

// CPUPercent is CPU utilisation between two snapshots, in 0..100.
func CPUPercent(prev, cur CpuTimes) float64 {
	totalDelta := sub(cur.Total, prev.Total)
	if totalDelta == 0 {
		return 0
	}
	busyDelta := sub(cur.Busy, prev.Busy)
	return float64(busyDelta) * 100.0 / float64(totalDelta)
}

func sub(a, b uint64) uint64 {
	if a < b {
		return 0
	}
	return a - b
}

// ParseMem returns (usedKB, totalKB) from MemTotal and MemAvailable.
func ParseMem(meminfo string) (uint64, uint64) {
	field := func(name string) uint64 {
		for _, l := range strings.Split(meminfo, "\n") {
			if strings.HasPrefix(l, name) {
				parts := strings.Fields(l)
				if len(parts) >= 2 {
					n, _ := strconv.ParseUint(parts[1], 10, 64)
					return n
				}
			}
		}
		return 0
	}
	total := field("MemTotal:")
	available := field("MemAvailable:")
	return sub(total, available), total
}

// MemPercent is memory used as a percentage.
func MemPercent(usedKB, totalKB uint64) float64 {
	if totalKB == 0 {
		return 0
	}
	return float64(usedKB) * 100.0 / float64(totalKB)
}

// ParseNet returns (rxBytes, txBytes) for iface from /proc/net/dev.
func ParseNet(netdev, iface string) (uint64, uint64) {
	needle := iface + ":"
	for _, l := range strings.Split(netdev, "\n") {
		l = strings.TrimSpace(l)
		if strings.HasPrefix(l, needle) {
			rest := strings.TrimPrefix(l, needle)
			f := strings.Fields(rest)
			var nums []uint64
			for _, x := range f {
				n, err := strconv.ParseUint(x, 10, 64)
				if err == nil {
					nums = append(nums, n)
				}
			}
			return at(nums, 0), at(nums, 8)
		}
	}
	return 0, 0
}

// ParseDisk returns (sectorsRead, sectorsWritten) for dev from /proc/diskstats.
func ParseDisk(diskstats, dev string) (uint64, uint64) {
	for _, l := range strings.Split(diskstats, "\n") {
		f := strings.Fields(l)
		if len(f) > 9 && f[2] == dev {
			r, _ := strconv.ParseUint(f[5], 10, 64)
			w, _ := strconv.ParseUint(f[9], 10, 64)
			return r, w
		}
	}
	return 0, 0
}

// RateBps is bytes-per-second between two counter readings taken secs apart.
func RateBps(prev, cur uint64, secs float64) float64 {
	if secs <= 0 {
		return 0
	}
	return float64(sub(cur, prev)) / secs
}

// FormatRate formats a byte-rate for humans: B/s, KB/s, or MB/s.
func FormatRate(bps float64) string {
	switch {
	case bps >= 1_000_000.0:
		return fmt.Sprintf("%.1f MB/s", bps/1_000_000.0)
	case bps >= 1_000.0:
		return fmt.Sprintf("%.1f KB/s", bps/1_000.0)
	default:
		return fmt.Sprintf("%.0f B/s", bps)
	}
}

func roundHalfUp(x float64) int {
	n := int(x + 0.5)
	if n < 0 {
		return 0
	}
	return n
}

func clamp(x, lo, hi float64) float64 {
	if x < lo {
		return lo
	}
	if x > hi {
		return hi
	}
	return x
}

func heatColor(percent float64) int {
	switch {
	case percent < 50.0:
		return 32 // green
	case percent < 80.0:
		return 33 // yellow
	default:
		return 31 // red
	}
}

// RenderBar renders a colored bar of width cells for percent (clamped 0..100).
func RenderBar(percent float64, width int) string {
	pct := clamp(percent, 0, 100)
	filled := roundHalfUp(pct / 100.0 * float64(width))
	if filled > width {
		filled = width
	}
	color := heatColor(pct)
	return fmt.Sprintf("\x1b[%dm%s%s\x1b[0m",
		color, strings.Repeat("█", filled), strings.Repeat("░", width-filled))
}

// RenderDashboard renders the full frame: four labeled heat bars. The exact
// bytes are the cross-language contract asserted by the tests.
func RenderDashboard(cpuPct, memPct, netBps, diskBps float64) string {
	netPct := clamp(netBps/netMaxBps*100.0, 0, 100)
	diskPct := clamp(diskBps/diskMaxBps*100.0, 0, 100)
	line := func(label, bar, value string) string {
		return fmt.Sprintf(" %-5s%s  %s", label, bar, value)
	}
	return strings.Join([]string{
		line("CPU", RenderBar(cpuPct, 20), fmt.Sprintf("%.1f%%", cpuPct)),
		line("MEM", RenderBar(memPct, 20), fmt.Sprintf("%.1f%%", memPct)),
		line("NET", RenderBar(netPct, 20), FormatRate(netBps)),
		line("DISK", RenderBar(diskPct, 20), FormatRate(diskBps)),
	}, "\n")
}
