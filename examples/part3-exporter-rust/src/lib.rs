//! Prometheus-style exporter — Rust implementation of the Part 3 project. The
//! same tool is built in Rust, Go, and Python; all three serve byte-identical
//! `/metrics` output for the same registry contents.
//!
//! The core is a pure, tested metrics registry rendering the Prometheus text
//! exposition format; the HTTP layer (in `main.rs`) is a hand-rolled response
//! over `std::net::TcpListener` — no dependencies.
//!
//! ```text
//! cargo test
//! cargo run -- 9100   # then: curl localhost:9100/metrics
//! ```

use std::collections::BTreeMap;

/// Counter or gauge — the two basic Prometheus metric kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Counter,
    Gauge,
}

impl Kind {
    fn as_str(&self) -> &'static str {
        match self {
            Kind::Counter => "counter",
            Kind::Gauge => "gauge",
        }
    }
}

struct Metric {
    help: String,
    kind: Kind,
    // label-string -> value. BTreeMap keeps series sorted => deterministic output.
    series: BTreeMap<String, f64>,
}

/// A metrics registry. BTreeMaps everywhere make `render` deterministic, which
/// is what lets the three implementations be diffed against each other.
#[derive(Default)]
pub struct Registry {
    metrics: BTreeMap<String, Metric>,
}

/// Renders labels as `{k="v",k2="v2"}` with keys sorted; empty -> no braces.
fn label_string(labels: &[(&str, &str)]) -> String {
    if labels.is_empty() {
        return String::new();
    }
    let mut sorted: Vec<_> = labels.to_vec();
    sorted.sort_by_key(|(k, _)| *k);
    let inner: Vec<String> = sorted.iter().map(|(k, v)| format!("{k}=\"{v}\"")).collect();
    format!("{{{}}}", inner.join(","))
}

/// Whole numbers print as integers, others in shortest form — the one rule that
/// keeps Rust/Go/Python output identical.
fn format_value(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

impl Registry {
    fn metric(&mut self, name: &str, help: &str, kind: Kind) -> &mut Metric {
        self.metrics
            .entry(name.to_string())
            .or_insert_with(|| Metric {
                help: help.to_string(),
                kind,
                series: BTreeMap::new(),
            })
    }

    /// Increments a counter series by `delta`.
    pub fn inc_counter(&mut self, name: &str, help: &str, labels: &[(&str, &str)], delta: f64) {
        let key = label_string(labels);
        let m = self.metric(name, help, Kind::Counter);
        *m.series.entry(key).or_insert(0.0) += delta;
    }

    /// Sets a gauge series to `value`.
    pub fn set_gauge(&mut self, name: &str, help: &str, labels: &[(&str, &str)], value: f64) {
        let key = label_string(labels);
        let m = self.metric(name, help, Kind::Gauge);
        m.series.insert(key, value);
    }

    /// Renders the Prometheus text exposition format, fully deterministic.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for (name, metric) in &self.metrics {
            out.push_str(&format!("# HELP {name} {}\n", metric.help));
            out.push_str(&format!("# TYPE {name} {}\n", metric.kind.as_str()));
            for (labels, value) in &metric.series {
                out.push_str(&format!("{name}{labels} {}\n", format_value(*value)));
            }
        }
        out
    }
}

/// Seeds the demo data every implementation shares, so `/metrics` can be
/// diffed across the three languages.
pub fn demo_registry() -> Registry {
    let mut r = Registry::default();
    r.inc_counter(
        "http_requests_total",
        "Total HTTP requests.",
        &[("method", "GET"), ("path", "/")],
        42.0,
    );
    r.inc_counter(
        "http_requests_total",
        "Total HTTP requests.",
        &[("method", "POST"), ("path", "/api")],
        7.0,
    );
    r.set_gauge("queue_depth", "Jobs waiting in the queue.", &[], 3.0);
    r.set_gauge("cpu_load", "1-minute load average.", &[("core", "0")], 0.5);
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_accumulates_per_series() {
        let mut r = Registry::default();
        r.inc_counter("hits", "Hits.", &[("path", "/")], 1.0);
        r.inc_counter("hits", "Hits.", &[("path", "/")], 2.0);
        r.inc_counter("hits", "Hits.", &[("path", "/a")], 5.0);
        let out = r.render();
        assert!(out.contains("hits{path=\"/\"} 3\n"));
        assert!(out.contains("hits{path=\"/a\"} 5\n"));
    }

    #[test]
    fn gauge_overwrites() {
        let mut r = Registry::default();
        r.set_gauge("depth", "Depth.", &[], 9.0);
        r.set_gauge("depth", "Depth.", &[], 3.0);
        assert!(r.render().contains("depth 3\n"));
    }

    #[test]
    fn labels_render_sorted() {
        assert_eq!(label_string(&[("z", "1"), ("a", "2")]), "{a=\"2\",z=\"1\"}");
        assert_eq!(label_string(&[]), "");
    }

    #[test]
    fn values_format_identically_across_languages() {
        assert_eq!(format_value(42.0), "42");
        assert_eq!(format_value(0.5), "0.5");
    }

    #[test]
    fn demo_renders_the_shared_exposition() {
        let out = demo_registry().render();
        assert!(out.starts_with("# HELP cpu_load 1-minute load average.\n"));
        assert!(out.contains("http_requests_total{method=\"GET\",path=\"/\"} 42\n"));
        assert!(out.contains("queue_depth 3\n"));
    }
}
