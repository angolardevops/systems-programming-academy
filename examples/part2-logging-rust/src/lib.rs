//! Logging & Observability — Rust companion for the Part 2 lesson. The same
//! design is implemented in Rust, Go, and Python for comparison.
//!
//! Principles demonstrated:
//! - **Structured** logs: key-value fields, one machine-parseable line per event.
//! - **Levels** filter noise: below-threshold events are skipped.
//! - The sink (writer) is **injected**, so tests capture and assert on output.
//! - **Context fields** (e.g. `request_id`) are bound once and appear on every
//!   subsequent line.
//!
//! Timestamps are deliberately omitted here so tests are deterministic; a real
//! logger (e.g. `tracing`) adds them. Run:
//!
//! ```text
//! cargo test
//! ```

use std::fmt;
use std::io::Write;

/// Log severity, ordered so levels can be compared for filtering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        };
        f.write_str(s)
    }
}

/// A minimal structured logger. Generic over any `Write` sink — a file, stderr,
/// or (in tests) a `Vec<u8>` we can inspect.
pub struct Logger<W: Write> {
    sink: W,
    min_level: Level,
    context: Vec<(String, String)>, // bound fields, present on every line
}

impl<W: Write> Logger<W> {
    /// Creates a logger writing to `sink`, dropping events below `min_level`.
    pub fn new(sink: W, min_level: Level) -> Self {
        Logger {
            sink,
            min_level,
            context: Vec::new(),
        }
    }

    /// Returns a value-level "child" by binding a context field the Rust way:
    /// mutate self (builder style) and return it.
    pub fn with_field(mut self, key: &str, value: &str) -> Self {
        self.context.push((key.to_string(), value.to_string()));
        self
    }

    /// Emits one structured line if `level` passes the filter.
    pub fn log(&mut self, level: Level, msg: &str, fields: &[(&str, &str)]) {
        if level < self.min_level {
            return; // filtered out
        }
        // Hand-rolled JSON to stay dependency-free; real code uses serde/tracing.
        let mut line = format!("{{\"level\":\"{level}\",\"msg\":\"{msg}\"");
        for (k, v) in self.context.iter() {
            line.push_str(&format!(",\"{k}\":\"{v}\""));
        }
        for (k, v) in fields {
            line.push_str(&format!(",\"{k}\":\"{v}\""));
        }
        line.push('}');
        // Ignoring the write error keeps the demo simple; production loggers
        // route it to a fallback.
        let _ = writeln!(self.sink, "{line}");
    }

    pub fn info(&mut self, msg: &str, fields: &[(&str, &str)]) {
        self.log(Level::Info, msg, fields);
    }

    pub fn warn(&mut self, msg: &str, fields: &[(&str, &str)]) {
        self.log(Level::Warn, msg, fields);
    }

    pub fn error(&mut self, msg: &str, fields: &[(&str, &str)]) {
        self.log(Level::Error, msg, fields);
    }

    pub fn debug(&mut self, msg: &str, fields: &[(&str, &str)]) {
        self.log(Level::Debug, msg, fields);
    }

    /// Consumes the logger, returning the sink (used by tests to inspect output).
    pub fn into_sink(self) -> W {
        self.sink
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(buf: Vec<u8>) -> Vec<String> {
        String::from_utf8(buf)
            .unwrap()
            .lines()
            .map(String::from)
            .collect()
    }

    #[test]
    fn emits_structured_line_with_fields() {
        let mut log = Logger::new(Vec::new(), Level::Info);
        log.info("user logged in", &[("user_id", "42")]);
        let out = lines(log.into_sink());
        assert_eq!(
            out,
            vec![r#"{"level":"INFO","msg":"user logged in","user_id":"42"}"#]
        );
    }

    #[test]
    fn filters_below_min_level() {
        let mut log = Logger::new(Vec::new(), Level::Warn);
        log.debug("noise", &[]);
        log.info("still noise", &[]);
        log.error("kept", &[]);
        let out = lines(log.into_sink());
        assert_eq!(out.len(), 1);
        assert!(out[0].contains("\"level\":\"ERROR\""));
    }

    #[test]
    fn context_fields_appear_on_every_line() {
        let mut log = Logger::new(Vec::new(), Level::Info).with_field("request_id", "abc-123");
        log.info("start", &[]);
        log.warn("slow query", &[("ms", "250")]);
        let out = lines(log.into_sink());
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|l| l.contains("\"request_id\":\"abc-123\"")));
        assert!(out[1].contains("\"ms\":\"250\""));
    }

    #[test]
    fn level_ordering_supports_filtering() {
        assert!(Level::Debug < Level::Info);
        assert!(Level::Info < Level::Warn);
        assert!(Level::Warn < Level::Error);
    }
}
