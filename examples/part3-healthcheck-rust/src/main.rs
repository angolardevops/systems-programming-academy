//! CLI entry: `healthcheck <targets-file>`; exit 0 if all up, 1 otherwise.

use std::time::Duration;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: healthcheck <targets-file>");
        std::process::exit(2);
    });
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("healthcheck: {path}: {e}");
        std::process::exit(2);
    });
    let targets = healthcheck::parse_targets(&text);
    let probes = healthcheck::check_all(&targets, Duration::from_millis(500));
    let (text, code) = healthcheck::report(&probes);
    print!("{text}");
    std::process::exit(code);
}
