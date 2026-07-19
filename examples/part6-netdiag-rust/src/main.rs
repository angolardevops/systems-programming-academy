//! netdiag — one CLI over the Part 6 tools.
//!
//! Usage: `netdiag <scan|ping|trace> <host> [args]`
//!
//! The command parsing and the shared report format are the tested library.
//! `scan` runs here directly (a TCP connect scan needs no privilege); `ping` and
//! `trace` need a raw socket, so this capstone reports the plan and points at the
//! dedicated tools from the ping and traceroute lessons, which do the privileged
//! work. In a production build you would link those in as library calls.

use part6_netdiag_rust::{banner, parse_command, section, Command};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_command(&args) {
        Ok(cmd) => run(cmd),
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(2);
        }
    }
}

fn run(cmd: Command) {
    match cmd {
        Command::Scan { host, ports } => scan(&host, &ports),
        Command::Ping { host, count } => {
            println!("{}", banner(&format!("ping {host}")));
            println!("{}", section("plan"));
            println!("  {count} ICMP echo probes to {host}");
            delegates_note("ping");
        }
        Command::Trace { host, max_hops } => {
            println!("{}", banner(&format!("trace {host}")));
            println!("{}", section("plan"));
            println!("  up to {max_hops} hops to {host}");
            delegates_note("traceroute");
        }
    }
}

fn delegates_note(tool: &str) {
    println!("{}", section("note"));
    println!("  {tool} needs a raw socket (root / CAP_NET_RAW).");
    println!("  Run the dedicated `{tool}` tool from its lesson with sudo.");
}

/// A compact TCP connect scan — the unprivileged probe from the port-scanner
/// lesson, rendered through the shared report format.
fn scan(host: &str, spec: &str) {
    println!("{}", banner(&format!("scan {host}")));
    let ports = parse_ports(spec);
    println!("{}", section("open ports"));
    let timeout = Duration::from_millis(300);
    let mut found = 0;
    for port in ports {
        let addr = match (host, port)
            .to_socket_addrs()
            .ok()
            .and_then(|mut a| a.next())
        {
            Some(a) => a,
            None => continue,
        };
        if is_open(addr, timeout) {
            println!("  {port}/tcp open");
            found += 1;
        }
    }
    if found == 0 {
        println!("  (none)");
    }
}

fn is_open(addr: SocketAddr, timeout: Duration) -> bool {
    TcpStream::connect_timeout(&addr, timeout).is_ok()
}

fn parse_ports(spec: &str) -> Vec<u16> {
    let mut ports = std::collections::BTreeSet::new();
    for item in spec.split(',') {
        let item = item.trim();
        if let Some((lo, hi)) = item.split_once('-') {
            if let (Ok(a), Ok(b)) = (lo.trim().parse::<u16>(), hi.trim().parse::<u16>()) {
                for p in a.min(b)..=a.max(b) {
                    ports.insert(p);
                }
            }
        } else if let Ok(p) = item.parse::<u16>() {
            ports.insert(p);
        }
    }
    ports.into_iter().collect()
}
