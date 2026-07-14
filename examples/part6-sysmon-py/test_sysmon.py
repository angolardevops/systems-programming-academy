"""Tests: the same six scenarios as the Rust and Go twins."""

import unittest

from sysmon import (
    CpuTimes,
    cpu_percent,
    format_rate,
    mem_percent,
    parse_cpu,
    parse_disk,
    parse_mem,
    parse_net,
    rate_bps,
    render_bar,
    render_dashboard,
)

STAT_A = "cpu  1000 0 0 1000 0 0 0 0 0 0\ncpu0 500 0 0 500 0 0 0 0"
STAT_B = "cpu  1200 0 0 1200 0 0 0 0 0 0\ncpu0 600 0 0 600 0 0 0 0"
MEMINFO = "MemTotal:       16000000 kB\nMemFree:  1000000 kB\nMemAvailable:    4800000 kB\nBuffers: 100 kB"
NETDEV_A = (
    "Inter-|   Receive\n face |bytes\n"
    "  eth0: 1000000 10 0 0 0 0 0 0 2000000 20 0 0 0 0 0 0\n    lo: 5 0 0 0 0 0 0 0 5 0"
)
NETDEV_B = (
    "Inter-|   Receive\n face |bytes\n"
    "  eth0: 21000000 99 0 0 0 0 0 0 32000000 99 0 0 0 0 0 0\n    lo: 9 0 0 0 0 0 0 0 9 0"
)
DISK_A = "   8       0 sda 100 0 200 0 100 0 300 0 0 0 0"
DISK_B = "   8       0 sda 100 0 25000 0 100 0 25000 0 0 0 0"


class SysmonTest(unittest.TestCase):
    def test_parses_and_computes_cpu(self) -> None:
        a, b = parse_cpu(STAT_A), parse_cpu(STAT_B)
        self.assertEqual(a, CpuTimes(busy=1000, total=2000))
        self.assertEqual(b, CpuTimes(busy=1200, total=2400))
        self.assertEqual(cpu_percent(a, b), 50.0)

    def test_parses_and_computes_memory(self) -> None:
        used, total = parse_mem(MEMINFO)
        self.assertEqual((used, total), (11_200_000, 16_000_000))
        self.assertEqual(mem_percent(used, total), 70.0)

    def test_parses_net_and_computes_rate(self) -> None:
        rx1, tx1 = parse_net(NETDEV_A, "eth0")
        rx2, tx2 = parse_net(NETDEV_B, "eth0")
        self.assertEqual((rx1, tx1), (1_000_000, 2_000_000))
        self.assertEqual((rx2, tx2), (21_000_000, 32_000_000))
        total = rate_bps(rx1 + tx1, rx2 + tx2, 2.0)
        self.assertEqual(total, 25_000_000.0)
        self.assertEqual(format_rate(total), "25.0 MB/s")

    def test_parses_disk(self) -> None:
        self.assertEqual(parse_disk(DISK_A, "sda"), (200, 300))
        self.assertEqual(parse_disk(DISK_B, "sda"), (25000, 25000))

    def test_bar_fills_proportionally_with_heat_color(self) -> None:
        self.assertEqual(render_bar(50.0, 20), "\x1b[33m██████████░░░░░░░░░░\x1b[0m")
        self.assertEqual(render_bar(10.0, 20), "\x1b[32m██░░░░░░░░░░░░░░░░░░\x1b[0m")
        self.assertEqual(render_bar(90.0, 20), "\x1b[31m██████████████████░░\x1b[0m")

    def test_renders_the_full_dashboard_frame(self) -> None:
        frame = render_dashboard(50.0, 70.0, 25_000_000.0, 12_800_000.0)
        expected = (
            " CPU  \x1b[33m██████████░░░░░░░░░░\x1b[0m  50.0%\n"
            " MEM  \x1b[33m██████████████░░░░░░\x1b[0m  70.0%\n"
            " NET  \x1b[32m████░░░░░░░░░░░░░░░░\x1b[0m  25.0 MB/s\n"
            " DISK \x1b[32m██░░░░░░░░░░░░░░░░░░\x1b[0m  12.8 MB/s"
        )
        self.assertEqual(frame, expected)


if __name__ == "__main__":
    unittest.main()
