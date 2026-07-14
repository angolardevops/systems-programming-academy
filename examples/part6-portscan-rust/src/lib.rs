//! A concurrent TCP port scanner — a `nmap`-lite — that probes a host's ports
//! and prints an elegant results table.
//!
//! It reuses the fan-out/fan-in concurrency of the Part 3 health-check agent:
//! N worker threads pull ports from a shared queue and try to connect. The
//! **TCP connect scan** here is completely unprivileged — it just calls
//! `connect()` like any client, so it runs as any user.
//!
//! Three concerns kept separate and testable:
//! * **Parsing** a port spec (`"22,80,1-1024"`) — pure, tested directly.
//! * **Service lookup** (port → well-known name) — pure, tested directly.
//! * **Scanning** — tested against REAL loopback sockets: a bound listener is a
//!   guaranteed-open port; a bound-then-closed port is a guaranteed-refused one.
//! * **Rendering** the table — a pure string, asserted byte-for-byte.
//!
//! A real `nmap` also does a **SYN scan** (a half-open probe that never
//! completes the handshake), which needs raw sockets and root — the connect
//! scan trades stealth for portability and needs no privileges.

use std::io::ErrorKind;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// The result of probing one port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Open,
    Closed,
    Filtered,
}

impl State {
    pub fn label(self) -> &'static str {
        match self {
            State::Open => "open",
            State::Closed => "closed",
            State::Filtered => "filtered",
        }
    }
}

/// Parses a port spec into a sorted, de-duplicated list. Items are
/// comma-separated; each is a single port (`80`) or an inclusive range
/// (`1-1024`). Invalid items are skipped.
pub fn parse_ports(spec: &str) -> Vec<u16> {
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

/// Returns the well-known service name for a port, or `"unknown"`.
pub fn service_name(port: u16) -> &'static str {
    match port {
        21 => "ftp",
        22 => "ssh",
        23 => "telnet",
        25 => "smtp",
        53 => "dns",
        80 => "http",
        110 => "pop3",
        143 => "imap",
        443 => "https",
        3306 => "mysql",
        5432 => "postgres",
        6379 => "redis",
        8080 => "http-alt",
        _ => "unknown",
    }
}

/// Probes a single socket address with a connect timeout.
/// Connected → Open; connection refused → Closed; timeout/unreachable →
/// Filtered.
pub fn scan_port(addr: SocketAddr, timeout: Duration) -> State {
    match TcpStream::connect_timeout(&addr, timeout) {
        Ok(_) => State::Open,
        Err(e) if e.kind() == ErrorKind::ConnectionRefused => State::Closed,
        Err(_) => State::Filtered,
    }
}

/// Scans every port of `host` concurrently with `workers` threads, returning
/// `(port, state)` pairs sorted by port.
pub fn scan_all(host: &str, ports: &[u16], timeout: Duration, workers: usize) -> Vec<(u16, State)> {
    let results: Arc<Mutex<Vec<(u16, State)>>> = Arc::new(Mutex::new(Vec::new()));
    let next = Arc::new(AtomicUsize::new(0));

    thread::scope(|s| {
        for _ in 0..workers.max(1) {
            let results = Arc::clone(&results);
            let next = Arc::clone(&next);
            s.spawn(move || loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= ports.len() {
                    break;
                }
                let port = ports[i];
                // Resolve per-probe so the scanner works with hostnames too.
                let state = match (host, port)
                    .to_socket_addrs()
                    .ok()
                    .and_then(|mut a| a.next())
                {
                    Some(addr) => scan_port(addr, timeout),
                    None => State::Filtered,
                };
                results.lock().unwrap().push((port, state));
            });
        }
    });

    let mut out = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    out.sort_by_key(|(p, _)| *p);
    out
}

/// Renders a results table for the given `(port, state)` rows (typically the
/// open ones). The exact bytes are the cross-language contract.
pub fn render_table(rows: &[(u16, State)]) -> String {
    let mut lines = vec![format!("{:<10}{:<10}{}", "PORT", "STATE", "SERVICE")];
    for (port, state) in rows {
        lines.push(format!(
            "{:<10}{:<10}{}",
            format!("{port}/tcp"),
            state.label(),
            service_name(*port)
        ));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn parses_ports_and_ranges_sorted_unique() {
        assert_eq!(parse_ports("80"), vec![80]);
        assert_eq!(parse_ports("22,80,443"), vec![22, 80, 443]);
        assert_eq!(parse_ports("1-3"), vec![1, 2, 3]);
        // Overlap + out-of-order + reversed range all normalize.
        assert_eq!(parse_ports("3-1, 2, 80"), vec![1, 2, 3, 80]);
        // Garbage is skipped.
        assert_eq!(parse_ports("22, oops, 90000, 443"), vec![22, 443]);
    }

    #[test]
    fn looks_up_well_known_services() {
        assert_eq!(service_name(22), "ssh");
        assert_eq!(service_name(443), "https");
        assert_eq!(service_name(6379), "redis");
        assert_eq!(service_name(12345), "unknown");
    }

    #[test]
    fn open_port_is_detected_against_a_real_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        assert_eq!(scan_port(addr, Duration::from_secs(1)), State::Open);
    }

    #[test]
    fn closed_port_is_detected_bind_then_close() {
        // Bind to grab a port, then drop the listener so nothing is listening:
        // a connect gets refused — a deterministic "closed" with no fixed port.
        let addr = {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap()
        };
        assert_eq!(scan_port(addr, Duration::from_secs(1)), State::Closed);
    }

    #[test]
    fn scan_all_finds_the_open_ports_among_closed_ones() {
        // Two real listeners (open) and one closed port.
        let l1 = TcpListener::bind("127.0.0.1:0").unwrap();
        let l2 = TcpListener::bind("127.0.0.1:0").unwrap();
        let p1 = l1.local_addr().unwrap().port();
        let p2 = l2.local_addr().unwrap().port();
        let closed = {
            let l = TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap().port()
        };

        let mut ports = vec![p1, p2, closed];
        ports.sort_unstable();
        let results = scan_all("127.0.0.1", &ports, Duration::from_secs(1), 8);

        let open: Vec<u16> = results
            .iter()
            .filter(|(_, s)| *s == State::Open)
            .map(|(p, _)| *p)
            .collect();
        let mut expected = vec![p1, p2];
        expected.sort_unstable();
        assert_eq!(open, expected);
        // The closed port is reported closed, not open.
        assert!(results.contains(&(closed, State::Closed)));
        drop((l1, l2));
    }

    #[test]
    fn renders_the_table() {
        let rows = vec![(22, State::Open), (80, State::Open), (443, State::Open)];
        let table = render_table(&rows);
        let expected = concat!(
            "PORT      STATE     SERVICE\n",
            "22/tcp    open      ssh\n",
            "80/tcp    open      http\n",
            "443/tcp   open      https"
        );
        assert_eq!(table, expected);
    }
}
