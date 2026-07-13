//! Health-check agent — Rust implementation of the Part 3 project. Probes a
//! list of TCP targets **in parallel** (scoped threads) with a connect timeout,
//! and renders a deterministic report with a scripting-friendly exit code.
//!
//! Tests probe real local listeners (up) and freshly-closed ports (down) — no
//! mocks, fully deterministic.
//!
//! ```text
//! cargo test
//! cargo run -- targets.conf   # exit 0 if all up, 1 otherwise
//! ```

use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;

/// One named probe target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub name: String,
    pub addr: String, // host:port
}

/// The outcome of probing one target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    pub name: String,
    pub up: bool,
}

/// Parses `name = host:port` lines (# comments, blanks tolerated).
pub fn parse_targets(text: &str) -> Vec<Target> {
    let mut targets = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, addr)) = line.split_once('=') {
            targets.push(Target {
                name: name.trim().to_string(),
                addr: addr.trim().to_string(),
            });
        }
    }
    targets
}

/// TCP-probes one address: up means a connection was accepted within `timeout`.
pub fn probe(addr: &str, timeout: Duration) -> bool {
    let Ok(mut addrs) = std::net::ToSocketAddrs::to_socket_addrs(&addr) else {
        return false; // unresolvable => down
    };
    let Some(sock_addr): Option<SocketAddr> = addrs.next() else {
        return false;
    };
    TcpStream::connect_timeout(&sock_addr, timeout).is_ok()
}

/// Probes every target **in parallel** using scoped threads (Part 1's
/// `thread::scope`, doing real ops work: N targets cost one timeout, not N).
pub fn check_all(targets: &[Target], timeout: Duration) -> Vec<Probe> {
    let mut probes: Vec<Probe> = thread::scope(|scope| {
        let handles: Vec<_> = targets
            .iter()
            .map(|t| {
                scope.spawn(move || Probe {
                    name: t.name.clone(),
                    up: probe(&t.addr, timeout),
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    probes.sort_by(|a, b| a.name.cmp(&b.name)); // deterministic report order
    probes
}

/// Renders the shared report format; also used to derive the exit code.
pub fn report(probes: &[Probe]) -> (String, i32) {
    let mut out = String::new();
    let mut up = 0;
    for p in probes {
        let status = if p.up { "UP" } else { "DOWN" };
        out.push_str(&format!("{status} {}\n", p.name));
        if p.up {
            up += 1;
        }
    }
    let down = probes.len() - up;
    out.push_str(&format!("---\n{up} up, {down} down\n"));
    let code = if down == 0 { 0 } else { 1 };
    (out, code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn parses_targets_with_comments() {
        let targets = parse_targets("# fleet\napi = 127.0.0.1:8080\n\nweb = 10.0.0.2:80\n");
        assert_eq!(targets.len(), 2);
        assert_eq!(
            targets[0],
            Target {
                name: "api".into(),
                addr: "127.0.0.1:8080".into()
            }
        );
    }

    #[test]
    fn probe_reports_up_for_a_real_listener() {
        // A real listener on an ephemeral port: deterministic "up".
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        assert!(probe(&addr, Duration::from_millis(500)));
    }

    #[test]
    fn probe_reports_down_for_a_closed_port() {
        // Bind, learn the port, drop the listener: deterministic "down".
        let addr = {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().to_string()
        };
        assert!(!probe(&addr, Duration::from_millis(500)));
    }

    #[test]
    fn check_all_probes_in_parallel_and_sorts() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let up_addr = listener.local_addr().unwrap().to_string();
        let down_addr = {
            let l = TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap().to_string()
        };
        let targets = vec![
            Target {
                name: "web".into(),
                addr: up_addr.clone(),
            },
            Target {
                name: "api".into(),
                addr: up_addr,
            },
            Target {
                name: "cache".into(),
                addr: down_addr,
            },
        ];
        let probes = check_all(&targets, Duration::from_millis(500));
        // Sorted by name regardless of completion order:
        let names: Vec<&str> = probes.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["api", "cache", "web"]);
        assert!(probes[0].up && !probes[1].up && probes[2].up);
    }

    #[test]
    fn report_renders_and_sets_exit_code() {
        let probes = vec![
            Probe {
                name: "api".into(),
                up: true,
            },
            Probe {
                name: "cache".into(),
                up: false,
            },
        ];
        let (text, code) = report(&probes);
        assert_eq!(text, "UP api\nDOWN cache\n---\n1 up, 1 down\n");
        assert_eq!(code, 1);

        let (_, all_up_code) = report(&[Probe {
            name: "api".into(),
            up: true,
        }]);
        assert_eq!(all_up_code, 0);
    }
}
