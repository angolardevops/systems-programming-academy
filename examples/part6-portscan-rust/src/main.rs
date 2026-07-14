//! portscan — a concurrent TCP connect scanner.
//!
//! Usage: portscan <host> [ports]   (ports default: common set)
//! Example: portscan scanme.nmap.org 1-1024
//!
//! No privileges required — this is a TCP connect scan.

use part6_portscan_rust::{parse_ports, render_table, scan_all, State};
use std::process::exit;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: portscan <host> [ports]   e.g. portscan 127.0.0.1 1-1024");
        exit(2);
    }
    let host = &args[1];
    let spec = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "21,22,23,25,53,80,110,143,443,3306,5432,6379,8080".to_string());
    let ports = parse_ports(&spec);

    println!("scanning {host} ({} ports)…\n", ports.len());
    let results = scan_all(host, &ports, Duration::from_millis(500), 128);
    let open: Vec<(u16, State)> = results
        .into_iter()
        .filter(|(_, s)| *s == State::Open)
        .collect();

    if open.is_empty() {
        println!("no open ports found");
    } else {
        println!("{}", render_table(&open));
    }
}
