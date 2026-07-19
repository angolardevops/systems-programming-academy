//! netdiag — the Part 6 capstone: one command over the five tools.
//!
//! Every tool in Part 6 grew its own `main`. A real network engineer wants them
//! under *one* command with a *consistent* look: `netdiag scan`, `netdiag ping`,
//! `netdiag trace`. This library is the new part that unifying them requires —
//! the **command layer**: parse a subcommand and its arguments into a typed
//! `Command`, and render every tool's output through one shared report format
//! (a boxed banner and section headers). Both are pure and byte-identical across
//! the three languages; the probes themselves are the earlier lessons.

/// A parsed subcommand and its arguments.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// `scan <host> <ports>` — the unprivileged TCP connect scanner.
    Scan { host: String, ports: String },
    /// `ping <host> [count]` — ICMP echo (needs root).
    Ping { host: String, count: u16 },
    /// `trace <host> [max_hops]` — path tracing via TTL (needs root).
    Trace { host: String, max_hops: usize },
}

/// The help text — one line per subcommand.
pub fn usage() -> String {
    "netdiag — network diagnostics\n\
     \n\
     usage:\n\
     \x20 netdiag scan  <host> <ports>     TCP connect scan (e.g. 1-1024)\n\
     \x20 netdiag ping  <host> [count]     ICMP echo, default 4 probes\n\
     \x20 netdiag trace <host> [max_hops]  path trace, default 30 hops"
        .to_string()
}

/// Parse argv-after-the-program-name into a [`Command`], or return the usage
/// text on an unknown subcommand or missing argument.
pub fn parse_command(args: &[String]) -> Result<Command, String> {
    let sub = args.first().ok_or_else(usage)?;
    match sub.as_str() {
        "scan" => {
            let host = args.get(1).ok_or_else(usage)?;
            let ports = args.get(2).ok_or_else(usage)?;
            Ok(Command::Scan {
                host: host.clone(),
                ports: ports.clone(),
            })
        }
        "ping" => {
            let host = args.get(1).ok_or_else(usage)?;
            let count = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
            Ok(Command::Ping {
                host: host.clone(),
                count,
            })
        }
        "trace" => {
            let host = args.get(1).ok_or_else(usage)?;
            let max_hops = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(30);
            Ok(Command::Trace {
                host: host.clone(),
                max_hops,
            })
        }
        other => Err(format!("netdiag: unknown command '{other}'\n\n{}", usage())),
    }
}

const WIDTH: usize = 46;

/// A boxed banner atop each report:
///
/// ```text
/// ╔══════════════════════════════════════════════╗
/// ║  netdiag :: scan example.com                   ║
/// ╚══════════════════════════════════════════════╝
/// ```
pub fn banner(title: &str) -> String {
    let bar = "═".repeat(WIDTH);
    let content = format!("  netdiag :: {title}");
    let pad = WIDTH.saturating_sub(content.chars().count());
    format!("╔{bar}╗\n║{content}{}║\n╚{bar}╝", " ".repeat(pad))
}

/// A section header rule: `── open ports ─────────────…` to a fixed width.
pub fn section(title: &str) -> String {
    let prefix = format!("── {title} ");
    let fill = (WIDTH + 2).saturating_sub(prefix.chars().count());
    format!("{prefix}{}", "─".repeat(fill))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_scan_with_host_and_ports() {
        assert_eq!(
            parse_command(&args(&["scan", "example.com", "1-1024"])),
            Ok(Command::Scan {
                host: "example.com".into(),
                ports: "1-1024".into()
            })
        );
    }

    #[test]
    fn parses_ping_and_trace_with_defaults_and_overrides() {
        assert_eq!(
            parse_command(&args(&["ping", "h"])),
            Ok(Command::Ping {
                host: "h".into(),
                count: 4
            })
        );
        assert_eq!(
            parse_command(&args(&["ping", "h", "7"])),
            Ok(Command::Ping {
                host: "h".into(),
                count: 7
            })
        );
        assert_eq!(
            parse_command(&args(&["trace", "h"])),
            Ok(Command::Trace {
                host: "h".into(),
                max_hops: 30
            })
        );
    }

    #[test]
    fn rejects_unknown_and_missing_arguments() {
        assert!(parse_command(&args(&[])).is_err());
        assert!(parse_command(&args(&["bogus"]))
            .unwrap_err()
            .contains("unknown command 'bogus'"));
        assert!(parse_command(&args(&["scan", "h"])).is_err()); // missing ports
    }

    #[test]
    fn usage_lists_all_three_subcommands() {
        let u = usage();
        assert!(u.contains("netdiag scan"));
        assert!(u.contains("netdiag ping"));
        assert!(u.contains("netdiag trace"));
    }

    #[test]
    fn renders_the_banner() {
        let b = banner("scan example.com");
        let lines: Vec<&str> = b.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], format!("╔{}╗", "═".repeat(46)));
        assert_eq!(lines[2], format!("╚{}╝", "═".repeat(46)));
        // The content line is a box of total width 48 (║ + 46 + ║).
        assert!(lines[1].starts_with("║  netdiag :: scan example.com"));
        assert!(lines[1].ends_with('║'));
        assert_eq!(lines[1].chars().count(), 48);
    }

    #[test]
    fn renders_a_section_rule() {
        let rule = section("open ports");
        assert!(rule.starts_with("── open ports "));
        assert_eq!(rule.chars().count(), 48);
        // The tail is padded with the rule character.
        assert!(rule.ends_with("─────"));
        assert_eq!(rule.chars().filter(|&c| c == '─').count(), 48 - "open ports".len() - 2);
    }
}
