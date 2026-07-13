//! Log analyzer — Rust implementation of the Part 3 project. The same tool is
//! built in Rust, Go, and Python and benchmarked head-to-head.
//!
//! Input: Common-Log-Format-style lines, e.g.
//! `203.0.113.9 - - [12/Jul/2026:10:00:00] "GET /api/users HTTP/1.1" 200 512`
//!
//! Output: totals per status class, error rate, top paths — identical across
//! the three implementations so they can be diffed.
//!
//! ```text
//! cargo test
//! cargo run --release -- access.log
//! ```

use std::collections::HashMap;

/// One successfully parsed request line.
#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    pub status: u16,
}

/// Parses one log line; `None` means malformed (counted, never fatal).
pub fn parse_line(line: &str) -> Option<Entry> {
    // The request is the segment between the first pair of double quotes.
    let mut parts = line.splitn(3, '"');
    let _prefix = parts.next()?;
    let request = parts.next()?; // e.g. GET /api/users HTTP/1.1
    let suffix = parts.next()?; // e.g.  200 512

    let path = request.split_whitespace().nth(1)?.to_string();
    let status: u16 = suffix.split_whitespace().next()?.parse().ok()?;
    if !(100..=599).contains(&status) {
        return None;
    }
    Some(Entry { path, status })
}

/// Aggregated statistics over a whole log.
#[derive(Debug, Default, PartialEq)]
pub struct Stats {
    pub total: u64,
    pub malformed: u64,
    pub by_class: [u64; 5], // index 0 => 1xx ... index 4 => 5xx
    pub paths: HashMap<String, u64>,
}

impl Stats {
    /// Folds one line into the stats.
    pub fn add_line(&mut self, line: &str) {
        match parse_line(line) {
            Some(entry) => {
                self.total += 1;
                self.by_class[(entry.status / 100 - 1) as usize] += 1;
                *self.paths.entry(entry.path).or_insert(0) += 1;
            }
            None => self.malformed += 1,
        }
    }

    /// Percentage of valid requests that were 4xx/5xx.
    pub fn error_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.by_class[3] + self.by_class[4]) as f64 / self.total as f64 * 100.0
    }

    /// The `n` most-requested paths, count desc then path asc (deterministic).
    pub fn top_paths(&self, n: usize) -> Vec<(String, u64)> {
        let mut pairs: Vec<(String, u64)> =
            self.paths.iter().map(|(p, c)| (p.clone(), *c)).collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        pairs.truncate(n);
        pairs
    }
}

/// Analyzes a whole log text and renders the report (shared output format).
pub fn report(input: &str) -> String {
    let mut stats = Stats::default();
    for line in input.lines() {
        if !line.trim().is_empty() {
            stats.add_line(line);
        }
    }
    let mut out = String::new();
    out.push_str(&format!("total: {}\n", stats.total));
    for (i, label) in ["1xx", "2xx", "3xx", "4xx", "5xx"].iter().enumerate() {
        out.push_str(&format!("{label}: {}\n", stats.by_class[i]));
    }
    out.push_str(&format!("malformed: {}\n", stats.malformed));
    out.push_str(&format!("error_rate: {:.1}%\n", stats.error_rate()));
    out.push_str("top paths:\n");
    for (path, count) in stats.top_paths(3) {
        out.push_str(&format!("  {path}: {count}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str =
        r#"203.0.113.9 - - [12/Jul/2026:10:00:00] "GET /api/users HTTP/1.1" 200 512"#;

    #[test]
    fn parses_a_valid_line() {
        assert_eq!(
            parse_line(LINE),
            Some(Entry {
                path: "/api/users".into(),
                status: 200
            })
        );
    }

    #[test]
    fn malformed_lines_return_none() {
        assert_eq!(parse_line("not a log line"), None);
        assert_eq!(parse_line(r#"x "GET /a HTTP/1.1" banana 1"#), None);
        assert_eq!(parse_line(r#"x "GET /a HTTP/1.1" 999999 1"#), None);
    }

    #[test]
    fn stats_aggregate_classes_and_malformed() {
        let mut s = Stats::default();
        s.add_line(LINE);
        s.add_line(r#"x - - [t] "GET /a HTTP/1.1" 404 0"#);
        s.add_line(r#"x - - [t] "GET /a HTTP/1.1" 500 0"#);
        s.add_line("garbage");
        assert_eq!(s.total, 3);
        assert_eq!(s.malformed, 1);
        assert_eq!(s.by_class, [0, 1, 0, 1, 1]);
        assert!((s.error_rate() - 66.7).abs() < 0.1);
    }

    #[test]
    fn top_paths_sorted_desc_then_alpha() {
        let mut s = Stats::default();
        for path in ["/b", "/a", "/b", "/c", "/a"] {
            s.add_line(&format!(r#"x - - [t] "GET {path} HTTP/1.1" 200 0"#));
        }
        assert_eq!(
            s.top_paths(2),
            vec![("/a".to_string(), 2), ("/b".to_string(), 2)]
        );
    }

    #[test]
    fn report_renders_the_shared_format() {
        let input = format!("{LINE}\ngarbage\n");
        let rendered = report(&input);
        assert!(rendered.starts_with("total: 1\n"));
        assert!(rendered.contains("malformed: 1\n"));
        assert!(rendered.contains("  /api/users: 1\n"));
    }
}
