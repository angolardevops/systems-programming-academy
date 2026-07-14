//! sysmon — a live terminal performance dashboard.
//!
//! Reads /proc every second and redraws the CPU/MEM/NET/DISK heat bars in
//! place. Reads world-readable /proc — no privileges needed. Ctrl-C to quit.
//!
//! Run: `cargo run --release`

use part6_sysmon_rust::{
    cpu_percent, mem_percent, parse_cpu, parse_disk, parse_mem, parse_net, rate_bps,
    render_dashboard,
};
use std::fs;
use std::thread::sleep;
use std::time::Duration;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn main() {
    // Pick the first non-loopback interface and first disk from a quick probe;
    // fall back to common names.
    let iface = "eth0";
    let disk = "sda";
    let interval = 1.0;

    let mut prev_cpu = parse_cpu(&read("/proc/stat"));
    let (mut prev_rx, mut prev_tx) = parse_net(&read("/proc/net/dev"), iface);
    let (mut prev_rd, mut prev_wr) = parse_disk(&read("/proc/diskstats"), disk);

    print!("\x1b[2J"); // clear screen once
    loop {
        sleep(Duration::from_secs_f64(interval));

        let cur_cpu = parse_cpu(&read("/proc/stat"));
        let (used, total) = parse_mem(&read("/proc/meminfo"));
        let (rx, tx) = parse_net(&read("/proc/net/dev"), iface);
        let (rd, wr) = parse_disk(&read("/proc/diskstats"), disk);

        let cpu = cpu_percent(prev_cpu, cur_cpu);
        let mem = mem_percent(used, total);
        let net = rate_bps(prev_rx + prev_tx, rx + tx, interval);
        let disk_bps = rate_bps(prev_rd + prev_wr, rd + wr, interval) * 512.0; // sectors -> bytes

        print!("\x1b[H"); // cursor home
        println!("  system monitor  (Ctrl-C to quit)\n");
        println!("{}", render_dashboard(cpu, mem, net, disk_bps));

        prev_cpu = cur_cpu;
        (prev_rx, prev_tx) = (rx, tx);
        (prev_rd, prev_wr) = (rd, wr);
    }
}
