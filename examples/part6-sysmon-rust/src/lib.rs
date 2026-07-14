//! A terminal performance dashboard: read Linux `/proc`, compute CPU, memory,
//! network, and disk metrics, and render them as colored "heat" bars.
//!
//! This is the anchor of the command-line-tools part: an elegant TUI built from
//! nothing but string formatting and ANSI escape codes. The design that keeps it
//! *testable* is separating three concerns:
//!
//! * **Parsing** — `/proc/stat`, `/proc/meminfo`, `/proc/net/dev`,
//!   `/proc/diskstats` are plain text; the parsers take a string, so tests feed
//!   fixed snapshots instead of the live, ever-changing kernel.
//! * **Computing** — CPU% and rates are deltas between two snapshots, pure
//!   arithmetic.
//! * **Rendering** — the dashboard frame is a string of ANSI-colored bars,
//!   asserted byte-for-byte (the same cross-language contract as the rest of
//!   this academy).
//!
//! A real live monitor just wraps these in a loop that re-reads `/proc` every
//! second and redraws — the `main` binary does exactly that. Nothing here needs
//! root or any privilege: `/proc` is world-readable.

// Reference maxima used to scale the network and disk throughput bars. Real
// tools auto-scale; fixed references keep the rendering deterministic here.
const NET_MAX_BPS: f64 = 125_000_000.0; // ~1 Gbit/s
const DISK_MAX_BPS: f64 = 128_000_000.0; // ~128 MB/s

/// The busy/total jiffies parsed from the `cpu` line of `/proc/stat`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuTimes {
    pub busy: u64,
    pub total: u64,
}

/// Parses the aggregate `cpu` line of `/proc/stat`.
///
/// Fields: user nice system idle iowait irq softirq steal [guest guest_nice].
/// Busy = everything except idle and iowait.
pub fn parse_cpu(stat: &str) -> CpuTimes {
    let line = stat
        .lines()
        .find(|l| l.starts_with("cpu "))
        .unwrap_or("cpu 0 0 0 0 0 0 0 0");
    let v: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|n| n.parse().ok())
        .collect();
    let idle = v.get(3).copied().unwrap_or(0) + v.get(4).copied().unwrap_or(0);
    let busy: u64 = v.iter().sum::<u64>() - idle;
    CpuTimes {
        busy,
        total: busy + idle,
    }
}

/// CPU utilisation between two snapshots, as a percentage in `0.0..=100.0`.
pub fn cpu_percent(prev: CpuTimes, cur: CpuTimes) -> f64 {
    let total_delta = cur.total.saturating_sub(prev.total);
    if total_delta == 0 {
        return 0.0;
    }
    let busy_delta = cur.busy.saturating_sub(prev.busy);
    busy_delta as f64 * 100.0 / total_delta as f64
}

/// Parses `MemTotal` and `MemAvailable` (in kB) from `/proc/meminfo` and
/// returns `(used_kb, total_kb)`.
pub fn parse_mem(meminfo: &str) -> (u64, u64) {
    let field = |name: &str| -> u64 {
        meminfo
            .lines()
            .find(|l| l.starts_with(name))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|n| n.parse().ok())
            .unwrap_or(0)
    };
    let total = field("MemTotal:");
    let available = field("MemAvailable:");
    (total.saturating_sub(available), total)
}

/// Memory used as a percentage.
pub fn mem_percent(used_kb: u64, total_kb: u64) -> f64 {
    if total_kb == 0 {
        return 0.0;
    }
    used_kb as f64 * 100.0 / total_kb as f64
}

/// Parses the rx+tx byte counters for `iface` from `/proc/net/dev`.
/// Field layout after the `iface:` label: rx_bytes is field 0, tx_bytes is
/// field 8.
pub fn parse_net(netdev: &str, iface: &str) -> (u64, u64) {
    let needle = format!("{iface}:");
    for line in netdev.lines() {
        let line = line.trim_start();
        if let Some(rest) = line.strip_prefix(&needle) {
            let f: Vec<u64> = rest
                .split_whitespace()
                .filter_map(|n| n.parse().ok())
                .collect();
            return (
                f.first().copied().unwrap_or(0),
                f.get(8).copied().unwrap_or(0),
            );
        }
    }
    (0, 0)
}

/// Parses read+write sectors for device `dev` from `/proc/diskstats`.
/// Field layout: device name is field 2, sectors-read is field 5,
/// sectors-written is field 9 (0-indexed).
pub fn parse_disk(diskstats: &str, dev: &str) -> (u64, u64) {
    for line in diskstats.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.get(2) == Some(&dev) {
            let read = f.get(5).and_then(|n| n.parse().ok()).unwrap_or(0);
            let write = f.get(9).and_then(|n| n.parse().ok()).unwrap_or(0);
            return (read, write);
        }
    }
    (0, 0)
}

/// Bytes-per-second between two counter readings taken `secs` apart.
pub fn rate_bps(prev: u64, cur: u64, secs: f64) -> f64 {
    if secs <= 0.0 {
        return 0.0;
    }
    cur.saturating_sub(prev) as f64 / secs
}

/// Formats a byte-rate for humans: `B/s`, `KB/s`, or `MB/s`.
pub fn format_rate(bps: f64) -> String {
    if bps >= 1_000_000.0 {
        format!("{:.1} MB/s", bps / 1_000_000.0)
    } else if bps >= 1_000.0 {
        format!("{:.1} KB/s", bps / 1_000.0)
    } else {
        format!("{:.0} B/s", bps)
    }
}

/// Consistent rounding across languages: round half away from zero for
/// non-negative input.
fn round_half_up(x: f64) -> usize {
    (x + 0.5).floor().max(0.0) as usize
}

/// ANSI color code for a "heat" level: green < 50, yellow < 80, red otherwise.
fn heat_color(percent: f64) -> u8 {
    if percent < 50.0 {
        32 // green
    } else if percent < 80.0 {
        33 // yellow
    } else {
        31 // red
    }
}

/// Renders a colored bar of `width` cells for `percent` (clamped to 0..=100):
/// filled cells in the heat color, empty cells dim.
pub fn render_bar(percent: f64, width: usize) -> String {
    let pct = percent.clamp(0.0, 100.0);
    let filled = round_half_up(pct / 100.0 * width as f64).min(width);
    let color = heat_color(pct);
    format!(
        "\x1b[{color}m{}{}\x1b[0m",
        "█".repeat(filled),
        "░".repeat(width - filled)
    )
}

/// Renders the full dashboard frame: four labeled heat bars. The exact bytes
/// (ANSI codes and all) are the cross-language contract asserted by the tests.
pub fn render_dashboard(cpu_pct: f64, mem_pct: f64, net_bps: f64, disk_bps: f64) -> String {
    let net_pct = (net_bps / NET_MAX_BPS * 100.0).clamp(0.0, 100.0);
    let disk_pct = (disk_bps / DISK_MAX_BPS * 100.0).clamp(0.0, 100.0);
    let line = |label: &str, bar: String, value: String| format!(" {label:<5}{bar}  {value}");
    [
        line("CPU", render_bar(cpu_pct, 20), format!("{cpu_pct:.1}%")),
        line("MEM", render_bar(mem_pct, 20), format!("{mem_pct:.1}%")),
        line("NET", render_bar(net_pct, 20), format_rate(net_bps)),
        line("DISK", render_bar(disk_pct, 20), format_rate(disk_bps)),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const STAT_A: &str = "cpu  1000 0 0 1000 0 0 0 0 0 0\ncpu0 500 0 0 500 0 0 0 0";
    const STAT_B: &str = "cpu  1200 0 0 1200 0 0 0 0 0 0\ncpu0 600 0 0 600 0 0 0 0";
    const MEMINFO: &str = "MemTotal:       16000000 kB\nMemFree:  1000000 kB\nMemAvailable:    4800000 kB\nBuffers: 100 kB";
    const NETDEV_A: &str =
        "Inter-|   Receive\n face |bytes\n  eth0: 1000000 10 0 0 0 0 0 0 2000000 20 0 0 0 0 0 0\n    lo: 5 0 0 0 0 0 0 0 5 0";
    const NETDEV_B: &str =
        "Inter-|   Receive\n face |bytes\n  eth0: 21000000 99 0 0 0 0 0 0 32000000 99 0 0 0 0 0 0\n    lo: 9 0 0 0 0 0 0 0 9 0";
    const DISK_A: &str = "   8       0 sda 100 0 200 0 100 0 300 0 0 0 0";
    const DISK_B: &str = "   8       0 sda 100 0 25000 0 100 0 25000 0 0 0 0";

    #[test]
    fn parses_and_computes_cpu() {
        let a = parse_cpu(STAT_A);
        let b = parse_cpu(STAT_B);
        assert_eq!(
            a,
            CpuTimes {
                busy: 1000,
                total: 2000
            }
        );
        assert_eq!(
            b,
            CpuTimes {
                busy: 1200,
                total: 2400
            }
        );
        // busy delta 200 over total delta 400 = 50%.
        assert_eq!(cpu_percent(a, b), 50.0);
    }

    #[test]
    fn parses_and_computes_memory() {
        let (used, total) = parse_mem(MEMINFO);
        assert_eq!((used, total), (11_200_000, 16_000_000));
        assert_eq!(mem_percent(used, total), 70.0);
    }

    #[test]
    fn parses_net_and_computes_rate() {
        let (rx1, tx1) = parse_net(NETDEV_A, "eth0");
        let (rx2, tx2) = parse_net(NETDEV_B, "eth0");
        assert_eq!((rx1, tx1), (1_000_000, 2_000_000));
        assert_eq!((rx2, tx2), (21_000_000, 32_000_000));
        // (20M rx + 30M tx) over 2s = 25 MB/s total.
        let total = rate_bps(rx1 + tx1, rx2 + tx2, 2.0);
        assert_eq!(total, 25_000_000.0);
        assert_eq!(format_rate(total), "25.0 MB/s");
    }

    #[test]
    fn parses_disk_and_computes_rate() {
        let (r1, w1) = parse_disk(DISK_A, "sda");
        let (r2, w2) = parse_disk(DISK_B, "sda");
        // sectors: read 100->25000 (+24900), write 300... wait fields: read=200, write=300
        assert_eq!((r1, w1), (200, 300));
        assert_eq!((r2, w2), (25000, 25000));
    }

    #[test]
    fn bar_fills_proportionally_with_heat_color() {
        // 50% of 20 cells = 10 filled, yellow (>=50).
        assert_eq!(render_bar(50.0, 20), "\x1b[33m██████████░░░░░░░░░░\x1b[0m");
        // 10% = 2 filled, green.
        assert_eq!(render_bar(10.0, 20), "\x1b[32m██░░░░░░░░░░░░░░░░░░\x1b[0m");
        // 90% = 18 filled, red.
        assert_eq!(render_bar(90.0, 20), "\x1b[31m██████████████████░░\x1b[0m");
    }

    #[test]
    fn renders_the_full_dashboard_frame() {
        // CPU 50%, MEM 70%, NET 25 MB/s (20% of 125MB), DISK 12.8 MB/s (10% of 128MB).
        let frame = render_dashboard(50.0, 70.0, 25_000_000.0, 12_800_000.0);
        let expected = concat!(
            " CPU  \x1b[33m██████████░░░░░░░░░░\x1b[0m  50.0%\n",
            " MEM  \x1b[33m██████████████░░░░░░\x1b[0m  70.0%\n",
            " NET  \x1b[32m████░░░░░░░░░░░░░░░░\x1b[0m  25.0 MB/s\n",
            " DISK \x1b[32m██░░░░░░░░░░░░░░░░░░\x1b[0m  12.8 MB/s"
        );
        assert_eq!(frame, expected);
    }
}
