"""A terminal performance dashboard: read Linux /proc, compute CPU, memory,
network, and disk metrics, and render them as colored "heat" bars.

The design that keeps it testable separates three concerns: parsing (/proc
files are plain text, so parsers take a string and tests feed fixtures),
computing (CPU% and rates are deltas between two snapshots), and rendering (the
frame is an ANSI-colored string asserted byte-for-byte). A live monitor just
loops these over real /proc — see ``__main__``. Nothing needs root: /proc is
world-readable.
"""

from __future__ import annotations

import math
from dataclasses import dataclass

# Reference maxima to scale the network and disk throughput bars.
NET_MAX_BPS = 125_000_000.0  # ~1 Gbit/s
DISK_MAX_BPS = 128_000_000.0  # ~128 MB/s


@dataclass(frozen=True)
class CpuTimes:
    busy: int
    total: int


def parse_cpu(stat: str) -> CpuTimes:
    """Parse the aggregate ``cpu`` line of /proc/stat. Busy is everything except
    idle and iowait."""
    line = "cpu 0 0 0 0 0 0 0 0"
    for l in stat.splitlines():  # noqa: E741
        if l.startswith("cpu "):
            line = l
            break
    vals = [int(x) for x in line.split()[1:] if x.isdigit()]
    idle = _at(vals, 3) + _at(vals, 4)
    busy = sum(vals) - idle
    return CpuTimes(busy=busy, total=busy + idle)


def _at(seq: list[int], i: int) -> int:
    return seq[i] if i < len(seq) else 0


def cpu_percent(prev: CpuTimes, cur: CpuTimes) -> float:
    """CPU utilisation between two snapshots, in 0..100."""
    total_delta = max(0, cur.total - prev.total)
    if total_delta == 0:
        return 0.0
    busy_delta = max(0, cur.busy - prev.busy)
    return busy_delta * 100.0 / total_delta


def parse_mem(meminfo: str) -> tuple[int, int]:
    """Return ``(used_kb, total_kb)`` from MemTotal and MemAvailable."""

    def field(name: str) -> int:
        for l in meminfo.splitlines():  # noqa: E741
            if l.startswith(name):
                parts = l.split()
                if len(parts) >= 2:
                    return int(parts[1])
        return 0

    total = field("MemTotal:")
    available = field("MemAvailable:")
    return max(0, total - available), total


def mem_percent(used_kb: int, total_kb: int) -> float:
    """Memory used as a percentage."""
    if total_kb == 0:
        return 0.0
    return used_kb * 100.0 / total_kb


def parse_net(netdev: str, iface: str) -> tuple[int, int]:
    """Return ``(rx_bytes, tx_bytes)`` for ``iface`` from /proc/net/dev."""
    needle = iface + ":"
    for l in netdev.splitlines():  # noqa: E741
        l = l.strip()  # noqa: E741
        if l.startswith(needle):
            nums = [int(x) for x in l[len(needle) :].split() if x.isdigit()]
            return _at(nums, 0), _at(nums, 8)
    return 0, 0


def parse_disk(diskstats: str, dev: str) -> tuple[int, int]:
    """Return ``(sectors_read, sectors_written)`` for ``dev`` from
    /proc/diskstats."""
    for l in diskstats.splitlines():  # noqa: E741
        f = l.split()
        if len(f) > 9 and f[2] == dev:
            return int(f[5]), int(f[9])
    return 0, 0


def rate_bps(prev: int, cur: int, secs: float) -> float:
    """Bytes-per-second between two counter readings taken ``secs`` apart."""
    if secs <= 0:
        return 0.0
    return max(0, cur - prev) / secs


def format_rate(bps: float) -> str:
    """Format a byte-rate for humans: B/s, KB/s, or MB/s."""
    if bps >= 1_000_000.0:
        return f"{bps / 1_000_000.0:.1f} MB/s"
    if bps >= 1_000.0:
        return f"{bps / 1_000.0:.1f} KB/s"
    return f"{bps:.0f} B/s"


def _round_half_up(x: float) -> int:
    return max(0, math.floor(x + 0.5))


def _heat_color(percent: float) -> int:
    if percent < 50.0:
        return 32  # green
    if percent < 80.0:
        return 33  # yellow
    return 31  # red


def render_bar(percent: float, width: int) -> str:
    """Render a colored bar of ``width`` cells for ``percent`` (clamped 0..100)."""
    pct = min(100.0, max(0.0, percent))
    filled = min(width, _round_half_up(pct / 100.0 * width))
    color = _heat_color(pct)
    return f"\x1b[{color}m{'█' * filled}{'░' * (width - filled)}\x1b[0m"


def render_dashboard(
    cpu_pct: float, mem_pct: float, net_bps: float, disk_bps: float
) -> str:
    """Render the full frame: four labeled heat bars. The exact bytes are the
    cross-language contract asserted by the tests."""
    net_pct = min(100.0, max(0.0, net_bps / NET_MAX_BPS * 100.0))
    disk_pct = min(100.0, max(0.0, disk_bps / DISK_MAX_BPS * 100.0))

    def line(label: str, bar: str, value: str) -> str:
        return f" {label:<5}{bar}  {value}"

    return "\n".join(
        [
            line("CPU", render_bar(cpu_pct, 20), f"{cpu_pct:.1f}%"),
            line("MEM", render_bar(mem_pct, 20), f"{mem_pct:.1f}%"),
            line("NET", render_bar(net_pct, 20), format_rate(net_bps)),
            line("DISK", render_bar(disk_pct, 20), format_rate(disk_bps)),
        ]
    )


if __name__ == "__main__":
    import time

    def read(path: str) -> str:
        try:
            with open(path) as f:
                return f.read()
        except OSError:
            return ""

    iface, disk = "eth0", "sda"
    interval = 1.0

    prev_cpu = parse_cpu(read("/proc/stat"))
    prev_rx, prev_tx = parse_net(read("/proc/net/dev"), iface)
    prev_rd, prev_wr = parse_disk(read("/proc/diskstats"), disk)

    print("\x1b[2J", end="")
    while True:
        time.sleep(interval)
        cur_cpu = parse_cpu(read("/proc/stat"))
        used, total = parse_mem(read("/proc/meminfo"))
        rx, tx = parse_net(read("/proc/net/dev"), iface)
        rd, wr = parse_disk(read("/proc/diskstats"), disk)

        cpu = cpu_percent(prev_cpu, cur_cpu)
        mem = mem_percent(used, total)
        net = rate_bps(prev_rx + prev_tx, rx + tx, interval)
        disk_bps = rate_bps(prev_rd + prev_wr, rd + wr, interval) * 512.0

        print("\x1b[H", end="")
        print("  system monitor  (Ctrl-C to quit)\n")
        print(render_dashboard(cpu, mem, net, disk_bps))

        prev_cpu = cur_cpu
        prev_rx, prev_tx = rx, tx
        prev_rd, prev_wr = rd, wr
